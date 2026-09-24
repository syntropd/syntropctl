//! Fleet status collection across all syntropd subsystem daemons.

use crate::daemon::DaemonEndpoint;
use crate::varlink::VarlinkClient;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

/// Status assessment for an individual daemon in the suite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub name: String,
    pub unit_name: String,
    pub socket_path: PathBuf,
    pub socket_exists: bool,
    pub responsive: bool,
    pub latency_ms: Option<u64>,
    pub product: Option<String>,
    pub version: Option<String>,
    pub interfaces: Vec<String>,
    pub error_message: Option<String>,
}

/// Collect status from a list of daemon endpoints.
pub async fn check_daemon_status(endpoint: &DaemonEndpoint) -> DaemonStatus {
    let socket = endpoint.socket_path();
    let exists = socket.exists();

    if !exists {
        return DaemonStatus {
            name: endpoint.name.to_string(),
            unit_name: endpoint.unit_name.to_string(),
            socket_path: socket,
            socket_exists: false,
            responsive: false,
            latency_ms: None,
            product: None,
            version: None,
            interfaces: Vec::new(),
            error_message: Some("Socket file not found".to_string()),
        };
    }

    let start = Instant::now();
    match VarlinkClient::get_info(&socket).await {
        Ok(info) => {
            let elapsed = start.elapsed().as_millis() as u64;
            DaemonStatus {
                name: endpoint.name.to_string(),
                unit_name: endpoint.unit_name.to_string(),
                socket_path: socket,
                socket_exists: true,
                responsive: true,
                latency_ms: Some(elapsed),
                product: Some(info.product),
                version: Some(info.version),
                interfaces: info.interfaces,
                error_message: None,
            }
        }
        Err(e) => DaemonStatus {
            name: endpoint.name.to_string(),
            unit_name: endpoint.unit_name.to_string(),
            socket_path: socket,
            socket_exists: true,
            responsive: false,
            latency_ms: None,
            product: None,
            version: None,
            interfaces: Vec::new(),
            error_message: Some(e.to_string()),
        },
    }
}

/// Query status across the entire daemon fleet concurrently.
pub async fn collect_fleet_status(endpoints: &[DaemonEndpoint]) -> Vec<DaemonStatus> {
    let mut results = Vec::with_capacity(endpoints.len());
    for ep in endpoints {
        results.push(check_daemon_status(ep).await);
    }
    results
}
