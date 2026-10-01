//! Ambient audio sensing operations.

use base64::Engine as _;
use serde::{Deserialize, Serialize};

use super::endpoint::{connect_check, sensory_socket_path};
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};

/// Captured audio result with raw PCM samples.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioCaptureResult {
    pub audio_pcm_base64: String,
    pub sample_rate: u32,
    pub channels: u32,
    #[serde(skip_serializing, default)]
    pub raw_bytes: Vec<u8>,
}

/// Record ambient audio PCM from default input device.
pub async fn capture_audio(
    duration_ms: Option<u32>,
    sample_rate: Option<u32>,
) -> Result<AudioCaptureResult, SyntropctlError> {
    let sock = sensory_socket_path();
    connect_check(&sock)?;

    let mut params = serde_json::Map::new();
    if let Some(d) = duration_ms {
        params.insert("duration_ms".to_string(), serde_json::json!(d));
    }
    if let Some(s) = sample_rate {
        params.insert("sample_rate".to_string(), serde_json::json!(s));
    }

    let p = if params.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(params))
    };
    let res = VarlinkClient::call(
        &sock,
        "io.syntrop.Sensory1.CaptureAudio",
        p,
        DEFAULT_RPC_TIMEOUT,
    )
    .await?;

    let b64 = res
        .get("audio_pcm_base64")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            SyntropctlError::MalformedReply("Missing audio_pcm_base64 in CaptureAudio reply".into())
        })?;
    let sr = res
        .get("sample_rate")
        .and_then(|v| v.as_u64())
        .unwrap_or(16000) as u32;
    let ch = res.get("channels").and_then(|v| v.as_u64()).unwrap_or(1) as u32;

    let raw = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| SyntropctlError::MalformedReply(format!("Invalid base64 audio PCM: {e}")))?;

    Ok(AudioCaptureResult {
        audio_pcm_base64: b64.to_string(),
        sample_rate: sr,
        channels: ch,
        raw_bytes: raw,
    })
}
