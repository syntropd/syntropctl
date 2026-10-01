//! Handler for `syn admin audit` displaying forensic self-healing records.

use std::process::ExitCode;
use syntropctl_core::admin::query_admin_audit;

fn format_audit_timestamp(raw: &str) -> String {
    if raw.is_empty() {
        return "-".into();
    }
    if raw.contains('T') {
        let clean = raw.split('.').next().unwrap_or(raw);
        return clean.replace('T', " ").trim_end_matches('Z').to_string();
    }
    if let Ok(mut num) = raw.parse::<u64>() {
        if num > 1_000_000_000_000 {
            num /= 1_000_000;
        }
        let secs = num % 86400;
        let mut days = num / 86400;
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        days += 719468;
        let era = days / 146097;
        let doe = days % 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let mut y = (yoe as i64) + (era as i64) * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m_month = if mp < 10 { mp + 3 } else { mp - 9 };
        if m_month <= 2 {
            y += 1;
        }
        return format!("{y:04}-{m_month:02}-{d:02} {h:02}:{m:02}:{s:02}");
    }
    raw.chars().take(19).collect()
}

/// Inspect immutable forensic records of all self-healing actions.
pub async fn handle_admin_audit(
    unit: Option<&str>,
    limit: usize,
    json: bool,
) -> anyhow::Result<ExitCode> {
    let entries = query_admin_audit(unit, limit).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(ExitCode::SUCCESS);
    }

    if entries.is_empty() {
        println!("No self-healing audit records found.");
        return Ok(ExitCode::SUCCESS);
    }

    println!(
        "{:<19}  {:<24}  {:<16}  {:<10}  {:<14}  MESSAGE",
        "TIMESTAMP", "UNIT", "ACTION", "RESULT", "INCIDENT ID"
    );
    for e in entries {
        let ts = format_audit_timestamp(&e.timestamp);
        let msg_short: String = e.message.chars().take(40).collect();
        println!(
            "{:<19}  {:<24}  {:<16}  {:<10}  {:<14}  {}",
            ts, e.unit, e.action, e.result, e.incident_id, msg_short
        );
    }

    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_audit_timestamp_numeric() {
        let formatted = format_audit_timestamp("1700000000000000");
        assert_eq!(formatted.len(), 19);
    }

    #[test]
    fn test_format_audit_timestamp_iso() {
        let formatted = format_audit_timestamp("2026-10-01T12:00:00Z");
        assert_eq!(formatted, "2026-10-01 12:00:00");
    }
}
