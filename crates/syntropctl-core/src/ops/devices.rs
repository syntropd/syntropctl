//! Device and accelerator query communicating with inferenced.

use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};
use serde::{Deserialize, Serialize};

/// Accelerator and compute device status report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceReport {
    pub id: String,
    pub device_type: String,
    pub vendor: String,
    pub model: String,
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
    pub psi_pressure: f32,
    pub status: String,
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

    let res = VarlinkClient::call(
        &sock,
        "io.syntrop.Inference1.ListDevices",
        None,
        DEFAULT_RPC_TIMEOUT,
    )
    .await?;

    let mut devices = Vec::new();
    if let Some(devs) = res.get("devices").and_then(|v| v.as_array()) {
        for d in devs {
            let id = d.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
            let dtype = d.get("type").and_then(|v| v.as_str()).unwrap_or("cpu").to_string();
            let vendor = d.get("vendor").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let model = d.get("model").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let mem_total = d.get("memory_total").and_then(|v| v.as_u64()).unwrap_or(0);
            let mem_used = d.get("memory_used").and_then(|v| v.as_u64()).unwrap_or(0);
            let psi = d.get("psi_pressure").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let status = d.get("status").and_then(|v| v.as_str()).unwrap_or("ready").to_string();

            devices.push(DeviceReport {
                id,
                device_type: dtype,
                vendor,
                model,
                memory_total_bytes: mem_total,
                memory_used_bytes: mem_used,
                psi_pressure: psi,
                status,
            });
        }
    }

    Ok(devices)
}
