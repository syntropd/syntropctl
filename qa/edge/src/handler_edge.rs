//! Edge tests for command handler failure paths with unreachable daemons.

#[cfg(test)]
mod tests {
    use syntropctl_cli::cmd::*;
    use syntropctl_core::ops::query_models;

    /// Pin every daemon socket override to a path that cannot exist.
    ///
    /// Overrides are deliberately never removed: parallel tests in this
    /// binary only ever observe missing sockets, so every interleaving
    /// stays deterministic even on hosts with live daemons.
    fn pin_all_sockets_missing() {
        for var in [
            "SYNTROP_SENTRY_SOCKET",
            "SYNTROP_INFERENCE_SOCKET",
            "SYNTROP_MODELD_SOCKET",
            "SYNTROP_CONTEXTD_SOCKET",
            "SYNTROP_TOOLD_SOCKET",
            "SYNTROP_RUNTIMED_SOCKET",
            "SYNTROP_ROUTER_SOCKET",
        ] {
            let path = std::env::temp_dir().join("syntropctl-qa-no-such-socket.sock");
            let _ = std::fs::remove_file(&path);
            assert!(!path.exists());
            std::env::set_var(var, &path);
        }
    }

    #[tokio::test]
    async fn test_devices_fails_without_inferenced() {
        pin_all_sockets_missing();
        let err = handle_devices(false).await.unwrap_err();
        assert!(err.to_string().contains("inferenced"), "{err}");
        assert!(handle_devices(true).await.is_err());
    }

    #[tokio::test]
    async fn test_drift_fails_without_contextd() {
        pin_all_sockets_missing();
        let err = handle_drift(None, false).await.unwrap_err();
        assert!(err.to_string().contains("contextd"), "{err}");
        assert!(handle_drift(Some("sentry.service"), true).await.is_err());
    }

    #[tokio::test]
    async fn test_run_fails_without_toold() {
        pin_all_sockets_missing();
        let err = handle_run("uname", &[], None, false).await.unwrap_err();
        assert!(err.to_string().contains("toold"), "{err}");
        assert!(handle_run("uname", &["-a".to_string()], Some("strict"), true).await.is_err());
    }

    #[tokio::test]
    async fn test_generate_fails_without_runtimed() {
        pin_all_sockets_missing();
        let err = handle_generate("hello", "tiny", 8, 0.0, false).await.unwrap_err();
        assert!(err.to_string().contains("runtimed"), "{err}");
        assert!(handle_generate("hello", "tiny", 8, 0.0, true).await.is_err());
    }

    #[tokio::test]
    async fn test_embed_fails_without_runtimed() {
        pin_all_sockets_missing();
        let err = handle_embed("hello", "tiny", false).await.unwrap_err();
        assert!(err.to_string().contains("runtimed"), "{err}");
        assert!(handle_embed("hello", "tiny", true).await.is_err());
    }

    #[tokio::test]
    async fn test_explain_without_daemons_uses_defaults() {
        pin_all_sockets_missing();
        assert!(handle_explain("caddy.service", false).await.is_ok());
        assert!(handle_explain("caddy.service", true).await.is_ok());
        let report = syntropctl_core::ops::explain_unit("caddy.service").await.unwrap();
        assert_eq!(report.unit, "caddy.service");
        assert_eq!(report.root_cause, "No active failure record found in sentry.");
        assert!(report.recent_drift.is_empty());
        assert!(report.journal_slice.is_empty());
    }

    #[tokio::test]
    async fn test_models_without_daemons_is_empty() {
        pin_all_sockets_missing();
        assert!(query_models().await.unwrap().is_empty());
        assert!(handle_models(false).await.is_ok());
        assert!(handle_models(true).await.is_ok());
    }

    #[tokio::test]
    async fn test_status_without_daemons_reports_inactive() {
        pin_all_sockets_missing();
        assert!(handle_status(None, false).await.is_ok());
        assert!(handle_status(None, true).await.is_ok());
        assert!(handle_status(Some("runtimed".to_string()), true).await.is_ok());
        let err = handle_status(Some("bogus-daemon".to_string()), false).await.unwrap_err();
        assert!(err.to_string().contains("Unknown daemon"), "{err}");
    }

    #[tokio::test]
    async fn test_info_without_daemons() {
        pin_all_sockets_missing();
        assert!(handle_info(None, false).await.is_ok());
        assert!(handle_info(None, true).await.is_ok());
        let unknown = handle_info(Some("bogus-daemon".to_string()), false).await.unwrap_err();
        assert!(unknown.to_string().contains("Unknown daemon"), "{unknown}");
        let missing = handle_info(Some("runtimed".to_string()), true).await.unwrap_err();
        assert!(missing.to_string().contains("socket not found"), "{missing}");
    }
}
