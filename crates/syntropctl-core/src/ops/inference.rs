//! Inference and embedding proxy communicating with runtimed.

use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};
use serde::{Deserialize, Serialize};

/// Result of text generation from runtimed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOutput {
    pub text: String,
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub finish_reason: String,
    pub duration_ms: u64,
}

/// Execute prompt text generation via runtimed.
pub async fn generate_text(
    prompt: &str,
    model: &str,
    max_tokens: usize,
    temperature: f32,
) -> Result<GenerationOutput, SyntropctlError> {
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

    let params = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "max_tokens": max_tokens,
        "temperature": temperature,
    });

    let res = VarlinkClient::call(
        &sock,
        "io.syntrop.Runtime1.Generate",
        Some(params),
        DEFAULT_RPC_TIMEOUT,
    )
    .await?;

    let inner = res.get("result").ok_or_else(|| {
        SyntropctlError::MalformedReply("Missing 'result' object in Generate reply".into())
    })?;

    let text = inner.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let prompt_tokens = inner.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let completion_tokens = inner.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let finish_reason = inner.get("finish_reason").and_then(|v| v.as_str()).unwrap_or("stop").to_string();
    let duration_ms = inner.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(0);

    Ok(GenerationOutput {
        text,
        prompt_tokens,
        completion_tokens,
        finish_reason,
        duration_ms,
    })
}

/// Generate text embeddings via runtimed.
pub async fn embed_text(text: &str, model: &str) -> Result<Vec<f32>, SyntropctlError> {
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

    let params = serde_json::json!({
        "model": model,
        "text": text,
    });

    let res = VarlinkClient::call(
        &sock,
        "io.syntrop.Runtime1.Embed",
        Some(params),
        DEFAULT_RPC_TIMEOUT,
    )
    .await?;

    let raw = res.get("embedding").and_then(|v| v.as_array()).ok_or_else(|| {
        SyntropctlError::MalformedReply("Missing 'embedding' array in Embed reply".into())
    })?;

    let vec: Vec<f32> = raw.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect();
    Ok(vec)
}
