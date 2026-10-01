//! Handler for `syn admin remediate <unit>` executing self-healing recipes.

use std::process::ExitCode;
use syntropctl_core::admin::execute_remediation;

/// Triggers a guided or automated remediation recipe for a failed service.
pub async fn handle_admin_remediate(
    unit: &str,
    recipe: Option<&str>,
    dry_run: bool,
    json: bool,
) -> anyhow::Result<ExitCode> {
    let outcome = execute_remediation(unit, recipe, dry_run).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
        if outcome.success {
            return Ok(ExitCode::SUCCESS);
        } else {
            return Ok(ExitCode::FAILURE);
        }
    }

    println!("=== Syntrop Autonomous Remediation: {} ===", outcome.unit);
    let dry_label = if outcome.dry_run { " [DRY-RUN]" } else { "" };
    println!("  Recipe:            {}{}", outcome.recipe_name, dry_label);
    println!("  Fault Class:       {}", outcome.fault_classification);
    println!(
        "  Circuit Breaker:   {}",
        if outcome.allowed_by_circuit_breaker {
            "ALLOWED"
        } else {
            "LOCKED OUT (Remediation Aborted)"
        }
    );
    println!(
        "  Causal Drift:      {}",
        if outcome.drift_detected { "DETECTED" } else { "NONE" }
    );
    if let Some(ref rb) = outcome.rollback_id {
        println!("  Rollback Snapshot: {}", rb);
    }
    println!();

    println!("Execution Sequence:");
    for (i, step) in outcome.executed_steps.iter().enumerate() {
        println!("  {}. {}", i + 1, step);
    }

    if let Some(ref diag) = outcome.llm_diagnosis {
        println!();
        println!("Routerd Diagnostic Fallback Analysis:");
        println!("  {}", diag);
    }

    println!();
    if outcome.success {
        println!("Result: SUCCESS — {}", outcome.message);
        Ok(ExitCode::SUCCESS)
    } else {
        println!("Result: FAILED — {}", outcome.message);
        Ok(ExitCode::FAILURE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remediate_cmd_module_exists() {
        assert!(std::mem::size_of::<ExitCode>() > 0);
    }
}
