//! Composite operator presence estimation queries.

use serde::{Deserialize, Serialize};

use super::endpoint::{connect_check, sensory_socket_path};
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};

/// Fused operator presence detection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorPresenceResult {
    pub present: bool,
    pub confidence: f64,
    pub reason: String,
}

/// Query composite operator presence.
pub async fn get_operator_presence() -> Result<OperatorPresenceResult, SyntropctlError> {
    let sock = sensory_socket_path();
    connect_check(&sock)?;

    let res = VarlinkClient::call(
        &sock,
        "io.syntrop.Sensory1.GetOperatorPresence",
        None,
        DEFAULT_RPC_TIMEOUT,
    )
    .await?;

    let present = res
        .get("present")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let confidence = res
        .get("confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let reason = res
        .get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(OperatorPresenceResult {
        present,
        confidence,
        reason,
    })
}
