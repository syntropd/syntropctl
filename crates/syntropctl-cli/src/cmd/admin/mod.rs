//! Autonomous OS self-healing and administration CLI command suite.

pub mod audit_cmd;
pub mod lockout_cmd;
pub mod remediate_cmd;
pub mod rollback_cmd;
pub mod status_cmd;

pub use audit_cmd::handle_admin_audit;
pub use lockout_cmd::{handle_admin_lockout, LockoutCommands};
pub use remediate_cmd::handle_admin_remediate;
pub use rollback_cmd::handle_admin_rollback;
pub use status_cmd::handle_admin_status;

use clap::Subcommand;
use std::process::ExitCode;

/// Autonomous administration and self-healing subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum AdminCommands {
    /// Display overall autonomous healing status, active circuit-breaker states, and locked-out units.
    Status,

    /// Trigger a guided or automated remediation recipe for a failed service.
    Remediate {
        /// Name of the target systemd unit (e.g. nginx.service).
        unit: String,

        /// Optional specific remediation recipe name (restart, config-rollback, daemon-reload, auto).
        #[arg(long)]
        recipe: Option<String>,

        /// Preview remediation steps without applying mutations.
        #[arg(long)]
        dry_run: bool,
    },

    /// Roll back state or configuration modifications executed during remediation.
    Rollback {
        /// Name of the target systemd unit.
        unit: String,

        /// Optional snapshot identifier to restore.
        #[arg(long)]
        snapshot: Option<String>,
    },

    /// Inspect immutable forensic records of all self-healing actions.
    Audit {
        /// Filter audit records by service unit name.
        #[arg(short = 'u', long = "unit")]
        unit: Option<String>,

        /// Maximum number of audit records to return.
        #[arg(short = 'n', long = "limit", default_value = "50")]
        limit: usize,
    },

    /// Inspect or reset circuit-breaker lockout states.
    Lockout {
        #[command(subcommand)]
        command: LockoutCommands,
    },
}

/// Dispatches all `syn admin` / `syntropctl admin` subcommands.
pub async fn handle_admin(command: AdminCommands, json: bool) -> anyhow::Result<ExitCode> {
    match command {
        AdminCommands::Status => handle_admin_status(json).await,
        AdminCommands::Remediate {
            unit,
            recipe,
            dry_run,
        } => handle_admin_remediate(&unit, recipe.as_deref(), dry_run, json).await,
        AdminCommands::Rollback { unit, snapshot } => {
            handle_admin_rollback(&unit, snapshot.as_deref(), json).await
        }
        AdminCommands::Audit { unit, limit } => {
            handle_admin_audit(unit.as_deref(), limit, json).await
        }
        AdminCommands::Lockout { command } => handle_admin_lockout(command, json).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_commands_dispatch() {
        let cmd = AdminCommands::Status;
        assert!(matches!(cmd, AdminCommands::Status));
    }
}
