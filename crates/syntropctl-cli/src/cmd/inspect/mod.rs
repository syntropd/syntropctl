//! Read-only fleet inspection and diagnosis command handlers.

pub mod devices_cmd;
pub mod drift_cmd;
pub mod explain_cmd;
pub mod info_cmd;
pub mod models_cmd;
pub mod status_cmd;

pub use devices_cmd::handle_devices;
pub use drift_cmd::handle_drift;
pub use explain_cmd::handle_explain;
pub use info_cmd::handle_info;
pub use models_cmd::handle_models;
pub use status_cmd::handle_status;
