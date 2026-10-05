//! Edge tests for CLI argument parsing.

#[cfg(test)]
mod tests {
    use clap::Parser;
    use syntropctl_cli::cli::{Cli, Commands};

    #[test]
    fn test_parse_status_without_daemon() {
        let args = vec!["syntropctl", "status"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Status { daemon } => assert!(daemon.is_none()),
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    fn test_parse_status_with_daemon() {
        let args = vec!["syntropctl", "status", "runtimed"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Status { daemon } => assert_eq!(daemon.unwrap(), "runtimed"),
            _ => panic!("Expected Status command with daemon"),
        }
    }

    #[test]
    fn test_parse_run_with_trailing_flags() {
        let args = vec!["syntropctl", "run", "ls", "-la", "/tmp"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Run { tool, args, .. } => {
                assert_eq!(tool, "ls");
                assert_eq!(args, vec!["-la", "/tmp"]);
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_parse_global_json_flag() {
        let args = vec!["syntropctl", "--json", "models"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.json);
        match cli.command {
            Commands::Models => (),
            _ => panic!("Expected Models command"),
        }
    }

    #[test]
    fn test_parse_companion_ask_without_display() {
        let args = vec!["syntropctl", "companion", "ask", "inspect screen"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Companion {
                command: syntropctl_cli::cmd::companion::CompanionCommands::Ask { prompt, display },
            } => {
                assert_eq!(prompt, "inspect screen");
                assert!(display.is_none());
            }
            _ => panic!("Expected Companion Ask"),
        }
    }

    #[test]
    fn test_parse_companion_execute_without_dry_run() {
        let args = vec!["syntropctl", "companion", "execute", "type hello world"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Companion {
                command:
                    syntropctl_cli::cmd::companion::CompanionCommands::Execute {
                        instruction,
                        display,
                        dry_run,
                        grounding,
                    },
            } => {
                assert_eq!(instruction, "type hello world");
                assert!(display.is_none());
                assert!(!dry_run);
                assert!(grounding);
            }
            _ => panic!("Expected Companion Execute"),
        }
    }

    #[test]
    fn test_parse_companion_listen_options() {
        let args = vec![
            "syntropctl",
            "companion",
            "listen",
            "--hotkey",
            "Ctrl+Alt+T",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Companion {
                command:
                    syntropctl_cli::cmd::companion::CompanionCommands::Listen {
                        voice,
                        hotkey,
                        once,
                    },
            } => {
                assert!(!voice);
                assert_eq!(hotkey.as_deref(), Some("Ctrl+Alt+T"));
                assert!(!once);
            }
            _ => panic!("Expected Companion Listen"),
        }
    }
}
