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

/// Wait before the single status retry (cold activation window).
const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(5);

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

    // Cold daemons under socket activation can miss the first snappy
    // probe while they fork and initialize. One patient retry keeps
    // `status` honest during installs and first boots.
    let start = Instant::now();
    let mut attempt = VarlinkClient::get_info(&socket).await;
    if attempt.is_err() {
        tokio::time::sleep(RETRY_DELAY).await;
        attempt = VarlinkClient::get_info(&socket).await;
    }
    match attempt {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daemon::{DaemonEndpoint, DaemonKind};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    fn test_endpoint() -> DaemonEndpoint {
        DaemonEndpoint {
            kind: DaemonKind::Toold,
            name: "test",
            unit_name: "test.service",
            default_socket: "/nonexistent-test.sock",
            env_var: "SYNTROP_TEST_STATUS_SOCK",
            interface: "org.varlink.service",
            description: "test",
        }
    }

    #[tokio::test]
    async fn cold_daemon_answers_on_retry() {
        let path = std::env::temp_dir().join(format!("status-retry-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).unwrap();
        std::env::set_var("SYNTROP_TEST_STATUS_SOCK", &path);
        let server = tokio::spawn(async move {
            // First connection: hold past the 1.5s probe (cold boot).
            let (mut s1, _) = listener.accept().await.unwrap();
            let mut chunk = [0u8; 1024];
            let _ = s1.read(&mut chunk).await;
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            drop(s1);
            // Second connection: answer.
            let (mut s2, _) = listener.accept().await.unwrap();
            let _ = s2.read(&mut chunk).await;
            let reply = serde_json::json!({
                "parameters": {
                    "vendor": "t", "product": "p", "version": "1",
                    "url": "u", "interfaces": []
                }
            });
            let mut bytes = serde_json::to_vec(&reply).unwrap();
            bytes.push(0);
            s2.write_all(&bytes).await.unwrap();
        });
        let status = check_daemon_status(&test_endpoint()).await;
        server.await.unwrap();
        std::env::remove_var("SYNTROP_TEST_STATUS_SOCK");
        let _ = std::fs::remove_file(&path);
        assert!(
            status.responsive,
            "retry should catch the cold daemon: {:?}",
            status.error_message
        );
        assert_eq!(status.product.as_deref(), Some("p"));
    }
}
