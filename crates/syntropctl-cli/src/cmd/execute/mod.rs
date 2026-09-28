//! Action command handlers that run tools or invoke neural inference.

pub mod embed_cmd;
pub mod generate_cmd;
pub mod run_cmd;

pub use embed_cmd::handle_embed;
pub use generate_cmd::handle_completions;
pub use generate_cmd::handle_generate;
pub use run_cmd::handle_run;
