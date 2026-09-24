//! Unit tests for formatters and table outputs.

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use syntropctl_cli::format::{
        print_devices_table, print_drift_table, print_incident_report, print_json,
        print_models_table, print_status_table,
    };
    use syntropctl_core::ops::{
        DaemonStatus, DeviceReport, DriftEvent, IncidentReport, ModelEntry,
    };

    #[test]
    fn test_print_tables_do_not_panic_when_empty() {
        print_status_table(&[]);
        print_models_table(&[]);
        print_devices_table(&[]);
        print_drift_table(&[]);
    }

    #[test]
    fn test_print_tables_with_mock_data() {
        let statuses = vec![DaemonStatus {
            name: "runtimed".to_string(),
            unit_name: "runtimed.service".to_string(),
            socket_path: PathBuf::from("/run/syntrop/io.syntrop.Runtime1"),
            socket_exists: true,
            responsive: true,
            latency_ms: Some(2),
            product: Some("runtimed".to_string()),
            version: Some("0.1.0".to_string()),
            interfaces: vec!["io.syntrop.Runtime1".to_string()],
            error_message: None,
        }];
        print_status_table(&statuses);

        let models = vec![ModelEntry {
            name: "qwen2.5-coder-7b".to_string(),
            status: "loaded".to_string(),
            size_bytes: 4_294_967_296,
            parameter_count: Some(7_000_000_000),
            backend: Some("cpu-avx2".to_string()),
            context_window: Some(8192),
        }];
        print_models_table(&models);

        let devices = vec![DeviceReport {
            id: "npu0".to_string(),
            device_type: "npu".to_string(),
            vendor: "Intel".to_string(),
            model: "NPU 3720".to_string(),
            memory_total_bytes: 8_589_934_592,
            memory_used_bytes: 1_073_741_824,
            psi_pressure: 0.1,
            status: "ready".to_string(),
        }];
        print_devices_table(&devices);

        let drift = vec![DriftEvent {
            path: "/etc/resolv.conf".to_string(),
            change_type: "updated".to_string(),
            timestamp: 1727145100,
            details: "nameserver updated by systemd-resolved".to_string(),
        }];
        print_drift_table(&drift);
    }

    #[test]
    fn test_print_incident_report() {
        let report = IncidentReport {
            unit: "postgresql.service".to_string(),
            status: "activating (auto-restart)".to_string(),
            root_cause: "Disk space exhausted on /var/lib/pgsql".to_string(),
            confidence: 0.98,
            recommended_action: "Free disk space or resize filesystem".to_string(),
            remediation_command: Some("journalctl --vacuum-size=500M".to_string()),
            journal_slice: vec!["FATAL: could not write to file: No space left on device".to_string()],
            recent_drift: vec!["Disk usage exceeded 99%".to_string()],
        };

        print_incident_report(&report);
    }

    #[test]
    fn test_print_json() {
        let obj = serde_json::json!({ "status": "ok", "count": 42 });
        print_json(&obj);
    }
}
