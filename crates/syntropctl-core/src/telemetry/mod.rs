//! Kernel telemetry, PSI readers, and dynamic closed-loop tuning governor.

pub mod client;
pub mod policy;
pub mod types;

pub use client::query_telemetry_status;
pub use policy::{apply_tuning_policy, load_active_tuning_config, tuning_config_paths};
pub use types::{
    PressureMetrics, TelemetryStatusReport, TuningConfig, TuningOutcome, TuningPolicy,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_module_reexports() {
        let default_policy = TuningPolicy::default();
        assert_eq!(default_policy, TuningPolicy::Balanced);
    }
}
