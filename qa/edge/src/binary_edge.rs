//! Edge tests driving the built syntropctl binary end to end.

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::process::Command;

    /// Debug binary built by `cargo test --workspace` before tests run.
    fn binary() -> PathBuf {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("qa/edge lives two levels below the workspace root")
            .to_path_buf();
        let bin = root.join("target/debug/syntropctl");
        assert!(
            bin.exists(),
            "run via cargo test --workspace so {} is built",
            bin.display()
        );
        bin
    }

    /// Point the child at sockets that cannot exist, never at live daemons.
    fn hermetic(mut cmd: Command) -> Command {
        for var in [
            "SYNTROP_SENTRY_SOCKET",
            "SYNTROP_INFERENCE_SOCKET",
            "SYNTROP_MODELD_SOCKET",
            "SYNTROP_CONTEXTD_SOCKET",
            "SYNTROP_TOOLD_SOCKET",
            "SYNTROP_RUNTIMED_SOCKET",
            "SYNTROP_ROUTER_SOCKET",
        ] {
            cmd.env(var, "/nonexistent-qa-socket.sock");
        }
        cmd
    }

    #[test]
    fn test_help_exits_success() {
        let out = Command::new(binary()).arg("--help").output().unwrap();
        assert!(out.status.success());
        assert!(String::from_utf8_lossy(&out.stdout).contains("syntropctl"));
    }

    #[test]
    fn test_completions_emit_bash_script() {
        let out = Command::new(binary())
            .args(["completions", "bash"])
            .output()
            .unwrap();
        assert!(out.status.success());
        assert!(String::from_utf8_lossy(&out.stdout).contains("syntropctl"));
    }

    #[test]
    fn test_unknown_daemon_fails_loudly() {
        let out = hermetic(Command::new(binary()))
            .args(["info", "bogus-daemon"])
            .output()
            .unwrap();
        assert!(!out.status.success());
        assert!(String::from_utf8_lossy(&out.stderr).contains("Unknown daemon"));
    }

    #[test]
    fn test_status_json_reports_seven_daemons() {
        let out = hermetic(Command::new(binary()))
            .args(["--json", "status"])
            .output()
            .unwrap();
        assert!(out.status.success());
        let statuses: Vec<serde_json::Value> =
            serde_json::from_slice(&out.stdout).expect("status --json prints an array");
        assert_eq!(statuses.len(), 7);
        assert!(statuses
            .iter()
            .all(|s| s["socket_exists"].as_bool() == Some(false)));
    }
}
