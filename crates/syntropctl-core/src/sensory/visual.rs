//! Visual frame and desktop screen capture operations.

use base64::Engine as _;
use serde::{Deserialize, Serialize};

use super::endpoint::{connect_check, sensory_socket_path};
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};

/// Captured video or screen frame result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageCaptureResult {
    pub image_base64: String,
    pub format: String,
    #[serde(skip_serializing, default)]
    pub raw_bytes: Vec<u8>,
}

/// Capture video frame via V4L2 device.
pub async fn capture_frame(
    device: Option<&str>,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<ImageCaptureResult, SyntropctlError> {
    let sock = sensory_socket_path();
    connect_check(&sock)?;

    let mut params = serde_json::Map::new();
    if let Some(dev) = device {
        params.insert("device".to_string(), serde_json::json!(dev));
    }
    if let Some(w) = width {
        params.insert("width".to_string(), serde_json::json!(w));
    }
    if let Some(h) = height {
        params.insert("height".to_string(), serde_json::json!(h));
    }

    let p = if params.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(params))
    };
    let res = VarlinkClient::call(
        &sock,
        "io.syntrop.Sensory1.CaptureFrame",
        p,
        DEFAULT_RPC_TIMEOUT,
    )
    .await?;
    parse_image_result(&res, "CaptureFrame")
}

/// Capture desktop screen image.
pub async fn capture_screen(display: Option<&str>) -> Result<ImageCaptureResult, SyntropctlError> {
    let sock = sensory_socket_path();
    connect_check(&sock)?;

    let p = display.map(|d| serde_json::json!({ "display": d }));
    let res = VarlinkClient::call(
        &sock,
        "io.syntrop.Sensory1.CaptureScreen",
        p,
        DEFAULT_RPC_TIMEOUT,
    )
    .await?;
    parse_image_result(&res, "CaptureScreen")
}

pub(crate) fn parse_image_result(
    res: &serde_json::Value,
    op: &str,
) -> Result<ImageCaptureResult, SyntropctlError> {
    let b64 = res
        .get("image_base64")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            SyntropctlError::MalformedReply(format!("Missing image_base64 in {op} reply"))
        })?;
    let fmt = res
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("png")
        .to_string();

    let raw = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| SyntropctlError::MalformedReply(format!("Invalid base64 image: {e}")))?;

    Ok(ImageCaptureResult {
        image_base64: b64.to_string(),
        format: fmt,
        raw_bytes: raw,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_image_result_valid() {
        let valid_png_b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAACklEQVR4nGNiAAAABgADNjd8qAAAAABJRU5ErkJggg==";
        let val = serde_json::json!({
            "image_base64": valid_png_b64,
            "format": "png",
        });
        let res = parse_image_result(&val, "CaptureFrame").expect("should parse");
        assert_eq!(res.format, "png");
        assert!(!res.raw_bytes.is_empty());
    }

    #[test]
    fn test_parse_image_result_invalid_base64() {
        let val = serde_json::json!({
            "image_base64": "!bad base64!",
            "format": "png",
        });
        let res = parse_image_result(&val, "CaptureFrame");
        assert!(res.is_err());
    }
}
