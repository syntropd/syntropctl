//! Cluster mesh and node status querying operations.

use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};
use serde::{Deserialize, Serialize};

/// Summary of an individual node in the cluster mesh.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClusterNodeSummary {
    /// Unique identifier of the node.
    pub id: String,
    /// Mesh network socket address.
    pub address: String,
    /// Current operational status.
    pub status: String,
    /// Total VRAM capacity in bytes across node accelerators.
    pub total_vram_bytes: u64,
    /// Currently available unallocated VRAM in bytes.
    pub available_vram_bytes: u64,
    /// Round-trip ping latency in milliseconds.
    pub latency_ms: u32,
}

/// Aggregated cluster topology and mesh status report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClusterStatusReport {
    /// Number of reachable active nodes.
    pub active_nodes: usize,
    /// Total number of registered nodes.
    pub total_nodes: usize,
    /// Aggregated cluster VRAM capacity in bytes.
    pub total_vram_bytes: u64,
    /// Aggregated available cluster VRAM in bytes.
    pub available_vram_bytes: u64,
    /// Individual node breakdown.
    pub nodes: Vec<ClusterNodeSummary>,
}

/// Query cluster mesh nodes and aggregated VRAM capacity from routerd.
pub async fn query_cluster_status() -> Result<ClusterStatusReport, SyntropctlError> {
    let routerd_ep = DaemonEndpoint::from_name("routerd")
        .ok_or_else(|| SyntropctlError::NotFound("routerd endpoint not configured".into()))?;

    let sock = routerd_ep.socket_path();
    if !sock.exists() {
        return Err(SyntropctlError::DaemonUnavailable {
            daemon: "routerd".into(),
            socket: sock,
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "Socket file does not exist"),
        });
    }

    let res = VarlinkClient::call(
        &sock,
        "io.syntrop.Router1.GetDaemonStatus",
        None,
        DEFAULT_RPC_TIMEOUT,
    )
    .await?;

    let mut nodes = Vec::new();
    let mut total_vram: u64 = 0;
    let mut avail_vram: u64 = 0;
    let mut active = 0;

    if let Some(nodes_arr) = res.get("cluster_nodes").and_then(|v| v.as_array()) {
        for n in nodes_arr {
            let id = n
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let address = n
                .get("address")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let status = n
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("active")
                .to_string();
            let total = n
                .get("total_vram_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let avail = n
                .get("available_vram_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let latency = n.get("latency_ms").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

            if status == "active" || status == "Active" {
                active += 1;
            }
            total_vram = total_vram.saturating_add(total);
            avail_vram = avail_vram.saturating_add(avail);

            nodes.push(ClusterNodeSummary {
                id,
                address,
                status,
                total_vram_bytes: total,
                available_vram_bytes: avail,
                latency_ms: latency,
            });
        }
    }

    let total_nodes = nodes.len();
    Ok(ClusterStatusReport {
        active_nodes: active,
        total_nodes,
        total_vram_bytes: total_vram,
        available_vram_bytes: avail_vram,
        nodes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_report_construction() {
        let node = ClusterNodeSummary {
            id: "node-alpha".into(),
            address: "10.0.0.1:9099".into(),
            status: "active".into(),
            total_vram_bytes: 32 << 30,
            available_vram_bytes: 16 << 30,
            latency_ms: 2,
        };

        let report = ClusterStatusReport {
            active_nodes: 1,
            total_nodes: 1,
            total_vram_bytes: 32 << 30,
            available_vram_bytes: 16 << 30,
            nodes: vec![node],
        };

        assert_eq!(report.active_nodes, 1);
        assert_eq!(report.nodes[0].id, "node-alpha");
    }
}
