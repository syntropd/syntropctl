//! Inference and embedding proxy communicating with runtimed.

use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use crate::varlink::{GENERATE_RPC_TIMEOUT, VarlinkClient};
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
    reasoning_effort: Option<&str>,
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

    let mut params = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "max_tokens": max_tokens,
        "temperature": temperature,
    });
    if let Some(effort) = reasoning_effort {
        params["reasoning_effort"] = serde_json::json!(effort);
    }

    let res = VarlinkClient::call(
        &sock,
        "io.syntrop.Runtime1.Generate",
        Some(params),
        GENERATE_RPC_TIMEOUT,
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
        GENERATE_RPC_TIMEOUT,
    )
    .await?;

    let raw = res.get("embedding").and_then(|v| v.as_array()).ok_or_else(|| {
        SyntropctlError::MalformedReply("Missing 'embedding' array in Embed reply".into())
    })?;

    let vec: Vec<f32> = raw.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect();
    Ok(vec)
}

#[cfg(test)]
mod tests {
    use super::generate_text;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    /// A cold engine answers slower than the 10s metadata budget
    /// (multi-GB weight load); generation must survive it. Fails on
    /// the old 10s budget, passes on the generation budget.
    #[tokio::test]
    async fn generate_survives_slow_cold_engine() {
        let path =
            std::env::temp_dir().join(format!("gen-slow-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).unwrap();
        std::env::set_var("SYNTROP_RUNTIMED_SOCKET", &path);
        let server = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut chunk = [0u8; 4096];
            let _ = sock.read(&mut chunk).await;
            tokio::time::sleep(std::time::Duration::from_secs(12)).await;
            let reply = serde_json::json!({
                "parameters": {
                    "result": {
                        "text": "slow-hi",
                        "prompt_tokens": 1,
                        "completion_tokens": 1,
                        "finish_reason": "stop",
                        "duration_ms": 12000
                    }
                }
            });
            let bytes = serde_json::to_vec(&reply).unwrap();
            let _ = sock.write_all(&bytes).await;
            let _ = sock.write_all(&[0x00]).await;
        });
        let out = generate_text("hi", "cold-model", 1, 0.0, None).await.unwrap();
        assert_eq!(out.text, "slow-hi");
        assert_eq!(out.completion_tokens, 1);
        let _ = server.await;
        std::env::remove_var("SYNTROP_RUNTIMED_SOCKET");
        let _ = std::fs::remove_file(&path);
    }
}
