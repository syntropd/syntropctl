//! Unit tests for autonomous administration domain types and behaviors.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use syntropctl_core::admin::{
        AdminAuditEntry, AdminStatusReport, CircuitBreakerInfo, LockoutResetOutcome, RecipeStep,
        RemediationOutcome, RemediationRecipe, RemediationRecipeKind, RollbackOutcome,
    };

    #[test]
    fn test_admin_status_report_creation() {
        let mut breakers = HashMap::new();
        breakers.insert(
            "nginx.service".to_string(),
            CircuitBreakerInfo {
                unit: "nginx.service".into(),
                state: "CLOSED".into(),
                cooldown_remaining_secs: 0,
                recent_failures: 0,
                permanently_locked: false,
                allows_remediation: true,
            },
        );

        let report = AdminStatusReport {
            healing_enabled: true,
            sentry_online: true,
            toold_online: true,
            contextd_online: true,
            routerd_online: true,
            circuit_breakers: breakers,
            locked_out_units: vec![],
        };

        assert!(report.healing_enabled);
        assert_eq!(report.circuit_breakers.len(), 1);
        assert!(report.locked_out_units.is_empty());
    }

    #[test]
    fn test_remediation_recipes_validation() {
        let restart = RemediationRecipe::restart();
        assert_eq!(restart.kind, RemediationRecipeKind::Restart);
        assert!(restart.steps.contains(&RecipeStep::VerifySyntax));

        let rollback = RemediationRecipe::config_rollback();
        assert_eq!(rollback.kind, RemediationRecipeKind::ConfigRollback);
        assert!(rollback.steps.contains(&RecipeStep::RollbackDrift));

        let daemon_reload = RemediationRecipe::daemon_reload();
        assert_eq!(daemon_reload.kind, RemediationRecipeKind::DaemonReload);

        let auto_novel = RemediationRecipe::auto("Unknown", false);
        assert!(auto_novel.steps.contains(&RecipeStep::QueryLlmFallback));
    }

    #[test]
    fn test_admin_outcomes_serialization() {
        let rem = RemediationOutcome {
            unit: "app.service".into(),
            recipe_name: "restart".into(),
            dry_run: false,
            allowed_by_circuit_breaker: true,
            fault_classification: "Crash".into(),
            drift_detected: false,
            executed_steps: vec!["Step 1".into()],
            llm_diagnosis: None,
            rollback_id: Some("rb-1".into()),
            success: true,
            message: "Fixed".into(),
        };
        let rem_json = serde_json::to_string(&rem).unwrap();
        assert!(rem_json.contains("app.service"));

        let rb = RollbackOutcome {
            unit: "app.service".into(),
            rollback_id: "rb-1".into(),
            target_path: Some("/etc/app.conf".into()),
            summary: "Restored".into(),
            success: true,
        };
        let rb_json = serde_json::to_string(&rb).unwrap();
        assert!(rb_json.contains("rb-1"));

        let lo = LockoutResetOutcome {
            unit: "app.service".into(),
            success: true,
            message: "Reset OK".into(),
        };
        let lo_json = serde_json::to_string(&lo).unwrap();
        assert!(lo_json.contains("Reset OK"));

        let audit = AdminAuditEntry {
            incident_id: "inc-1".into(),
            timestamp: "123".into(),
            unit: "app.service".into(),
            action: "restart".into(),
            result: "success".into(),
            message: "Done".into(),
        };
        let audit_json = serde_json::to_string(&audit).unwrap();
        assert!(audit_json.contains("inc-1"));
    }
}
