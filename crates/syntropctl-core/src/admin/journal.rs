//! Structured audit logging to systemd-journald and audit inspection.

use crate::error::SyntropctlError;
use serde::{Deserialize, Serialize};
use std::io::BufRead;
use std::os::unix::net::UnixDatagram;
use std::path::Path;
use tokio::process::Command;

/// Structured audit record of an administrative or self-healing action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdminAuditEntry {
    #[serde(alias = "INCIDENT_ID", default)]
    pub incident_id: String,
    #[serde(alias = "__REALTIME_TIMESTAMP", default)]
    pub timestamp: String,
    #[serde(alias = "UNIT", default)]
    pub unit: String,
    #[serde(alias = "REMEDIATION_ACTION", default)]
    pub action: String,
    #[serde(alias = "RESULT", default)]
    pub result: String,
    #[serde(alias = "MESSAGE", default)]
    pub message: String,
}

/// Emits structured audit fields exclusively to systemd-journald.
pub fn log_admin_audit(
    incident_id: &str,
    unit: &str,
    action: &str,
    result: &str,
    message: &str,
) {
    let payload = format!(
        "MESSAGE={message}\n\
         PRIORITY=6\n\
         SYSLOG_IDENTIFIER=syntrop-admin\n\
         INCIDENT_ID={incident_id}\n\
         UNIT={unit}\n\
         REMEDIATION_ACTION={action}\n\
         RESULT={result}\n"
    );

    let sock = "/run/systemd/journal/socket";
    if Path::new(sock).exists() {
        if let Ok(datagram) = UnixDatagram::unbound() {
            let _ = datagram.send_to(payload.as_bytes(), sock);
        }
    }
}

/// Queries forensic audit records exclusively from systemd-journald.
pub async fn query_admin_audit(
    unit: Option<&str>,
    limit: usize,
) -> Result<Vec<AdminAuditEntry>, SyntropctlError> {
    let mut entries = Vec::new();

    let mut cmd = Command::new("journalctl");
    cmd.args([
        "SYSLOG_IDENTIFIER=syntrop-admin",
        "-o",
        "json",
        "-n",
        &limit.to_string(),
    ]);
    if let Some(target_unit) = unit {
        cmd.arg(format!("UNIT={target_unit}"));
    }

    if let Ok(out) = cmd.output().await {
        if out.status.success() {
            let cursor = std::io::Cursor::new(out.stdout);
            for line in cursor.lines().map_while(Result::ok) {
                if let Ok(e) = serde_json::from_str::<AdminAuditEntry>(&line) {
                    if !e.action.is_empty() || !e.result.is_empty() {
                        entries.push(e);
                    }
                }
            }
        }
    }

    if let Some(target_unit) = unit {
        entries.retain(|e| e.unit == target_unit);
    }

    if entries.len() > limit {
        entries.truncate(limit);
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_parsing() {
        let json_line = r#"{"INCIDENT_ID":"inc-123","UNIT":"nginx.service","REMEDIATION_ACTION":"restart","RESULT":"success","MESSAGE":"remediation completed"}"#;
        let entry: AdminAuditEntry = serde_json::from_str(json_line).unwrap();
        assert_eq!(entry.incident_id, "inc-123");
        assert_eq!(entry.unit, "nginx.service");
        assert_eq!(entry.action, "restart");
        assert_eq!(entry.result, "success");
    }
}
