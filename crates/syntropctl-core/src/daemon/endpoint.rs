//! Static registry and socket resolution for all syntropd suite daemons.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Enumeration of all daemons forming the syntropd AI subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DaemonKind {
    Sentry,
    Inferenced,
    Modeld,
    Contextd,
    Toold,
    Runtimed,
}

/// Metadata and socket locator for a syntropd suite daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonEndpoint {
    pub kind: DaemonKind,
    pub name: &'static str,
    pub unit_name: &'static str,
    pub default_socket: &'static str,
    pub env_var: &'static str,
    pub interface: &'static str,
    pub description: &'static str,
}

impl DaemonEndpoint {
    /// Return the resolved socket path respecting environment variable overrides.
    pub fn socket_path(&self) -> PathBuf {
        if let Ok(val) = std::env::var(self.env_var) {
            if !val.trim().is_empty() {
                return PathBuf::from(val);
            }
        }
        PathBuf::from(self.default_socket)
    }

    /// Check if the socket file currently exists on the filesystem.
    pub fn socket_exists(&self) -> bool {
        self.socket_path().exists()
    }

    /// Return all official daemons in the syntropd subsystem.
    pub fn all() -> &'static [DaemonEndpoint] {
        &DAEMONS
    }

    /// Locate a daemon endpoint definition by its canonical or shorthand name.
    pub fn from_name(name: &str) -> Option<&'static DaemonEndpoint> {
        let clean = name.trim().to_lowercase();
        DAEMONS.iter().find(|d| {
            d.name == clean
                || d.unit_name.trim_end_matches(".service") == clean
                || clean.contains(d.name)
        })
    }
}

pub static DAEMONS: [DaemonEndpoint; 6] = [
    DaemonEndpoint {
        kind: DaemonKind::Sentry,
        name: "sentry",
        unit_name: "sentry.service",
        default_socket: "/run/syntrop/io.syntrop.Sentry1",
        env_var: "SYNTROP_SENTRY_SOCKET",
        interface: "io.syntrop.Sentry1",
        description: "Autonomous Zero-Trust Supervisor & Incident Triage",
    },
    DaemonEndpoint {
        kind: DaemonKind::Inferenced,
        name: "inferenced",
        unit_name: "inferenced.service",
        default_socket: "/run/syntrop/io.syntrop.Inference1",
        env_var: "SYNTROP_INFERENCE_SOCKET",
        interface: "io.syntrop.Inference1",
        description: "Heterogeneous Hardware Arbiter & Memory Broker",
    },
    DaemonEndpoint {
        kind: DaemonKind::Modeld,
        name: "modeld",
        unit_name: "modeld.service",
        default_socket: "/run/syntrop/io.syntrop.Model1",
        env_var: "SYNTROP_MODELD_SOCKET",
        interface: "io.syntrop.Model1",
        description: "CAS Model Registry & Layer Deduplicator",
    },
    DaemonEndpoint {
        kind: DaemonKind::Contextd,
        name: "contextd",
        unit_name: "contextd.service",
        default_socket: "/run/syntrop/io.syntrop.Context1",
        env_var: "SYNTROP_CONTEXTD_SOCKET",
        interface: "io.syntrop.Context1",
        description: "System Chronology & Configuration Drift Tracker",
    },
    DaemonEndpoint {
        kind: DaemonKind::Toold,
        name: "toold",
        unit_name: "toold.service",
        default_socket: "/run/syntrop/io.syntrop.Tool1",
        env_var: "SYNTROP_TOOLD_SOCKET",
        interface: "io.syntrop.Tool1",
        description: "Sandboxed Agentic Action & Execution Broker",
    },
    DaemonEndpoint {
        kind: DaemonKind::Runtimed,
        name: "runtimed",
        unit_name: "runtimed.service",
        default_socket: "/run/syntrop/io.syntrop.Runtime1",
        env_var: "SYNTROP_RUNTIMED_SOCKET",
        interface: "io.syntrop.Runtime1",
        description: "Headless Neural Model Execution Engine",
    },
];
