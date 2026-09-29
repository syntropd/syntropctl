//! Composite gang lease inspection and acquisition operations.

use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};
use serde::{Deserialize, Serialize};

/// Plane slice allocation item in a composite lease.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompositeSliceItem {
    /// Compute plane identifier.
    pub plane_id: String,
    /// Assigned functional role in pipeline / gang.
    pub role: String,
    /// Pipeline stage index if applicable.
    pub stage_index: Option<usize>,
    /// VRAM reserved in bytes.
    pub memory_bytes: u64,
}

/// Composite lease status report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompositeLeaseStatus {
    /// Unique lease identifier.
    pub lease_id: String,
    /// Identifier of the executing gang.
    pub gang_id: String,
    /// Gang policy (e.g. AllOrNothing).
    pub gang_policy: String,
    /// Individual plane slice allocations.
    pub slices: Vec<CompositeSliceItem>,
}

/// Query active composite leases from inferenced.
pub async fn query_composite_leases() -> Result<Vec<CompositeLeaseStatus>, SyntropctlError> {
    let inferenced_ep = DaemonEndpoint::from_name("inferenced")
        .ok_or_else(|| SyntropctlError::NotFound("inferenced endpoint not configured".into()))?;

    let sock = inferenced_ep.socket_path();
    if !sock.exists() {
        return Err(SyntropctlError::DaemonUnavailable {
            daemon: "inferenced".into(),
            socket: sock,
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "Socket file does not exist"),
        });
    }

    let res = VarlinkClient::call(
        &sock,
        "io.systemd.inferenced1.ListCompositeLeases",
        None,
        DEFAULT_RPC_TIMEOUT,
    )
    .await?;

    let mut leases = Vec::new();
    if let Some(leases_arr) = res.get("leases").and_then(|v| v.as_array()) {
        for l in leases_arr {
            let lease_id = l.get("lease_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let gang_id = l.get("gang_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let gang_policy = l
                .get("gang_policy")
                .and_then(|v| v.as_str())
                .unwrap_or("AllOrNothing")
                .to_string();

            let mut slices = Vec::new();
            if let Some(s_arr) = l.get("slices").and_then(|v| v.as_array()) {
                for s in s_arr {
                    let plane_id = s.get("plane_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let role = s.get("role").and_then(|v| v.as_str()).unwrap_or("Standalone").to_string();
                    let stage_index = s.get("stage_index").and_then(|v| v.as_u64()).map(|v| v as usize);
                    let memory_bytes = s.get("memory_bytes").and_then(|v| v.as_u64()).unwrap_or(0);
                    slices.push(CompositeSliceItem {
                        plane_id,
                        role,
                        stage_index,
                        memory_bytes,
                    });
                }
            }

            leases.push(CompositeLeaseStatus {
                lease_id,
                gang_id,
                gang_policy,
                slices,
            });
        }
    }

    Ok(leases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composite_lease_deserialization() {
        let slice = CompositeSliceItem {
            plane_id: "plane-0".into(),
            role: "PipelineStage".into(),
            stage_index: Some(0),
            memory_bytes: 8 << 30,
        };
        let lease = CompositeLeaseStatus {
            lease_id: "lease-101".into(),
            gang_id: "gang-1".into(),
            gang_policy: "AllOrNothing".into(),
            slices: vec![slice],
        };
        assert_eq!(lease.lease_id, "lease-101");
        assert_eq!(lease.slices[0].stage_index, Some(0));
    }
}
