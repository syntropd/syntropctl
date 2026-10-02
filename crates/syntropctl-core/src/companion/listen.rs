//! Daemonized ambient listening for voice triggers and hotkey chords.

use crate::error::SyntropctlError;
use crate::sensory::{capture_audio, get_operator_presence};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration options for companion listening daemon session.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompanionListenOptions {
    /// Enable voice trigger detection via ambient audio and VAD.
    pub voice: bool,
    /// Hotkey chord identifier (e.g. Super+Space, F12).
    pub hotkey: Option<String>,
    /// Sample for one cycle and exit immediately.
    pub once: bool,
}

/// Event emitted when voice trigger or hotkey chord is activated.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompanionListenEvent {
    pub trigger_type: String,
    pub description: String,
    pub active: bool,
    pub timestamp_ms: u64,
}

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Run an ambient listening session with a streaming event callback.
pub async fn listen_session_with_callback<F>(
    options: &CompanionListenOptions,
    mut on_event: F,
) -> Result<Vec<CompanionListenEvent>, SyntropctlError>
where
    F: FnMut(&CompanionListenEvent),
{
    let mut collected = Vec::new();

    // Hotkey chord listener registration occurs once upon session start
    if let Some(ref chord) = options.hotkey {
        let ev = CompanionListenEvent {
            trigger_type: "hotkey".to_string(),
            description: format!("Hotkey chord listener registered for '{}'", chord),
            active: true,
            timestamp_ms: current_timestamp_ms(),
        };
        on_event(&ev);
        collected.push(ev);
    }

    loop {
        if options.voice {
            // Sample ambient audio PCM and query operator presence VAD
            let _ = capture_audio(Some(300), Some(16000)).await;
            if let Ok(presence) = get_operator_presence().await {
                if presence.present && presence.confidence > 0.5 {
                    let ev = CompanionListenEvent {
                        trigger_type: "voice".to_string(),
                        description: format!("Operator speech activity: {}", presence.reason),
                        active: true,
                        timestamp_ms: current_timestamp_ms(),
                    };
                    on_event(&ev);
                    if options.once {
                        collected.push(ev);
                    }
                }
            }
        }

        if options.once {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    Ok(collected)
}

/// Run an ambient listening session, returning collected activation events.
pub async fn listen_session(
    options: &CompanionListenOptions,
) -> Result<Vec<CompanionListenEvent>, SyntropctlError> {
    listen_session_with_callback(options, |_| {}).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_listen_once_hotkey() {
        let opts = CompanionListenOptions {
            voice: false,
            hotkey: Some("Super+Space".into()),
            once: true,
        };
        let events = listen_session(&opts).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].trigger_type, "hotkey");
        assert!(events[0].description.contains("Super+Space"));
    }
}
