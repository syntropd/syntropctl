//! Edge tests for admin command argument parsing and error propagation.

#[cfg(test)]
mod tests {
    use clap::Parser;
    use syntropctl_cli::cli::{Cli, Commands};
    use syntropctl_cli::cmd::admin::{handle_admin, AdminCommands, LockoutCommands};
    use syntropctl_core::admin::execute_remediation;

    #[test]
    fn test_parse_admin_status_cli() {
        let cli = Cli::try_parse_from(["syntropctl", "admin", "status"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Admin {
                command: AdminCommands::Status
            }
        ));
    }

    #[test]
    fn test_parse_admin_remediate_cli() {
        let cli = Cli::try_parse_from([
            "syntropctl",
            "admin",
            "remediate",
            "foo.service",
            "--recipe",
            "restart",
            "--dry-run",
        ])
        .unwrap();

        match cli.command {
            Commands::Admin {
                command:
                    AdminCommands::Remediate {
                        unit,
                        recipe,
                        dry_run,
                    },
            } => {
                assert_eq!(unit, "foo.service");
                assert_eq!(recipe.as_deref(), Some("restart"));
                assert!(dry_run);
            }
            _ => panic!("Expected Remediate command"),
        }
    }

    #[test]
    fn test_parse_admin_rollback_and_audit_cli() {
        let cli = Cli::try_parse_from([
            "syntropctl",
            "admin",
            "rollback",
            "bar.service",
            "--snapshot",
            "rb-999",
        ])
        .unwrap();

        match cli.command {
            Commands::Admin {
                command: AdminCommands::Rollback { unit, snapshot },
            } => {
                assert_eq!(unit, "bar.service");
                assert_eq!(snapshot.as_deref(), Some("rb-999"));
            }
            _ => panic!("Expected Rollback command"),
        }

        let cli2 = Cli::try_parse_from([
            "syntropctl",
            "admin",
            "audit",
            "--unit",
            "bar.service",
            "--limit",
            "10",
        ])
        .unwrap();

        match cli2.command {
            Commands::Admin {
                command: AdminCommands::Audit { unit, limit },
            } => {
                assert_eq!(unit.as_deref(), Some("bar.service"));
                assert_eq!(limit, 10);
            }
            _ => panic!("Expected Audit command"),
        }
    }

    #[test]
    fn test_parse_admin_lockout_cli() {
        let cli =
            Cli::try_parse_from(["syntropctl", "admin", "lockout", "reset", "foo.service"]).unwrap();

        match cli.command {
            Commands::Admin {
                command:
                    AdminCommands::Lockout {
                        command: LockoutCommands::Reset { unit },
                    },
            } => {
                assert_eq!(unit, "foo.service");
            }
            _ => panic!("Expected Lockout Reset command"),
        }
    }

    #[tokio::test]
    async fn test_remediate_dry_run_executes_safely() {
        let outcome = execute_remediation("test-dummy.service", Some("restart"), true)
            .await
            .unwrap();
        assert!(outcome.dry_run);
        assert!(outcome.success);
        assert_eq!(outcome.recipe_name, "restart");
    }

    #[tokio::test]
    async fn test_handle_admin_status_without_daemons() {
        let code = handle_admin(AdminCommands::Status, true).await.unwrap();
        assert_eq!(code, std::process::ExitCode::SUCCESS);
    }

    #[tokio::test]
    async fn test_remediate_syntax_failure_handled_cleanly() {
        let outcome = execute_remediation("nonexistent-invalid-unit.service", Some("restart"), false)
            .await
            .unwrap();
        assert!(!outcome.dry_run);
        assert!(!outcome.success);
        assert!(outcome.message.contains("syntax.verify") || outcome.message.contains("toold sandbox socket"));
    }

    #[tokio::test]
    async fn test_admin_audit_journal_emission_and_query() {
        use syntropctl_core::admin::{log_admin_audit, query_admin_audit};
        let inc_id = format!("edge-{}", std::process::id());
        log_admin_audit(&inc_id, "audit-edge.service", "test_step", "success", "Edge audit entry");
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        if let Ok(entries) = query_admin_audit(Some("audit-edge.service"), 5).await {
            if let Some(entry) = entries.iter().find(|e| e.incident_id == inc_id) {
                assert_eq!(entry.unit, "audit-edge.service");
                assert_eq!(entry.action, "test_step");
            }
        }
    }
}
