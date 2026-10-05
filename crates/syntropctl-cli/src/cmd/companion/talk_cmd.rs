//! Full-duplex conversational voice companion loop CLI command.

use anyhow::Result;
use syntropctl_core::companion::{run_talk_session, TalkOptions};

/// Execute `companion talk` interactive or one-shot voice loop.
pub async fn handle_talk(
    model: Option<&str>,
    voice: Option<&str>,
    once: bool,
    json: bool,
) -> Result<()> {
    let options = TalkOptions {
        model: model.map(ToString::to_string),
        voice: voice.map(ToString::to_string),
        once,
    };

    if !json {
        println!("Starting full-duplex conversational voice companion...");
        println!("Voice: {}", options.voice.as_deref().unwrap_or("af_bella"));
        println!("Listening for speech (PipeWire VAD) — speak to interact, press Ctrl+C to exit.");
    }

    run_talk_session(&options, |event| {
        if json {
            if let Ok(serialized) = serde_json::to_string(event) {
                println!("{serialized}");
            }
        } else {
            println!("User: \"{}\"", event.prompt);
            println!("Companion: \"{}\"", event.reply);
        }
    })
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_talk_options() {
        let opts = TalkOptions {
            model: Some("clef-flash".into()),
            voice: Some("af_bella".into()),
            once: true,
        };
        assert_eq!(opts.voice.as_deref(), Some("af_bella"));
        assert!(opts.once);
    }
}
