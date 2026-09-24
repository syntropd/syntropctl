//! Formatting utilities for CLI output.

pub mod json;
pub mod table;

pub use json::print_json;
pub use table::{
    print_devices_table, print_drift_table, print_incident_report, print_models_table,
    print_status_table,
};
