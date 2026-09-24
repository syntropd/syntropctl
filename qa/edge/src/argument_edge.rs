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
}
