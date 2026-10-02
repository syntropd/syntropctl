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
    pub storyboard_path: Option<String>,
    pub manifest_path: Option<String>,
    pub keyframes: Option<usize>,
}

/// Execute generative visual synthesis via runtimed.
pub async fn generate_visual(
    prompt: &str,
    model: Option<&str>,
    loras: &[String],
    width: u32,
    height: u32,
    storyboard: Option<usize>,
    allow_degrade: bool,
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
        "allow_degrade": allow_degrade,
    });
    if let Some(m) = model {
        params["model"] = serde_json::json!(m);
    }
    if !loras.is_empty() {
        params["loras"] = serde_json::json!(loras);
    }
    if let Some(sb) = storyboard {
        params["storyboard"] = serde_json::json!(sb);
    }

    let res = match VarlinkClient::call(
        &sock,
        "io.syntrop.Runtime1.GenerateVisual",
        Some(params),
        GENERATE_RPC_TIMEOUT,
    )
    .await {
        Ok(v) => v,
        Err(SyntropctlError::ProtocolError { error, parameters })
            if error.ends_with(".HardwareIncompatible") =>
        {
            let p = parameters.unwrap_or(serde_json::Value::Null);
            let deficit = p.get("deficit").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let estimated_cpu_latency_secs = p.get("estimated_cpu_latency_secs").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let suggested_alternatives = p.get("suggested_alternatives")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default();
            return Err(SyntropctlError::HardwareIncompatible {
                deficit,
                estimated_cpu_latency_secs,
                suggested_alternatives,
            });
        }
        Err(e) => return Err(e),
    };

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
        storyboard_path: res.get("storyboard_path").and_then(|v| v.as_str()).map(String::from),
        manifest_path: res.get("manifest_path").and_then(|v| v.as_str()).map(String::from),
        keyframes: res.get("keyframes").and_then(|v| v.as_u64()).map(|k| k as usize),
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
            storyboard_path: None,
            manifest_path: None,
            keyframes: None,
        };
        assert_eq!(out.width, 512);
        assert_eq!(out.height, 512);
    }
}
