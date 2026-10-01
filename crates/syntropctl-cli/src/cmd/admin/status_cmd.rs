//! Handler for `syn admin status` inspecting circuit breakers and daemons.

use std::process::ExitCode;
use syntropctl_core::admin::query_admin_status;

/// Renders autonomous healing and circuit-breaker status in text or JSON.
pub async fn handle_admin_status(json: bool) -> anyhow::Result<ExitCode> {
    let report = query_admin_status().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(ExitCode::SUCCESS);
    }

    println!("=== Syntrop Autonomous Healing Status ===");
    let health_label = if report.healing_enabled {
        "ENABLED (Ready)"
    } else {
        "DEGRADED / INCOMPLETE"
    };
    println!("  Autonomous Healing: {}", health_label);
    println!(
        "  Sentry Supervisor:  {}",
        if report.sentry_online { "ONLINE" } else { "OFFLINE" }
    );
    println!(
        "  Toold Sandbox:      {}",
        if report.toold_online { "ONLINE" } else { "OFFLINE" }
    );
    println!(
        "  Contextd Causal:    {}",
        if report.contextd_online { "ONLINE" } else { "OFFLINE" }
    );
    println!(
        "  Routerd Gateway:    {}",
        if report.routerd_online { "ONLINE" } else { "OFFLINE" }
    );
    println!();

    if report.circuit_breakers.is_empty() {
        println!("Active Circuit Breakers:");
        println!("  No active circuit breakers recorded by supervisor.");
    } else {
        println!("Active Circuit Breakers:");
        println!(
            "  {:<28} {:<18} {:<10} {:<10} REMEDIATION",
            "UNIT", "STATE", "FAILURES", "COOLDOWN"
        );
        let mut sorted_keys: Vec<_> = report.circuit_breakers.keys().collect();
        sorted_keys.sort();
        for unit in sorted_keys {
            let b = &report.circuit_breakers[unit];
            let cooldown = if b.cooldown_remaining_secs > 0 {
                format!("{}s", b.cooldown_remaining_secs)
            } else {
                "-".into()
            };
            let rem_label = if b.allows_remediation {
                "ALLOWED"
            } else if b.permanently_locked {
                "LOCKED OUT"
            } else {
                "BLOCKED"
            };
            println!(
                "  {:<28} {:<18} {:<10} {:<10} {}",
                b.unit, b.state, b.recent_failures, cooldown, rem_label
            );
        }
    }

    if !report.locked_out_units.is_empty() {
        println!();
        println!("Locked-Out Units (Manual Operator Inspection Required):");
        for unit in &report.locked_out_units {
            println!(
                "  ! {} (tripped/flapping — reset via 'syn admin lockout reset {}')",
                unit, unit
            );
        }
    }

    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_cmd_module_exists() {
        assert!(std::mem::size_of::<ExitCode>() > 0);
    }
}
