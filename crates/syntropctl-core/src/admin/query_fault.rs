//! Multi-daemon fault inquiry querying sentry, contextd, and routerd.

use super::status::{sentry_ipc_call, sentry_socket_path};
use crate::daemon::DaemonEndpoint;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn daemon_socket(name: &str, fallback: &str) -> PathBuf {
    DaemonEndpoint::from_name(name)
        .map(|d| d.socket_path())
        .unwrap_or_else(|| PathBuf::from(fallback))
}

/// Queries sentry supervisor for circuit breaker clearance and fault classification.
pub async fn query_sentry_safety(unit: &str) -> (bool, String, String) {
    let sock = sentry_socket_path();
    if !sock.exists() {
        return (true, "NoActiveRecord".into(), format!("inc-{}", unit));
    }
    let mut allowed = true;
    if let Ok(data) = sentry_ipc_call(&sock, &json!({ "type": "Status" })).await {
        if let Some(b) = data.get("circuit_breakers").and_then(|b| b.get(unit)) {
            let st = b.get("state").and_then(|s| s.as_str()).unwrap_or("");
            let locked = b.get("permanently_locked").and_then(|l| l.as_bool()).unwrap_or(false);
            if st == "OPEN" || st == "PERMANENTLY_LOCKED" || locked {
                allowed = false;
            }
        }
    }
    let mut fault_class = "UnclassifiedFault".to_string();
    let mut incident_id = format!("inc-{}", unit);
    let list_req = json!({ "type": "ListIncidents", "payload": { "limit": 20 } });
    if let Ok(data) = sentry_ipc_call(&sock, &list_req).await {
        if let Some(incidents) = data.as_array() {
            for inc in incidents {
                let matches = inc.get("unit_name").or_else(|| inc.get("unit")).and_then(|u| u.as_str()) == Some(unit);
                if matches {
                    if let Some(id) = inc.get("incident_id").and_then(|i| i.as_str()) {
                        incident_id = id.to_string();
                    }
                    if let Some(s) = inc.get("root_cause").and_then(|r| r.get("summary")).and_then(|s| s.as_str()) {
                        fault_class = s.to_string();
                    } else if let Some(rc) = inc.get("root_cause").and_then(|r| r.as_str()) {
                        fault_class = rc.to_string();
                    } else if let Some(fc) = inc.get("failure_type").and_then(|f| f.as_str()) {
                        fault_class = fc.to_string();
                    }
                    break;
                }
            }
        }
    }
    (allowed, fault_class, incident_id)
}

/// Inspects contextd for recent file diffs and configuration drift.
pub async fn inspect_context_drift(unit: &str) -> (bool, String) {
    let sock = daemon_socket("contextd", "/run/syntrop/io.syntrop.Context1");
    if sock.exists() {
        let p = json!({ "unit": unit, "since_seconds": 3600 });
        if let Ok(val) = VarlinkClient::call(&sock, "io.syntrop.Context1.GetUnitContext", Some(p), DEFAULT_RPC_TIMEOUT).await {
            if let Some(ctx) = val.get("context") {
                let diffs = ctx.get("config_diffs").and_then(|d| d.as_array());
                let has = diffs.map(|d| !d.is_empty()).unwrap_or(false);
                let sum = ctx.get("summary").and_then(|s| s.as_str()).unwrap_or("");
                return (has, sum.to_string());
            }
        }
    }
    (false, "No causal drift recorded".into())
}

async fn query_routerd_http(prompt: &str) -> Option<String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let sock = Path::new("/run/syntrop/router.sock");
    if !sock.exists() {
        return None;
    }
    let mut stream = tokio::net::UnixStream::connect(sock).await.ok()?;
    let body = json!({
        "model": "router:auto",
        "messages": [{ "role": "user", "content": prompt }],
        "max_tokens": 128
    });
    let bytes = serde_json::to_vec(&body).ok()?;
    let req = format!(
        "POST /v1/chat/completions HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        bytes.len()
    );
    stream.write_all(req.as_bytes()).await.ok()?;
    stream.write_all(&bytes).await.ok()?;
    stream.flush().await.ok()?;
    let mut out = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(5), stream.read_to_end(&mut out)).await.ok()?;
    let text = String::from_utf8_lossy(&out);
    let body_str = text.split("\r\n\r\n").nth(1)?;
    let val: serde_json::Value = serde_json::from_str(body_str.trim()).ok()?;
    val.get("choices")?.get(0)?.get("message")?.get("content")?.as_str().map(ToString::to_string)
}

/// Dispatches novel or unclassified failure context to routerd for reasoning.
pub async fn query_routerd_fallback(unit: &str, fault: &str, drift: &str) -> Option<String> {
    let prompt = format!("Unit '{unit}' failed: {fault}. Drift: {drift}. Plan recovery.");
    if let Some(ans) = query_routerd_http(&prompt).await {
        return Some(ans);
    }
    let r_sock = daemon_socket("routerd", "/run/syntrop/io.syntrop.Router1");
    if r_sock.exists() {
        let p = json!({ "model": "router:auto" });
        if let Ok(res) = VarlinkClient::call(&r_sock, "io.syntrop.Router1.RouteRequest", Some(p), DEFAULT_RPC_TIMEOUT).await {
            if let Some(c) = res.get("candidates").and_then(|c| c.as_array()) {
                let names: Vec<&str> = c.iter().filter_map(|m| m.get("model_name").and_then(|s| s.as_str())).collect();
                if !names.is_empty() {
                    return Some(format!("Available candidate models: {}", names.join(", ")));
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_sentry_safety_unknown_unit() {
        let (allowed, fault, inc) = query_sentry_safety("nonexistent.service").await;
        assert!(allowed);
        assert!(!fault.is_empty());
        assert!(inc.contains("nonexistent.service"));
    }
}
