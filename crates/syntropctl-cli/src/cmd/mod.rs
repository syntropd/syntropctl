//! Command implementation modules for syntropctl subcommands.

pub mod completions_cmd;
pub mod devices_cmd;
pub mod drift_cmd;
pub mod embed_cmd;
pub mod explain_cmd;
pub mod generate_cmd;
pub mod info_cmd;
pub mod models_cmd;
pub mod run_cmd;
pub mod status_cmd;

pub use completions_cmd::handle_completions;
pub use devices_cmd::handle_devices;
pub use drift_cmd::handle_drift;
pub use embed_cmd::handle_embed;
pub use explain_cmd::handle_explain;
pub use generate_cmd::handle_generate;
pub use info_cmd::handle_info;
pub use models_cmd::handle_models;
pub use run_cmd::handle_run;
pub use status_cmd::handle_status;
