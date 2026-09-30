//! Device and accelerator query communicating with inferenced.

use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};
use serde::{Deserialize, Serialize};

/// Point-to-point interconnect link between devices.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceP2pLink {
    pub peer_plane_id: String,
    pub link_type: String,
    pub bandwidth_bytes_sec: u64,
    pub latency_nanos: u64,
}

/// Accelerator and compute device status report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceReport {
    pub id: String,
    pub device_type: String,
    pub vendor: String,
    pub model: String,
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
    pub available_memory_bytes: u64,
    pub headroom_pct: f32,
    pub psi_pressure: f32,
    pub status: String,
    pub p2p_links: Option<Vec<DeviceP2pLink>>,
}

/// Query hardware accelerators and resource pressure from inferenced.
pub async fn query_devices() -> Result<Vec<DeviceReport>, SyntropctlError> {
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

    let res = match VarlinkClient::call(
        &sock,
        "io.syntrop.Inference1.ListDevices",
        None,
        DEFAULT_RPC_TIMEOUT,
    )
    .await
    {
        Ok(r) => r,
        Err(_) => {
            VarlinkClient::call(
                &sock,
                "io.systemd.inferenced1.ListPlanes",
                None,
                DEFAULT_RPC_TIMEOUT,
            )
            .await?
        }
    };

    let mut devices = Vec::new();
    if let Some(devs) = res.get("devices").and_then(|v| v.as_array()) {
        for d in devs {
            let id = d.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
            let dtype = d.get("type").and_then(|v| v.as_str()).unwrap_or("cpu").to_string();
            let vendor = d.get("vendor").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let model = d.get("model").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let mem_total = d.get("memory_total").and_then(|v| v.as_u64()).unwrap_or(0);
            let mem_used = d.get("memory_used").and_then(|v| v.as_u64()).unwrap_or(0);
            let mem_avail = d
                .get("memory_available")
                .and_then(|v| v.as_u64())
                .unwrap_or_else(|| mem_total.saturating_sub(mem_used));
            let headroom_pct = if mem_total > 0 {
                (mem_avail as f32 / mem_total as f32) * 100.0
            } else {
                0.0
            };
            let psi = d.get("psi_pressure").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let status = d.get("status").and_then(|v| v.as_str()).unwrap_or("ready").to_string();
            let p2p_links = d
                .get("p2p_links")
                .and_then(|v| serde_json::from_value::<Vec<DeviceP2pLink>>(v.clone()).ok());

            devices.push(DeviceReport {
                id,
                device_type: dtype,
                vendor,
                model,
                memory_total_bytes: mem_total,
                memory_used_bytes: mem_used,
                available_memory_bytes: mem_avail,
                headroom_pct,
                psi_pressure: psi,
                status,
                p2p_links,
            });
        }
    } else if let Some(planes) = res.get("planes").and_then(|v| v.as_array()) {
        for p in planes {
            let id = p.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
            let dtype = p.get("kind").and_then(|v| v.as_str()).unwrap_or("compute").to_string();
            let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let mem_total = p.get("total_memory").and_then(|v| v.as_u64()).unwrap_or(0);
            let mem_avail = p.get("available_memory").and_then(|v| v.as_u64()).unwrap_or(0);
            let mem_used = mem_total.saturating_sub(mem_avail);
            let headroom_pct = if mem_total > 0 {
                (mem_avail as f32 / mem_total as f32) * 100.0
            } else {
                0.0
            };
            let p2p_links = p
                .get("p2p_links")
                .and_then(|v| serde_json::from_value::<Vec<DeviceP2pLink>>(v.clone()).ok());
            let status = if p.get("is_triage_reserved").and_then(|v| v.as_bool()).unwrap_or(false) {
                "triage-reserved".to_string()
            } else {
                "ready".to_string()
            };

            devices.push(DeviceReport {
                id,
                device_type: dtype,
                vendor: name,
                model: String::new(),
                memory_total_bytes: mem_total,
                memory_used_bytes: mem_used,
                available_memory_bytes: mem_avail,
                headroom_pct,
                psi_pressure: 0.0,
                status,
                p2p_links,
            });
        }
    }

    Ok(devices)
}
