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

/// Capture the current screen frame and ask a multimodal question grounded in the visual context.
pub async fn ask_screen(
    prompt: &str,
    display: Option<&str>,
) -> Result<CompanionAskResult, SyntropctlError> {
    let screen = capture_screen(display).await?;

    let system_instructions = "You are a Linux Cognitive Desktop Companion. Inspect the provided screenshot and answer the user's question accurately with grounded visual references.";

    let answer = query_router_multimodal(prompt, &screen.image_base64, Some(system_instructions)).await?;

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
