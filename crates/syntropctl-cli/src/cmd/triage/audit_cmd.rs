//! Audit trail query command for systemd-journald and sentry audit log.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use tokio::process::Command;

/// Audit entry deserialized from journal or audit.log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    #[serde(alias = "SYNTROP_INCIDENT_ID", default)]
    pub incident_id: String,
    #[serde(alias = "__REALTIME_TIMESTAMP", default)]
    pub timestamp: String,
    #[serde(alias = "SYNTROP_UNIT", default)]
    pub unit: String,
    #[serde(alias = "SYNTROP_FAULT_CLASS", default)]
    pub fault_class: String,
    #[serde(alias = "SYNTROP_TIER", default)]
    pub tier: String,
    #[serde(alias = "SYNTROP_ACTION", default)]
    pub action: String,
    #[serde(alias = "SYNTROP_CONFIDENCE", default)]
    pub confidence: serde_json::Value,
    #[serde(alias = "SYNTROP_STATUS", default)]
    pub status: String,
    #[serde(alias = "MESSAGE", default)]
    pub explanation: String,
}

/// Queries structured audit entries from journald and fallback file.
pub async fn handle_audit(
    unit: Option<&str>,
    limit: usize,
    json: bool,
) -> anyhow::Result<()> {
    let mut entries = Vec::new();

    // 1. Try querying journalctl
    let output = Command::new("journalctl")
        .args([
            "SYSLOG_IDENTIFIER=systemd-sentry",
            "-o",
            "json",
            "-n",
            &limit.to_string(),
        ])
        .output()
        .await;

    if let Ok(out) = output {
        if out.status.success() {
            let cursor = std::io::Cursor::new(out.stdout);
            for line in cursor.lines().map_while(Result::ok) {
                if let Ok(e) = serde_json::from_str::<AuditEntry>(&line) {
                    if !e.unit.is_empty() {
                        entries.push(e);
                    }
                }
            }
        }
    }

    // 2. Supplement from /run/syntrop/audit.log if journalctl gave no matches
    if entries.is_empty() {
        let base = std::env::var("RUNTIME_DIRECTORY")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/run/syntrop"));
        let audit_file = base.join("audit.log");
        if audit_file.exists() {
            if let Ok(f) = File::open(&audit_file) {
                let reader = BufReader::new(f);
                for line in reader.lines().map_while(Result::ok) {
                    if let Ok(e) = serde_json::from_str::<AuditEntry>(&line) {
                        entries.push(e);
                    }
                }
            }
        }
    }

    // Filter by unit if requested
    if let Some(target_unit) = unit {
        entries.retain(|e| e.unit == target_unit);
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        if entries.is_empty() {
            println!("No audit trail records found.");
            return Ok(());
        }

        println!(
            "{:<24} {:<20} {:<8} {:<16} {:<16} {:<8}",
            "TIMESTAMP", "UNIT", "TIER", "ACTION", "STATUS", "CONF"
        );
        println!("{}", "-".repeat(96));

        for e in entries.iter().take(limit) {
            let conf_str = match &e.confidence {
                serde_json::Value::Number(n) => format!("{:.3}", n.as_f64().unwrap_or(0.0)),
                serde_json::Value::String(s) => s.clone(),
                _ => "-".into(),
            };
            println!(
                "{:<24} {:<20} {:<8} {:<16} {:<16} {:<8}",
                e.timestamp.chars().take(23).collect::<String>(),
                e.unit,
                e.tier,
                e.action,
                e.status,
                conf_str
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_parsing() {
        let line = r#"{"SYNTROP_UNIT":"test.service","SYNTROP_TIER":"HIGH","SYNTROP_ACTION":"RESTART","SYNTROP_STATUS":"AUTO_REMEDIATED","SYNTROP_CONFIDENCE":"0.950"}"#;
        let entry: AuditEntry = serde_json::from_str(line).unwrap();
        assert_eq!(entry.unit, "test.service");
        assert_eq!(entry.tier, "HIGH");
        assert_eq!(entry.action, "RESTART");
    }
}
