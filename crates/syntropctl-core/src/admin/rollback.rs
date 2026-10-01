//! Rollback execution reverting configuration and filesystem mutations via toold.

use super::journal::log_admin_audit;
use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

/// Outcome of a configuration or filesystem rollback operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RollbackOutcome {
    pub unit: String,
    pub rollback_id: String,
    pub target_path: Option<String>,
    pub summary: String,
    pub success: bool,
}

/// Resolves the socket path for the toold sandboxed execution broker.
fn toold_socket_path() -> PathBuf {
    DaemonEndpoint::from_name("toold")
        .map(|d| d.socket_path())
        .unwrap_or_else(|| PathBuf::from("/run/syntrop/io.syntrop.Tool1"))
}

/// Reverts state modifications for a unit using a specific or most recent snapshot.
pub async fn execute_rollback(
    unit: &str,
    snapshot_id: Option<&str>,
) -> Result<RollbackOutcome, SyntropctlError> {
    let toold_sock = toold_socket_path();
    if !toold_sock.exists() {
        return Err(SyntropctlError::DaemonUnavailable {
            daemon: "toold".into(),
            socket: toold_sock,
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Toold socket does not exist",
            ),
        });
    }

    let target_id = match snapshot_id {
        Some(id) => id.to_string(),
        None => {
            let list_params = json!({
                "since_seconds": 86400,
                "limit": 100
            });
            let reply = VarlinkClient::call(
                &toold_sock,
                "io.syntrop.Tool1.ListRollbacks",
                Some(list_params),
                DEFAULT_RPC_TIMEOUT,
            )
            .await?;

            let mut matched_id = None;
            if let Some(records) = reply.get("records").and_then(|r| r.as_array()) {
                for rec in records.iter().rev() {
                    let target_unit = rec.get("target_unit").and_then(|u| u.as_str());
                    let summary = rec.get("summary").and_then(|s| s.as_str()).unwrap_or("");
                    if target_unit == Some(unit) || summary.contains(unit) {
                        if let Some(id) = rec.get("id").and_then(|i| i.as_str()) {
                            matched_id = Some(id.to_string());
                            break;
                        }
                    }
                }
            }

            matched_id.ok_or_else(|| {
                SyntropctlError::NotFound(format!(
                    "No rollback snapshots found for unit {unit}"
                ))
            })?
        }
    };

    let rollback_params = json!({ "rollback_id": target_id });
    let res = match VarlinkClient::call(
        &toold_sock,
        "io.syntrop.Tool1.Rollback",
        Some(rollback_params),
        DEFAULT_RPC_TIMEOUT,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            log_admin_audit(
                &format!("rb-{target_id}"),
                unit,
                "rollback",
                "failure",
                &format!("Failed to roll back state for {unit} using snapshot {target_id}: {e}"),
            );
            return Err(e);
        }
    };

    let restored = res.get("restored").ok_or_else(|| {
        SyntropctlError::MalformedReply("Missing 'restored' object in Rollback reply".into())
    })?;

    let restored_id = restored
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or(&target_id)
        .to_string();
    let target_path = restored
        .get("target_path")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let summary = restored
        .get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("Rollback restored successfully")
        .to_string();

    let incident_id = format!("rb-{}", restored_id);
    log_admin_audit(
        &incident_id,
        unit,
        "rollback",
        "success",
        &format!("Rolled back state for {unit} using snapshot {restored_id}"),
    );

    Ok(RollbackOutcome {
        unit: unit.to_string(),
        rollback_id: restored_id,
        target_path,
        summary,
        success: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollback_outcome_structure() {
        let outcome = RollbackOutcome {
            unit: "my-service.service".into(),
            rollback_id: "rb-123".into(),
            target_path: Some("/etc/my-service/conf.d/test.conf".into()),
            summary: "Restored previous config".into(),
            success: true,
        };
        assert!(outcome.success);
        assert_eq!(outcome.rollback_id, "rb-123");
    }
}
