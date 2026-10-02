//! Audio music generation and speech transcription operations communicating with runtimed.

use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, GENERATE_RPC_TIMEOUT};
use serde::{Deserialize, Serialize};

/// Result of music/audio generation from runtimed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioGenerationOutput {
    pub audio_path: String,
    pub sample_rate: u32,
    pub duration_ms: u64,
}

/// Result of speech transcription from runtimed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscribeAudioOutput {
    pub text: String,
    pub language: String,
    pub duration_ms: u64,
}

/// Result of streaming audio synthesis from runtimed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamAudioOutOutput {
    pub bytes_streamed: usize,
    pub sample_rate: u32,
    pub channels: u32,
    pub pcm_base64: Option<String>,
}

/// Execute generative music synthesis via runtimed.
pub async fn generate_music(
    prompt: &str,
    duration_sec: u32,
    bpm: Option<u32>,
) -> Result<AudioGenerationOutput, SyntropctlError> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        return Err(SyntropctlError::OperationFailed("prompt cannot be empty".into()));
    }
    if duration_sec == 0 {
        return Err(SyntropctlError::OperationFailed("duration_sec must be greater than 0".into()));
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
        "duration_sec": duration_sec,
    });
    if let Some(b) = bpm {
        params["bpm"] = serde_json::json!(b);
    }

    let res = VarlinkClient::call(
        &sock,
        "io.syntrop.Runtime1.GenerateMusic",
        Some(params),
        GENERATE_RPC_TIMEOUT,
    )
    .await?;

    Ok(AudioGenerationOutput {
        audio_path: res
            .get("audio_path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        sample_rate: res
            .get("sample_rate")
            .and_then(|v| v.as_u64())
            .unwrap_or(32000) as u32,
        duration_ms: res
            .get("duration_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(duration_sec as u64 * 1000),
    })
}

/// Execute neural speech-to-text transcription via runtimed.
pub async fn transcribe_audio(
    pcm_base64: &str,
    language: Option<&str>,
) -> Result<TranscribeAudioOutput, SyntropctlError> {
    let trimmed = pcm_base64.trim();
    if trimmed.is_empty() {
        return Err(SyntropctlError::OperationFailed("pcm_base64 cannot be empty".into()));
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
        "pcm_base64": trimmed,
    });
    if let Some(lang) = language {
        params["language"] = serde_json::json!(lang);
    }

    let res = VarlinkClient::call(
        &sock,
        "io.syntrop.Runtime1.TranscribeAudio",
        Some(params),
        GENERATE_RPC_TIMEOUT,
    )
    .await?;

    Ok(TranscribeAudioOutput {
        text: res
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        language: res
            .get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("en")
            .to_string(),
        duration_ms: res.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(0),
    })
}

/// Stream speech audio synthesized by Kokoro TTS to PipeWire or buffer.
pub async fn stream_audio_out(
    text: &str,
    voice: Option<&str>,
    sink_type: Option<&str>,
) -> Result<StreamAudioOutOutput, SyntropctlError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(SyntropctlError::OperationFailed("text cannot be empty".into()));
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
        "text": trimmed,
    });
    if let Some(v) = voice {
        params["voice"] = serde_json::json!(v);
    }
    if let Some(s) = sink_type {
        params["sink_type"] = serde_json::json!(s);
    }

    let res = VarlinkClient::call(
        &sock,
        "io.syntrop.Runtime1.StreamAudioOut",
        Some(params),
        GENERATE_RPC_TIMEOUT,
    )
    .await?;

    Ok(StreamAudioOutOutput {
        bytes_streamed: res.get("bytes_streamed").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        sample_rate: res.get("sample_rate").and_then(|v| v.as_u64()).unwrap_or(24000) as u32,
        channels: res.get("channels").and_then(|v| v.as_u64()).unwrap_or(1) as u32,
        pcm_base64: res.get("pcm_base64").and_then(|v| v.as_str()).map(ToString::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_output_structs() {
        let music = AudioGenerationOutput {
            audio_path: "/tmp/music.wav".into(),
            sample_rate: 32000,
            duration_ms: 5000,
        };
        assert_eq!(music.sample_rate, 32000);

        let stt = TranscribeAudioOutput {
            text: "test".into(),
            language: "en".into(),
            duration_ms: 1000,
        };
        assert_eq!(stt.text, "test");

        let stream = StreamAudioOutOutput {
            bytes_streamed: 4800,
            sample_rate: 24000,
            channels: 1,
            pcm_base64: Some("AAAA".into()),
        };
        assert_eq!(stream.bytes_streamed, 4800);
        assert_eq!(stream.sample_rate, 24000);
    }
}
