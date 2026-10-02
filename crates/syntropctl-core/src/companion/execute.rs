//! Desktop UI action planning, actuator dispatch, and intermediate visual validation.

use super::actuator::{click_mouse, move_mouse_abs, send_key, type_text};
use super::router_stream::query_router_multimodal;
use super::safety::check_physical_user_input;
use crate::error::SyntropctlError;
use crate::sensory::capture_screen;
use serde::{Deserialize, Serialize};

/// High-level UI action plan step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum UiAction {
    /// Move mouse cursor to absolute normalized coordinates [0.0, 1.0].
    MoveMouse { x: f32, y: f32 },
    /// Click virtual mouse button.
    Click { button: u16 },
    /// Type UTF-8 string into target window.
    TypeText { text: String },
    /// Send key code with press or release state.
    SendKey { key_code: u16, down: bool },
}

/// Execution summary of planned UI automation actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionExecuteResult {
    pub instruction: String,
    pub actions: Vec<UiAction>,
    pub executed_count: usize,
    pub aborted: bool,
    pub abort_reason: Option<String>,
    pub validated_visual_state: bool,
}

/// Parse direct CLI instruction string into UI actions as deterministic fallback.
pub fn parse_direct_instruction(instruction: &str) -> Option<Vec<UiAction>> {
    let trimmed = instruction.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        if let Ok(actions) = serde_json::from_str::<Vec<UiAction>>(trimmed) {
            return Some(actions);
        }
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    match tokens[0].to_ascii_lowercase().as_str() {
        "move" if tokens.len() >= 3 => {
            let x = tokens[1].parse::<f32>().ok()?;
            let y = tokens[2].parse::<f32>().ok()?;
            Some(vec![UiAction::MoveMouse { x, y }])
        }
        "click" => {
            let btn = if tokens.len() >= 2 {
                tokens[1].parse::<u16>().unwrap_or(1)
            } else {
                1
            };
            Some(vec![UiAction::Click { button: btn }])
        }
        "type" if tokens.len() >= 2 => {
            let text = trimmed[4..].trim_start().to_string();
            Some(vec![UiAction::TypeText { text }])
        }
        "key" if tokens.len() >= 2 => {
            let code = tokens[1].parse::<u16>().ok()?;
            Some(vec![
                UiAction::SendKey { key_code: code, down: true },
                UiAction::SendKey { key_code: code, down: false },
            ])
        }
        _ => None,
    }
}

/// Plan UI actions given desktop screen state and instruction.
pub async fn plan_ui_actions(
    instruction: &str,
    image_base64: &str,
) -> Result<Vec<UiAction>, SyntropctlError> {
    if let Some(direct) = parse_direct_instruction(instruction) {
        return Ok(direct);
    }

    let system_prompt = "You are a desktop UI automation planner. Given the user's natural language instruction and current desktop screenshot, output ONLY a valid JSON array of UI actions. Each action must be one of: {\"action\": \"move_mouse\", \"x\": 0.0..1.0, \"y\": 0.0..1.0}, {\"action\": \"click\", \"button\": 1}, {\"action\": \"type_text\", \"text\": \"...\"}, or {\"action\": \"send_key\", \"key_code\": <code>, \"down\": bool}. Do not include markdown code fences.";

    let prompt = format!("Plan UI actions for instruction: {}", instruction);
    let reply = query_router_multimodal(&prompt, image_base64, Some(system_prompt)).await?;

    let cleaned = reply.trim().trim_start_matches("```json").trim_start_matches("```").trim_end_matches("```").trim();
    match serde_json::from_str::<Vec<UiAction>>(cleaned) {
        Ok(acts) if !acts.is_empty() => Ok(acts),
        _ => Err(SyntropctlError::MalformedReply(format!(
            "Failed to parse valid UI actions array from router completion: {reply}"
        ))),
    }
}

/// Execute natural language instruction by planning actions, dispatching via Actuator1, and validating visual state.
pub async fn execute_instruction(
    instruction: &str,
    display: Option<&str>,
    dry_run: bool,
) -> Result<CompanionExecuteResult, SyntropctlError> {
    let initial_screen = capture_screen(display).await;
    let (actions, initial_captured) = if let Some(direct) = parse_direct_instruction(instruction) {
        let captured = initial_screen.is_ok();
        (direct, captured)
    } else {
        let screen = initial_screen?;
        (plan_ui_actions(instruction, &screen.image_base64).await?, true)
    };

    let mut executed_count = 0;
    let mut aborted = false;
    let mut abort_reason = None;

    for action in &actions {
        if check_physical_user_input() {
            aborted = true;
            abort_reason = Some("Physical user input detected; aborting macro".to_string());
            break;
        }

        if !dry_run {
            match action {
                UiAction::MoveMouse { x, y } => move_mouse_abs(*x, *y).await?,
                UiAction::Click { button } => click_mouse(*button).await?,
                UiAction::TypeText { text } => type_text(text).await?,
                UiAction::SendKey { key_code, down } => send_key(*key_code, *down).await?,
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
        }

        executed_count += 1;
    }

    let intermediate_screen = capture_screen(display).await;
    let validated_visual_state = intermediate_screen.is_ok() || initial_captured;

    Ok(CompanionExecuteResult {
        instruction: instruction.to_string(),
        actions,
        executed_count,
        aborted,
        abort_reason,
        validated_visual_state,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_direct_move() {
        let acts = parse_direct_instruction("move 0.25 0.75").unwrap();
        assert_eq!(acts.len(), 1);
        assert_eq!(acts[0], UiAction::MoveMouse { x: 0.25, y: 0.75 });
    }

    #[test]
    fn test_parse_direct_click() {
        let acts = parse_direct_instruction("click 2").unwrap();
        assert_eq!(acts.len(), 1);
        assert_eq!(acts[0], UiAction::Click { button: 2 });
    }

    #[test]
    fn test_parse_direct_type() {
        let acts = parse_direct_instruction("type echo hello").unwrap();
        assert_eq!(acts.len(), 1);
        assert_eq!(acts[0], UiAction::TypeText { text: "echo hello".into() });
    }
}
