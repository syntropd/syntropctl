//! Status query inspecting circuit-breaker states and autonomous healing health.

use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// Information for a single tracked circuit breaker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CircuitBreakerInfo {
    pub unit: String,
    pub state: String,
    pub cooldown_remaining_secs: u64,
    pub recent_failures: usize,
    pub permanently_locked: bool,
    pub allows_remediation: bool,
}

/// Comprehensive status of the autonomous healing subsystem.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdminStatusReport {
    pub healing_enabled: bool,
    pub sentry_online: bool,
    pub toold_online: bool,
    pub contextd_online: bool,
    pub routerd_online: bool,
    pub circuit_breakers: HashMap<String, CircuitBreakerInfo>,
    pub locked_out_units: Vec<String>,
}

/// Sends an IPC request to sentry and returns the data payload.
pub(crate) async fn sentry_ipc_call(
    socket: &Path,
    request: &Value,
) -> Result<Value, SyntropctlError> {
    let stream = UnixStream::connect(socket)
        .await
        .map_err(|e| SyntropctlError::DaemonUnavailable {
            daemon: "sentry".into(),
            socket: socket.to_path_buf(),
            source: e,
        })?;
    let (read_half, mut write_half) = stream.into_split();
    let mut payload = serde_json::to_vec(request)?;
    payload.push(b'\n');
    write_half.write_all(&payload).await?;
    write_half.flush().await?;

    let mut reader = BufReader::new(read_half);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    if line.is_empty() {
        return Err(SyntropctlError::MalformedReply("Empty IPC reply".into()));
    }
    let resp: Value = serde_json::from_str(&line)?;
    if resp.get("status").and_then(|s| s.as_str()) == Some("error") {
        let msg = resp
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Sentry IPC error");
        return Err(SyntropctlError::OperationFailed(msg.to_string()));
    }
    Ok(resp.get("data").cloned().unwrap_or(Value::Null))
}

/// Resolves the socket path for the sentry supervisor daemon.
pub fn sentry_socket_path() -> PathBuf {
    DaemonEndpoint::from_name("sentry")
        .map(|d| d.socket_path())
        .unwrap_or_else(|| PathBuf::from("/run/systemd-sentry/sentry.sock"))
}

/// Parses circuit breaker snapshots from a sentry Status reply.
pub fn parse_circuit_breakers(data: &Value) -> HashMap<String, CircuitBreakerInfo> {
    let mut map = HashMap::new();
    let Some(raw_breakers) = data.get("circuit_breakers").and_then(|v| v.as_object()) else {
        return map;
    };
    for (unit, obj) in raw_breakers {
        let state = obj
            .get("state")
            .and_then(|s| s.as_str())
            .unwrap_or("UNKNOWN")
            .to_string();
        let cooldown = obj
            .get("cooldown_remaining_secs")
            .and_then(|c| c.as_u64())
            .unwrap_or(0);
        let failures = obj
            .get("recent_failures")
            .and_then(|f| f.as_u64())
            .unwrap_or(0) as usize;
        let permanently_locked = obj
            .get("permanently_locked")
            .and_then(|p| p.as_bool())
            .unwrap_or(false);
        let allows = (state == "CLOSED" || state == "HALF_OPEN") && !permanently_locked;
        map.insert(
            unit.clone(),
            CircuitBreakerInfo {
                unit: unit.clone(),
                state,
                cooldown_remaining_secs: cooldown,
                recent_failures: failures,
                permanently_locked,
                allows_remediation: allows,
            },
        );
    }
    map
}

/// Queries overall autonomous healing status across daemons.
pub async fn query_admin_status() -> Result<AdminStatusReport, SyntropctlError> {
    let sentry_sock = sentry_socket_path();
    let toold_sock = DaemonEndpoint::from_name("toold")
        .map(|d| d.socket_path())
        .unwrap_or_else(|| PathBuf::from("/run/syntrop/io.syntrop.Tool1"));
    let contextd_sock = DaemonEndpoint::from_name("contextd")
        .map(|d| d.socket_path())
        .unwrap_or_else(|| PathBuf::from("/run/syntrop/io.syntrop.Context1"));
    let routerd_sock = DaemonEndpoint::from_name("routerd")
        .map(|d| d.socket_path())
        .unwrap_or_else(|| PathBuf::from("/run/syntrop/io.syntrop.Router1"));

    let toold_online = toold_sock.exists();
    let contextd_online = contextd_sock.exists();
    let routerd_online = routerd_sock.exists();

    let mut circuit_breakers = HashMap::new();
    let mut sentry_online = false;

    if sentry_sock.exists() {
        let req = json!({ "type": "Status" });
        if let Ok(data) = sentry_ipc_call(&sentry_sock, &req).await {
            sentry_online = true;
            circuit_breakers = parse_circuit_breakers(&data);
        }
    }

    let mut locked_out_units = Vec::new();
    for (unit, info) in &circuit_breakers {
        if !info.allows_remediation {
            locked_out_units.push(unit.clone());
        }
    }
    locked_out_units.sort();

    let healing_enabled = sentry_online && toold_online;

    Ok(AdminStatusReport {
        healing_enabled,
        sentry_online,
        toold_online,
        contextd_online,
        routerd_online,
        circuit_breakers,
        locked_out_units,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_circuit_breakers_empty() {
        let empty = json!({});
        let parsed = parse_circuit_breakers(&empty);
        assert!(parsed.is_empty());
    }

    #[test]
    fn test_parse_circuit_breakers_populated() {
        let data = json!({
            "circuit_breakers": {
                "foo.service": {
                    "state": "CLOSED",
                    "cooldown_remaining_secs": 0,
                    "recent_failures": 0,
                    "permanently_locked": false
                },
                "bar.service": {
                    "state": "PERMANENTLY_LOCKED",
                    "cooldown_remaining_secs": 0,
                    "recent_failures": 5,
                    "permanently_locked": true
                },
                "baz.service": {
                    "state": "OPEN",
                    "cooldown_remaining_secs": 30,
                    "recent_failures": 3,
                    "permanently_locked": false
                }
            }
        });
        let parsed = parse_circuit_breakers(&data);
        assert_eq!(parsed.len(), 3);
        assert!(parsed["foo.service"].allows_remediation);
        assert!(!parsed["bar.service"].allows_remediation);
        assert!(!parsed["baz.service"].allows_remediation);
    }
}
