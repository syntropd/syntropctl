//! High-level administration and diagnostic operations.

pub mod devices;
pub mod drift;
pub mod explain;
pub mod inference;
pub mod models;
pub mod run;
pub mod status;

pub use devices::{query_devices, DeviceReport};
pub use drift::{query_drift, DriftEvent};
pub use explain::{explain_unit, IncidentReport};
pub use inference::{embed_text, generate_text, GenerationOutput};
pub use models::{query_models, ModelEntry};
pub use run::{execute_sandboxed_tool, ToolRunResult};
pub use status::{check_daemon_status, collect_fleet_status, DaemonStatus};
