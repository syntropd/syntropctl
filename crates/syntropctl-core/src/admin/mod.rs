//! Autonomous OS self-healing and administration subsystem.

pub mod journal;
pub mod lockout;
pub mod recipe;
pub mod remediate;
pub mod rollback;
pub mod status;

pub use journal::{log_admin_audit, query_admin_audit, AdminAuditEntry};
pub use lockout::{reset_lockout, LockoutResetOutcome};
pub use recipe::{RecipeStep, RemediationRecipe, RemediationRecipeKind};
pub use remediate::{execute_remediation, RemediationOutcome};
pub use rollback::{execute_rollback, RollbackOutcome};
pub use status::{query_admin_status, AdminStatusReport, CircuitBreakerInfo};
