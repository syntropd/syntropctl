//! Varlink client for io.syntrop.Actuator1 virtual HID controls.

use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

/// High-level UI actuator action command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ActuatorAction {
    /// Move mouse cursor to absolute normalized coordinates [0.0, 1.0].
    MoveMouse { x: f32, y: f32 },
    /// Click virtual mouse button (e.g. 1=Left, 2=Right, 3=Middle, 272=BTN_LEFT).
    Click { button: u16 },
    /// Type UTF-8 string into focused application.
    TypeText { text: String },
    /// Send low-level evdev key event (code and down state).
    SendKey { key_code: u16, down: bool },
}

/// Resolve the Actuator1 Varlink Unix domain socket path.
pub fn actuator_socket_path() -> PathBuf {
    if let Ok(val) = std::env::var("SYNTROP_ACTUATOR_SOCKET") {
        if !val.trim().is_empty() {
            return PathBuf::from(val);
        }
    }
    let default_sock = PathBuf::from("/run/syntrop/io.syntrop.Actuator1");
    if default_sock.exists() {
        return default_sock;
    }
    let tool_sock = PathBuf::from("/run/syntrop/io.syntrop.Tool1");
    if tool_sock.exists() {
        return tool_sock;
    }
    default_sock
}

/// Send a low-level key press or release event via io.syntrop.Actuator1.
pub async fn send_key(key_code: u16, down: bool) -> Result<(), SyntropctlError> {
    let sock = actuator_socket_path();
    let params = json!({
        "key_code": key_code,
        "down": down,
    });
    VarlinkClient::call(&sock, "io.syntrop.Actuator1.SendKey", Some(params), DEFAULT_RPC_TIMEOUT).await?;
    Ok(())
}

/// Type a UTF-8 text string sequentially via io.syntrop.Actuator1.
pub async fn type_text(text: &str) -> Result<(), SyntropctlError> {
    let sock = actuator_socket_path();
    let params = json!({ "text": text });
    VarlinkClient::call(&sock, "io.syntrop.Actuator1.TypeText", Some(params), DEFAULT_RPC_TIMEOUT).await?;
    Ok(())
}

/// Move cursor to normalized absolute desktop coordinates (0.0..=1.0).
pub async fn move_mouse_abs(x: f32, y: f32) -> Result<(), SyntropctlError> {
    let sock = actuator_socket_path();
    let clamped_x = x.clamp(0.0, 1.0);
    let clamped_y = y.clamp(0.0, 1.0);
    let params = json!({
        "x": clamped_x,
        "y": clamped_y,
    });
    VarlinkClient::call(&sock, "io.syntrop.Actuator1.MoveMouseAbs", Some(params), DEFAULT_RPC_TIMEOUT).await?;
    Ok(())
}

/// Emit mouse button press and release click event.
pub async fn click_mouse(button: u16) -> Result<(), SyntropctlError> {
    let sock = actuator_socket_path();
    let params = json!({ "button": button });
    VarlinkClient::call(&sock, "io.syntrop.Actuator1.ClickMouse", Some(params), DEFAULT_RPC_TIMEOUT).await?;
    Ok(())
}

/// Dispatch a generic ActuatorAction enum variant.
pub async fn dispatch_action(action: &ActuatorAction) -> Result<(), SyntropctlError> {
    match action {
        ActuatorAction::MoveMouse { x, y } => move_mouse_abs(*x, *y).await,
        ActuatorAction::Click { button } => click_mouse(*button).await,
        ActuatorAction::TypeText { text } => type_text(text).await,
        ActuatorAction::SendKey { key_code, down } => send_key(*key_code, *down).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actuator_socket_resolution_override() {
        let custom = "/tmp/test-actuator.sock";
        std::env::set_var("SYNTROP_ACTUATOR_SOCKET", custom);
        assert_eq!(actuator_socket_path(), PathBuf::from(custom));
        std::env::remove_var("SYNTROP_ACTUATOR_SOCKET");
    }

    #[test]
    fn test_actuator_action_serialization() {
        let action = ActuatorAction::MoveMouse { x: 0.5, y: 0.25 };
        let serialized = serde_json::to_string(&action).unwrap();
        assert!(serialized.contains("\"action\":\"move_mouse\""));
        assert!(serialized.contains("\"x\":0.5"));
        let deserialized: ActuatorAction = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, action);
    }
}
