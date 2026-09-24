//! Unit tests for operations data types and default behaviors.

#[cfg(test)]
mod tests {
    use syntropctl_core::daemon::DaemonEndpoint;
    use syntropctl_core::ops::{
        check_daemon_status, DeviceReport, DriftEvent, GenerationOutput, IncidentReport,
        ModelEntry, ToolRunResult,
    };

    #[tokio::test]
    async fn test_check_daemon_status_missing_socket() {
        let ep = DaemonEndpoint::from_name("sentry").unwrap();
        // Point to a non-existent socket
        std::env::set_var("SYNTROP_SENTRY_SOCKET", "/tmp/definitely_not_a_real_socket.sock");
        let status = check_daemon_status(ep).await;

        assert_eq!(status.name, "sentry");
        assert_eq!(status.unit_name, "sentry.service");
        assert!(!status.socket_exists);
        assert!(!status.responsive);
        assert!(status.error_message.is_some());
        assert!(status.latency_ms.is_none());

        std::env::remove_var("SYNTROP_SENTRY_SOCKET");
    }

    #[test]
    fn test_incident_report_creation() {
        let report = IncidentReport {
            unit: "caddy.service".to_string(),
            status: "failed".to_string(),
            root_cause: "Port 443 already bound by nginx".to_string(),
            confidence: 0.95,
            recommended_action: "Stop conflicting nginx service".to_string(),
            remediation_command: Some("systemctl stop nginx".to_string()),
            journal_slice: vec!["bind: address already in use".to_string()],
            recent_drift: vec!["/etc/caddy/Caddyfile modified".to_string()],
        };

        assert_eq!(report.unit, "caddy.service");
        assert_eq!(report.status, "failed");
        assert!(report.confidence > 0.9);
        assert_eq!(report.journal_slice.len(), 1);
    }

    #[test]
    fn test_model_entry_and_device_report_serialization() {
        let model = ModelEntry {
            name: "qwen2.5-coder-7b".to_string(),
            status: "loaded".to_string(),
            size_bytes: 4_294_967_296,
            parameter_count: Some(7_000_000_000),
            backend: Some("cpu-avx2".to_string()),
            context_window: Some(8192),
        };
        let m_json = serde_json::to_string(&model).unwrap();
        assert!(m_json.contains("qwen2.5-coder-7b"));

        let device = DeviceReport {
            id: "gpu0".to_string(),
            device_type: "gpu".to_string(),
            vendor: "NVIDIA".to_string(),
            model: "RTX 4090".to_string(),
            memory_total_bytes: 25_769_803_776,
            memory_used_bytes: 4_294_967_296,
            psi_pressure: 0.0,
            status: "active".to_string(),
        };
        let d_json = serde_json::to_string(&device).unwrap();
        assert!(d_json.contains("RTX 4090"));
    }

    #[test]
    fn test_drift_and_tool_run_results() {
        let drift = DriftEvent {
            path: "/etc/systemd/system/sentry.service.d/override.conf".to_string(),
            change_type: "created".to_string(),
            timestamp: 1727145000,
            details: "Added MemoryMax=2G drop-in".to_string(),
        };
        assert_eq!(drift.change_type, "created");

        let tool_run = ToolRunResult {
            exit_code: 0,
            stdout: "Linux 6.12.0".to_string(),
            stderr: String::new(),
            duration_ms: 12,
        };
        assert_eq!(tool_run.exit_code, 0);

        let gen_output = GenerationOutput {
            text: "Root cause found".to_string(),
            prompt_tokens: 10,
            completion_tokens: 3,
            finish_reason: "stop".to_string(),
            duration_ms: 45,
        };
        assert_eq!(gen_output.finish_reason, "stop");
    }
}
