//! System configuration and journal drift query communicating with contextd.

use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};
use serde::{Deserialize, Serialize};

/// Configuration or state drift record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftEvent {
    pub path: String,
    pub change_type: String,
    pub timestamp: u64,
    pub details: String,
}

/// Query system configuration changes and drift from contextd.
pub async fn query_drift(unit: Option<&str>) -> Result<Vec<DriftEvent>, SyntropctlError> {
    let contextd_ep = DaemonEndpoint::from_name("contextd")
        .ok_or_else(|| SyntropctlError::NotFound("contextd endpoint not configured".into()))?;

    let sock = contextd_ep.socket_path();
    if !sock.exists() {
        return Err(SyntropctlError::DaemonUnavailable {
            daemon: "contextd".into(),
            socket: sock,
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "Socket file does not exist"),
        });
    }

    let params = match unit {
        Some(u) => serde_json::json!({ "unit": u }),
        None => serde_json::json!({}),
    };

    let res = VarlinkClient::call(
        &sock,
        "io.syntrop.Context1.GetTimeline",
        Some(params),
        DEFAULT_RPC_TIMEOUT,
    )
    .await?;

    let mut events = Vec::new();
    if let Some(evs) = res.get("events").and_then(|v| v.as_array()) {
        for e in evs {
            let path = e.get("path").or_else(|| e.get("target")).and_then(|v| v.as_str()).unwrap_or("/etc").to_string();
            let ctype = e.get("change_type").or_else(|| e.get("kind")).and_then(|v| v.as_str()).unwrap_or("modified").to_string();
            let ts = e.get("timestamp").and_then(|v| v.as_u64()).unwrap_or(0);
            let details = e.get("summary").or_else(|| e.get("details")).and_then(|v| v.as_str()).unwrap_or("").to_string();

            events.push(DriftEvent {
                path,
                change_type: ctype,
                timestamp: ts,
                details,
            });
        }
    }

    Ok(events)
}
