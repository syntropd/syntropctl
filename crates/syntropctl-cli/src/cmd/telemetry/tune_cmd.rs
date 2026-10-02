//! Handler for `syntropctl telemetry tune` configuring closed-loop tuning policies.

use std::process::ExitCode;
use syntropctl_core::telemetry::{
    apply_tuning_policy, load_active_tuning_config, tuning_config_paths, TuningOutcome,
    TuningPolicy,
};

/// Configures dynamic tuning policy or displays current configuration.
pub async fn handle_telemetry_tune(
    policy_str: Option<&str>,
    json: bool,
) -> anyhow::Result<ExitCode> {
    let outcome = match policy_str {
        Some(p) => {
            let policy = TuningPolicy::parse_str(p).ok_or_else(|| {
                anyhow::anyhow!(
                    "unknown tuning policy '{}'; valid choices: conservative, balanced, aggressive",
                    p
                )
            })?;
            apply_tuning_policy(policy)?
        }
        None => {
            let config = load_active_tuning_config();
            let path = tuning_config_paths()
                .into_iter()
                .find(|p| p.exists())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "default".to_string());
            TuningOutcome {
                applied: false,
                path,
                config,
            }
        }
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
        return Ok(ExitCode::SUCCESS);
    }

    println!("=== Syntrop Dynamic Closed-Loop Tuning Configuration ===");
    println!("  Active Policy:        {}", outcome.config.policy.as_str());
    println!(
        "  Memory Some Spike:    > {:.1}%",
        outcome.config.memory_some_threshold
    );
    println!(
        "  Memory Full Spike:    > {:.1}%",
        outcome.config.memory_full_threshold
    );
    println!(
        "  Speculative Draft K:  {} (clamped to 1 under memory spike)",
        outcome.config.k_draft_horizon
    );
    println!(
        "  Max Tokens Clamp:     {} tokens",
        outcome.config.max_tokens_clamp
    );
    println!(
        "  Cooperative Deadline: {} ms (SIGUSR1 runqueue yield)",
        outcome.config.cooperative_yield_deadline_ms
    );
    let status_str = if outcome.applied {
        format!("Applied & Active (saved to {})", outcome.path)
    } else {
        format!("Current Configuration (from {})", outcome.path)
    };
    println!("  Status:               {}", status_str);

    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_telemetry_tune_query_json() {
        let code = handle_telemetry_tune(None, true)
            .await
            .expect("query tune succeeds");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[tokio::test]
    async fn test_handle_telemetry_tune_invalid_policy() {
        let res = handle_telemetry_tune(Some("invalid"), false).await;
        assert!(res.is_err());
    }
}
