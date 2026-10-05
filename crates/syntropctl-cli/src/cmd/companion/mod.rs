//! Linux Cognitive Desktop Companion CLI command suite.

pub mod ask_cmd;
pub mod execute_cmd;
pub mod listen_cmd;
pub mod talk_cmd;

pub use ask_cmd::handle_ask;
pub use execute_cmd::handle_execute;
pub use listen_cmd::handle_listen;
pub use talk_cmd::handle_talk;

use clap::Subcommand;
use std::process::ExitCode;

/// Linux Cognitive Desktop Companion subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum CompanionCommands {
    /// Captures screen frame, queries routerd with visual context, and renders grounded answer.
    Ask {
        /// Grounded question or prompt regarding current screen content.
        prompt: String,

        /// Optional display identifier (e.g. :0, wayland-0).
        #[arg(long = "display")]
        display: Option<String>,
    },

    /// Plans UI actions from screen, executes mouse/keyboard commands via Actuator1, and validates state.
    Execute {
        /// High-level natural language instruction or UI action plan.
        instruction: String,

        /// Optional display identifier (e.g. :0, wayland-0).
        #[arg(long = "display")]
        display: Option<String>,

        /// Validate and preview action plan without emitting hardware actuator events.
        #[arg(long = "dry-run")]
        dry_run: bool,

        /// Enable visual grounding for semantic UI targeting.
        #[arg(long = "grounding", default_value_t = true)]
        grounding: bool,
    },

    /// Daemonized session listening for voice triggers or hotkey chords.
    Listen {
        /// Listen for voice commands via ambient audio and VAD.
        #[arg(long = "voice")]
        voice: bool,

        /// Hotkey chord identifier (e.g. Super+Space, F12).
        #[arg(long = "hotkey")]
        hotkey: Option<String>,

        /// Run a single listen cycle and exit (useful for testing or one-shot hooks).
        #[arg(long = "once")]
        once: bool,
    },

    /// Full-duplex conversational voice loop: PipeWire VAD -> Whisper STT -> LLM -> Kokoro TTS.
    Talk {
        /// LLM reasoning model to route conversational queries to.
        #[arg(long = "model")]
        model: Option<String>,

        /// Voice persona identifier for Kokoro TTS audio synthesis.
        #[arg(long = "voice")]
        voice: Option<String>,

        /// Run a single conversational exchange and exit.
        #[arg(long = "once")]
        once: bool,
    },
}

/// Dispatches all `syn companion` / `syntropctl companion` subcommands.
pub async fn handle_companion(command: CompanionCommands, json: bool) -> anyhow::Result<ExitCode> {
    match command {
        CompanionCommands::Ask { prompt, display } => {
            handle_ask(&prompt, display.as_deref(), json).await?;
            Ok(ExitCode::SUCCESS)
        }
        CompanionCommands::Execute {
            instruction,
            display,
            dry_run,
            grounding,
        } => {
            handle_execute(&instruction, display.as_deref(), dry_run, grounding, json).await?;
            Ok(ExitCode::SUCCESS)
        }
        CompanionCommands::Listen { voice, hotkey, once } => {
            handle_listen(voice, hotkey.as_deref(), once, json).await?;
            Ok(ExitCode::SUCCESS)
        }
        CompanionCommands::Talk { model, voice, once } => {
            handle_talk(model.as_deref(), voice.as_deref(), once, json).await?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_companion_commands_ask_variant() {
        let cmd = CompanionCommands::Ask {
            prompt: "What is this?".to_string(),
            display: None,
        };
        assert!(matches!(cmd, CompanionCommands::Ask { .. }));
    }
}
