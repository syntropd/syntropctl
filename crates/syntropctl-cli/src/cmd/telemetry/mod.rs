//! Kernel telemetry and closed-loop tuning CLI command suite.

pub mod status_cmd;
pub mod tune_cmd;

pub use status_cmd::handle_telemetry_status;
pub use tune_cmd::handle_telemetry_tune;

use clap::Subcommand;
use std::process::ExitCode;

/// Closed-loop kernel telemetry and tuning subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum TelemetryCommands {
    /// Inspect non-blocking kernel PSI metrics and eBPF scheduler latency.
    Status,

    /// Query or configure dynamic closed-loop PSI tuning policy and draft horizons.
    Tune {
        /// Tuning policy preset (conservative, balanced, aggressive).
        #[arg(short = 'p', long = "policy")]
        policy: Option<String>,
    },
}

/// Dispatches all `syntropctl telemetry` subcommands.
pub async fn handle_telemetry(command: TelemetryCommands, json: bool) -> anyhow::Result<ExitCode> {
    match command {
        TelemetryCommands::Status => handle_telemetry_status(json).await,
        TelemetryCommands::Tune { policy } => {
            handle_telemetry_tune(policy.as_deref(), json).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_commands_variant() {
        let cmd = TelemetryCommands::Status;
        assert!(matches!(cmd, TelemetryCommands::Status));
    }
}
