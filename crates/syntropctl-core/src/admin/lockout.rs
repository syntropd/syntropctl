//! Circuit-breaker lockout management and manual operator reset.

use super::journal::log_admin_audit;
use super::status::{sentry_ipc_call, sentry_socket_path};
use crate::error::SyntropctlError;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Result of a manual circuit-breaker lockout reset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockoutResetOutcome {
    pub unit: String,
    pub success: bool,
    pub message: String,
}

/// Manually resets circuit breaker lockout for a unit via sentry IPC.
pub async fn reset_lockout(unit: &str) -> Result<LockoutResetOutcome, SyntropctlError> {
    let sentry_sock = sentry_socket_path();
    if !sentry_sock.exists() {
        return Err(SyntropctlError::DaemonUnavailable {
            daemon: "sentry".into(),
            socket: sentry_sock,
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Sentry socket does not exist",
            ),
        });
    }

    let req = json!({
        "type": "ResetCircuit",
        "payload": {
            "unit": unit
        }
    });

    let data = sentry_ipc_call(&sentry_sock, &req).await?;
    let reset = data
        .get("reset")
        .and_then(|r| r.as_bool())
        .unwrap_or(false);

    let incident_id = format!(
        "reset-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );

    if reset {
        log_admin_audit(
            &incident_id,
            unit,
            "lockout_reset",
            "success",
            &format!("Circuit breaker for {unit} manually reset to CLOSED"),
        );
        Ok(LockoutResetOutcome {
            unit: unit.to_string(),
            success: true,
            message: format!("Circuit breaker for {unit} successfully reset to CLOSED"),
        })
    } else {
        log_admin_audit(
            &incident_id,
            unit,
            "lockout_reset",
            "failure",
            &format!("Failed to reset circuit breaker for {unit}"),
        );
        Err(SyntropctlError::OperationFailed(format!(
            "Sentry rejected circuit reset for unit {unit}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockout_reset_outcome_serialization() {
        let outcome = LockoutResetOutcome {
            unit: "test.service".into(),
            success: true,
            message: "Reset OK".into(),
        };
        let val = serde_json::to_value(&outcome).unwrap();
        assert_eq!(val["unit"], "test.service");
        assert_eq!(val["success"], true);
    }
}
