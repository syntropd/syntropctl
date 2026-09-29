//! Cluster mesh and multi-GPU composite operations.

pub mod composite;
pub mod status;

pub use composite::{query_composite_leases, CompositeLeaseStatus, CompositeSliceItem};
pub use status::{query_cluster_status, ClusterNodeSummary, ClusterStatusReport};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_mod_exports() {
        let summary = ClusterNodeSummary {
            id: "node-0".into(),
            address: "127.0.0.1:9099".into(),
            status: "active".into(),
            total_vram_bytes: 16 << 30,
            available_vram_bytes: 16 << 30,
            latency_ms: 1,
        };
        assert_eq!(summary.id, "node-0");
    }
}
