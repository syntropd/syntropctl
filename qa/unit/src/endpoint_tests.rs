//! Unit tests for daemon endpoints and socket path resolution.

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use syntropctl_core::daemon::{DaemonEndpoint, DaemonKind, DAEMONS};

    #[test]
    fn test_all_daemons_present() {
        assert_eq!(DAEMONS.len(), 7);
        let names: Vec<&str> = DAEMONS.iter().map(|d| d.name).collect();
        assert!(names.contains(&"sentry"));
        assert!(names.contains(&"inferenced"));
        assert!(names.contains(&"modeld"));
        assert!(names.contains(&"contextd"));
        assert!(names.contains(&"toold"));
        assert!(names.contains(&"runtimed"));
        assert!(names.contains(&"routerd"));
    }

    #[test]
    fn test_lookup_by_canonical_name() {
        let sentry = DaemonEndpoint::from_name("sentry").unwrap();
        assert_eq!(sentry.kind, DaemonKind::Sentry);
        assert_eq!(sentry.unit_name, "sentry.service");
        assert_eq!(sentry.interface, "io.syntrop.Sentry1");

        let toold = DaemonEndpoint::from_name("toold").unwrap();
        assert_eq!(toold.kind, DaemonKind::Toold);
        assert_eq!(toold.unit_name, "toold.service");
        assert_eq!(toold.interface, "io.syntrop.Tool1");

        let routerd = DaemonEndpoint::from_name("routerd").unwrap();
        assert_eq!(routerd.kind, DaemonKind::Routerd);
        assert_eq!(routerd.unit_name, "routerd.service");
        assert_eq!(routerd.interface, "io.syntrop.Router1");
    }

    #[test]
    fn test_lookup_by_unit_name() {
        let runtimed = DaemonEndpoint::from_name("runtimed.service").unwrap();
        assert_eq!(runtimed.kind, DaemonKind::Runtimed);

        let routerd = DaemonEndpoint::from_name("routerd.service").unwrap();
        assert_eq!(routerd.kind, DaemonKind::Routerd);

        let unknown = DaemonEndpoint::from_name("nonexistent_daemon");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_environment_variable_socket_override() {
        let ep = DaemonEndpoint::from_name("runtimed").unwrap();
        let default_path = ep.socket_path();
        assert_eq!(default_path, PathBuf::from("/run/syntrop/io.syntrop.Runtime1"));

        std::env::set_var("SYNTROP_RUNTIMED_SOCKET", "/tmp/custom_runtime.sock");
        let custom_path = ep.socket_path();
        assert_eq!(custom_path, PathBuf::from("/tmp/custom_runtime.sock"));
        std::env::remove_var("SYNTROP_RUNTIMED_SOCKET");

        let ep_r = DaemonEndpoint::from_name("routerd").unwrap();
        assert_eq!(ep_r.socket_path(), PathBuf::from("/run/syntrop/io.syntrop.Router1"));
        std::env::set_var("SYNTROP_ROUTER_SOCKET", "/tmp/custom_router.sock");
        assert_eq!(ep_r.socket_path(), PathBuf::from("/tmp/custom_router.sock"));
        std::env::remove_var("SYNTROP_ROUTER_SOCKET");
    }
}
