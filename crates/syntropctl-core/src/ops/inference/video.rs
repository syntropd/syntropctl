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
}

/// Execute generative video synthesis via runtimed.
pub async fn generate_video(
    prompt: &str,
    frames: usize,
    fps: u32,
) -> Result<VideoGenerationOutput, SyntropctlError> {
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
        "prompt": prompt,
        "frames": frames,
        "fps": fps,
    });

    let res = VarlinkClient::call(
        &sock,
        "io.syntrop.Runtime1.GenerateVideo",
        Some(params),
        GENERATE_RPC_TIMEOUT,
    )
    .await?;

    Ok(VideoGenerationOutput {
        video_path: res
            .get("video_path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        frames: res.get("frames").and_then(|v| v.as_u64()).unwrap_or(frames as u64) as usize,
        duration_ms: res.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(0),
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
        };
        assert_eq!(vid.frames, 16);
        assert_eq!(vid.duration_ms, 2000);
    }
}
