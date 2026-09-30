//! System One operator decision, prompt, and audit commands.
//!
//! Submodules:
//! - `audit_cmd`: Queries structured journald audit logs.
//! - `decide_cmd`: Interactive single-keystroke approval TUI.
//! - `prompt_cmd`: Sub-millisecond shell prompt status indicator.

pub mod audit_cmd;
pub mod decide_cmd;
pub mod prompt_cmd;

pub use audit_cmd::handle_audit;
pub use audit_cmd::AuditEntry;
pub use decide_cmd::handle_decide;
pub use decide_cmd::PendingIncident;
pub use prompt_cmd::handle_prompt;
