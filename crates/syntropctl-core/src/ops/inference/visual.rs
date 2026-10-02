//! Generative visual synthesis operations communicating with runtimed.

use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, GENERATE_RPC_TIMEOUT};
use serde::{Deserialize, Serialize};

/// Result of visual generation from runtimed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualGenerationOutput {
    pub image_path: String,
    pub bytes: usize,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

/// Execute generative visual synthesis via runtimed.
pub async fn generate_visual(
    prompt: &str,
    model: Option<&str>,
    loras: &[String],
    width: u32,
    height: u32,
) -> Result<VisualGenerationOutput, SyntropctlError> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        return Err(SyntropctlError::OperationFailed("prompt cannot be empty".into()));
    }

    let runtimed_ep = DaemonEndpoint::from_name("runtimed")
        .ok_or_else(|| SyntropctlError::NotFound("runtimed endpoint not configured".into()))?;
    let sock = runtimed_ep.socket_path();
    if !sock.exists() {
        return Err(SyntropctlError::DaemonUnavailable {
            daemon: "runtimed".into(),
            socket: sock,
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "Socket file does not exist"),
        });
    }

    let mut params = serde_json::json!({
        "prompt": trimmed,
        "width": width,
        "height": height,
    });
    if let Some(m) = model {
        params["model"] = serde_json::json!(m);
    }
    if !loras.is_empty() {
        params["loras"] = serde_json::json!(loras);
    }

    let res = VarlinkClient::call(
        &sock,
        "io.syntrop.Runtime1.GenerateVisual",
        Some(params),
        GENERATE_RPC_TIMEOUT,
    )
    .await?;

    Ok(VisualGenerationOutput {
        image_path: res
            .get("image_path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        bytes: res.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        width: res
            .get("width")
            .and_then(|v| v.as_u64())
            .unwrap_or(width as u64) as u32,
        height: res
            .get("height")
            .and_then(|v| v.as_u64())
            .unwrap_or(height as u64) as u32,
        format: res
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("png")
            .to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_generation_output_struct() {
        let out = VisualGenerationOutput {
            image_path: "/tmp/img.png".into(),
            bytes: 1024,
            width: 512,
            height: 512,
            format: "png".into(),
        };
        assert_eq!(out.width, 512);
        assert_eq!(out.height, 512);
    }
}
