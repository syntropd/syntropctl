//! Full-duplex ambient conversational loop: PipeWire VAD -> STT -> LLM -> Kokoro TTS -> pw-cat.

use crate::error::SyntropctlError;
use crate::sensory::{capture_audio, get_operator_presence};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Configuration options for real-time full-duplex conversational voice loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalkOptions {
    /// LLM reasoning model (default "clef-flash" / reflex).
    pub model: Option<String>,
    /// Kokoro-82M TTS voice persona (default "af_bella").
    pub voice: Option<String>,
    /// Execute one conversational turn and exit.
    pub once: bool,
}

impl Default for TalkOptions {
    fn default() -> Self {
        Self {
            model: Some("clef-flash".to_string()),
            voice: Some("af_bella".to_string()),
            once: false,
        }
    }
}

/// Transcribed speech or assistant spoken response event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TalkTranscriptEvent {
    pub prompt: String,
    pub reply: String,
    pub barged_in: bool,
}

/// Run a full-duplex conversational voice session with streaming event callback.
pub async fn run_talk_session<F>(
    options: &TalkOptions,
    mut on_transcript: F,
) -> Result<(), SyntropctlError>
where
    F: FnMut(&TalkTranscriptEvent),
{
    let voice = options.voice.as_deref().unwrap_or("af_bella");
    let active_playback = Arc::new(AtomicBool::new(false));

    loop {
        // 1. Listen & VAD via Sensory1.CaptureAudio
        let audio_opt = capture_audio(Some(350), Some(16000)).await.ok();
        let presence = get_operator_presence().await.unwrap_or(crate::sensory::OperatorPresenceResult {
            present: false,
            confidence: 0.0,
            reason: "no activity".into(),
        });

        // Barge-in check: new speech or physical user input immediately halts ongoing playback
        if (presence.present || super::safety::check_physical_user_input())
            && active_playback.load(Ordering::SeqCst)
        {
            let _ = Command::new("pkill").args(["-f", "pw-cat"]).status();
            active_playback.store(false, Ordering::SeqCst);
        }

        if presence.present && presence.confidence > 0.45 {
            if let Some(ref a) = audio_opt {
                // 2. Transcribe via local Whisper STT
                if let Ok(stt) = crate::ops::inference::transcribe_audio(&a.audio_pcm_base64, Some("en")).await {
                    let text = stt.text.trim().to_string();
                    if !text.is_empty() {
                        // 3. Reason via routerd System 1 reflex
                        let system = "You are a real-time sovereign voice companion. Provide concise, direct 1-2 sentence spoken answers without markdown formatting.";
                        let reply = super::router_stream::query_router_text(&text, Some(system))
                            .await
                            .unwrap_or_else(|_| "Understood.".to_string());

                        let event = TalkTranscriptEvent {
                            prompt: text,
                            reply: reply.clone(),
                            barged_in: false,
                        };
                        on_transcript(&event);

                        // 4. Stream completion sentences to Kokoro-82M TTS and PipeWire pw-cat
                        active_playback.store(true, Ordering::SeqCst);
                        let _ = crate::ops::inference::stream_audio_out(&reply, Some(voice), Some("auto")).await;
                        active_playback.store(false, Ordering::SeqCst);

                        if options.once {
                            break;
                        }
                    }
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(40)).await;
        if options.once && presence.present {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_talk_options_default() {
        let opts = TalkOptions::default();
        assert_eq!(opts.voice.as_deref(), Some("af_bella"));
        assert!(!opts.once);
    }

    #[test]
    fn test_talk_transcript_event_serde() {
        let ev = TalkTranscriptEvent {
            prompt: "What is the CPU temp?".into(),
            reply: "CPU temperature is 42 degrees Celsius.".into(),
            barged_in: false,
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: TalkTranscriptEvent = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }
}
