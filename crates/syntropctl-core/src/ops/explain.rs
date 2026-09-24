//! Incident explanation and root-cause analysis combining sentry and contextd.

use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};
use serde::{Deserialize, Serialize};

/// Comprehensive incident explanation fusing supervisor triage and system drift.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentReport {
    pub unit: String,
    pub status: String,
    pub root_cause: String,
    pub confidence: f32,
    pub recommended_action: String,
    pub remediation_command: Option<String>,
    pub journal_slice: Vec<String>,
    pub recent_drift: Vec<String>,
}

/// Query sentry and contextd to explain failure or state of a systemd unit.
pub async fn explain_unit(unit: &str) -> Result<IncidentReport, SyntropctlError> {
    let sentry_ep = DaemonEndpoint::from_name("sentry")
        .ok_or_else(|| SyntropctlError::NotFound("sentry endpoint not found".into()))?;
    let contextd_ep = DaemonEndpoint::from_name("contextd")
        .ok_or_else(|| SyntropctlError::NotFound("contextd endpoint not found".into()))?;

    let sentry_socket = sentry_ep.socket_path();
    let contextd_socket = contextd_ep.socket_path();

    let mut root_cause = "No active failure record found in sentry.".to_string();
    let mut confidence = 0.0f32;
    let mut recommended_action = "Inspect journal logs using journalctl.".to_string();
    let mut remediation_cmd = None;
    let mut journal_slice = Vec::new();
    let mut status = "active".to_string();

    // Query sentry if available
    if sentry_socket.exists() {
        let params = serde_json::json!({ "unit": unit });
        if let Ok(res) = VarlinkClient::call(
            &sentry_socket,
            "io.syntrop.Sentry1.TriageUnit",
            Some(params),
            DEFAULT_RPC_TIMEOUT,
        )
        .await
        {
            if let Some(rc) = res.get("root_cause").and_then(|v| v.as_str()) {
                root_cause = rc.to_string();
            }
            if let Some(c) = res.get("confidence").and_then(|v| v.as_f64()) {
                confidence = c as f32;
            }
            if let Some(act) = res.get("recommended_action").and_then(|v| v.as_str()) {
                recommended_action = act.to_string();
            }
            if let Some(cmd) = res.get("remediation_command").and_then(|v| v.as_str()) {
                remediation_cmd = Some(cmd.to_string());
            }
            if let Some(logs) = res.get("journal_slice").and_then(|v| v.as_array()) {
                journal_slice = logs
                    .iter()
                    .filter_map(|l| l.as_str().map(|s| s.to_string()))
                    .collect();
            }
            if let Some(st) = res.get("status").and_then(|v| v.as_str()) {
                status = st.to_string();
            }
        }
    }

    // Query contextd drift if available
    let mut recent_drift = Vec::new();
    if contextd_socket.exists() {
        let params = serde_json::json!({ "unit": unit });
        if let Ok(res) = VarlinkClient::call(
            &contextd_socket,
            "io.syntrop.Context1.GetTimeline",
            Some(params),
            DEFAULT_RPC_TIMEOUT,
        )
        .await
        {
            if let Some(events) = res.get("events").and_then(|v| v.as_array()) {
                for ev in events {
                    if let Some(summary) = ev.get("summary").and_then(|v| v.as_str()) {
                        recent_drift.push(summary.to_string());
                    }
                }
            }
        }
    }

    Ok(IncidentReport {
        unit: unit.to_string(),
        status,
        root_cause,
        confidence,
        recommended_action,
        remediation_command: remediation_cmd,
        journal_slice,
        recent_drift,
    })
}
