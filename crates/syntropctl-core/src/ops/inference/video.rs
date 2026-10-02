//! Video generation operations communicating with runtimed.

use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, GENERATE_RPC_TIMEOUT};
use serde::{Deserialize, Serialize};

/// Result of video generation from runtimed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoGenerationOutput {
    pub video_path: String,
    pub frames: usize,
    pub duration_ms: u64,
    pub storyboard_path: Option<String>,
    pub manifest_path: Option<String>,
    pub format: Option<String>,
    pub keyframes: Option<usize>,
}

/// Execute generative video synthesis via runtimed.
pub async fn generate_video(
    prompt: &str,
    frames: usize,
    fps: u32,
    storyboard: Option<usize>,
    allow_degrade: bool,
) -> Result<VideoGenerationOutput, SyntropctlError> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        return Err(SyntropctlError::OperationFailed("prompt cannot be empty".into()));
    }
    if frames == 0 {
        return Err(SyntropctlError::OperationFailed("frames must be greater than 0".into()));
    }
    if fps == 0 {
        return Err(SyntropctlError::OperationFailed("fps must be greater than 0".into()));
    }
    if let Some(sb) = storyboard {
        if sb == 0 {
            return Err(SyntropctlError::OperationFailed(
                "storyboard keyframe count must be greater than 0".into(),
            ));
        }
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
        "frames": frames,
        "fps": fps,
        "allow_degrade": allow_degrade,
    });
    if let Some(sb) = storyboard {
        params["storyboard"] = serde_json::json!(sb);
    }

    let res = match VarlinkClient::call(
        &sock,
        "io.syntrop.Runtime1.GenerateVideo",
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

    Ok(VideoGenerationOutput {
        video_path: res
            .get("video_path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        frames: res.get("frames").and_then(|v| v.as_u64()).unwrap_or(frames as u64) as usize,
        duration_ms: res.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(0),
        storyboard_path: res.get("storyboard_path").and_then(|v| v.as_str()).map(String::from),
        manifest_path: res.get("manifest_path").and_then(|v| v.as_str()).map(String::from),
        format: res.get("format").and_then(|v| v.as_str()).map(String::from),
        keyframes: res.get("keyframes").and_then(|v| v.as_u64()).map(|k| k as usize),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_output_struct() {
        let vid = VideoGenerationOutput {
            video_path: "/tmp/video.mp4".into(),
            frames: 16,
            duration_ms: 2000,
            storyboard_path: None,
            manifest_path: None,
            format: None,
            keyframes: None,
        };
        assert_eq!(vid.frames, 16);
        assert_eq!(vid.duration_ms, 2000);
    }

    #[tokio::test]
    async fn test_generate_video_zero_storyboard() {
        let res = generate_video("test", 16, 8, Some(0), false).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("storyboard keyframe count must be greater than 0"));
    }
}
