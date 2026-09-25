//! Sandboxed tool execution proxy communicating with toold.

use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};
use serde::{Deserialize, Serialize};

/// Result of a sandboxed command execution in toold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRunResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

/// Execute a sandboxed tool or command via toold.
pub async fn execute_sandboxed_tool(
    tool: &str,
    args: &[String],
    sandbox_profile: Option<&str>,
) -> Result<ToolRunResult, SyntropctlError> {
    let toold_ep = DaemonEndpoint::from_name("toold")
        .ok_or_else(|| SyntropctlError::NotFound("toold endpoint not configured".into()))?;

    let sock = toold_ep.socket_path();
    if !sock.exists() {
        return Err(SyntropctlError::DaemonUnavailable {
            daemon: "toold".into(),
            socket: sock,
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "Socket file does not exist"),
        });
    }

    let mut params = serde_json::json!({
        "name": tool,
        "args": args,
    });

    if let Some(prof) = sandbox_profile {
        params["sandbox"] = serde_json::json!(prof);
    }

    let res = VarlinkClient::call(
        &sock,
        "io.syntrop.Tool1.ExecuteTool",
        Some(params),
        DEFAULT_RPC_TIMEOUT,
    )
    .await?;

    let inner = res.get("result").unwrap_or(&res);
    let exit_code = inner.get("exit_code").and_then(|v| v.as_i64()).unwrap_or(-1) as i32;
    let stdout = inner.get("stdout").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let stderr = inner.get("stderr").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let duration_ms = inner.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(0);

    Ok(ToolRunResult {
        exit_code,
        stdout,
        stderr,
        duration_ms,
    })
}
