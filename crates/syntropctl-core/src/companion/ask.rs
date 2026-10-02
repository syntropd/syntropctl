//! Grounded visual inspection and visual question answering over desktop screen.

use super::router_stream::query_router_multimodal;
use crate::error::SyntropctlError;
use crate::sensory::capture_screen;
use serde::{Deserialize, Serialize};

/// Grounded visual answer generated from current desktop screen context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionAskResult {
    pub prompt: String,
    pub answer: String,
    pub format: String,
    pub display: Option<String>,
}

/// Fast sub-50ms OCR text discovery via runtimed GroundVisual Varlink RPC.
pub async fn ground_screen_ocr(image_base64: &str) -> Option<String> {
    let sock = std::path::PathBuf::from(
        std::env::var("SYNTROP_RUNTIME_SOCKET").unwrap_or_else(|_| "/run/syntrop/io.syntrop.Runtime1".into()),
    );
    let params = serde_json::json!({
        "image_bytes": image_base64,
        "task": "<OCR>",
    });

    let res = crate::varlink::VarlinkClient::call(
        &sock,
        "io.syntrop.Runtime1.GroundVisual",
        Some(params),
        std::time::Duration::from_millis(150),
    )
    .await
    .ok()?;

    let text = res.get("text").and_then(|t| t.as_str())?;
    if text.trim().is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

/// Capture the current screen frame and ask a multimodal question grounded in the visual context.
pub async fn ask_screen(
    prompt: &str,
    display: Option<&str>,
) -> Result<CompanionAskResult, SyntropctlError> {
    let screen = capture_screen(display).await?;

    let lower = prompt.trim().to_ascii_lowercase();
    let is_ocr_query = lower.starts_with("ocr")
        || lower.starts_with("read")
        || lower.contains("read text")
        || lower.contains("what text");

    let answer = if is_ocr_query {
        if let Some(ocr_text) = ground_screen_ocr(&screen.image_base64).await {
            ocr_text
        } else {
            let system_instructions = "You are a Linux Cognitive Desktop Companion. Inspect the provided screenshot and answer the user's question accurately with grounded visual references.";
            query_router_multimodal(prompt, &screen.image_base64, Some(system_instructions)).await?
        }
    } else {
        let system_instructions = "You are a Linux Cognitive Desktop Companion. Inspect the provided screenshot and answer the user's question accurately with grounded visual references.";
        query_router_multimodal(prompt, &screen.image_base64, Some(system_instructions)).await?
    };

    Ok(CompanionAskResult {
        prompt: prompt.to_string(),
        answer,
        format: screen.format,
        display: display.map(ToString::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_companion_ask_result_serde() {
        let res = CompanionAskResult {
            prompt: "What window is open?".to_string(),
            answer: "A terminal window with bash prompt.".to_string(),
            format: "png".to_string(),
            display: Some(":0".to_string()),
        };
        let s = serde_json::to_string(&res).unwrap();
        let back: CompanionAskResult = serde_json::from_str(&s).unwrap();
        assert_eq!(back.prompt, res.prompt);
        assert_eq!(back.answer, res.answer);
    }
}
