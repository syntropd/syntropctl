//! Handler for `syn admin rollback <unit>` restoring state from snapshots.

use std::process::ExitCode;
use syntropctl_core::admin::execute_rollback;

/// Roll back state or configuration modifications executed during remediation.
pub async fn handle_admin_rollback(
    unit: &str,
    snapshot: Option<&str>,
    json: bool,
) -> anyhow::Result<ExitCode> {
    let outcome = execute_rollback(unit, snapshot).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
        return Ok(ExitCode::SUCCESS);
    }

    println!("=== Syntrop Configuration Rollback ===");
    println!("  Target Unit:   {}", outcome.unit);
    println!("  Snapshot ID:   {}", outcome.rollback_id);
    if let Some(ref path) = outcome.target_path {
        println!("  Restored Path: {}", path);
    }
    println!("  Summary:       {}", outcome.summary);
    println!("  Result:        SUCCESS");

    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollback_cmd_module_exists() {
        assert!(std::mem::size_of::<ExitCode>() > 0);
    }
}
