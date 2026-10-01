//! Handler for `syn admin lockout reset <unit>` clearing circuit breakers.

use std::process::ExitCode;
use syntropctl_core::admin::reset_lockout;

/// Subcommands for lockout management.
#[derive(clap::Subcommand, Debug, Clone)]
pub enum LockoutCommands {
    /// Manually reset circuit-breaker lockout after operator inspection.
    Reset {
        /// Name of the target systemd unit (e.g. nginx.service).
        unit: String,
    },
}

/// Dispatches lockout commands.
pub async fn handle_admin_lockout(
    command: LockoutCommands,
    json: bool,
) -> anyhow::Result<ExitCode> {
    match command {
        LockoutCommands::Reset { unit } => {
            let outcome = reset_lockout(&unit).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&outcome)?);
            } else {
                println!("{}", outcome.message);
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockout_commands_enum() {
        let cmd = LockoutCommands::Reset {
            unit: "test.service".into(),
        };
        match cmd {
            LockoutCommands::Reset { unit } => assert_eq!(unit, "test.service"),
        }
    }
}
