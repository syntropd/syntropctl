//! Generative music and acoustic synthesis commands.

pub mod generate;

use clap::Subcommand;
pub use generate::handle_audio_generate;

/// Subcommands for acoustic music and audio operations.
#[derive(Subcommand, Debug)]
pub enum AudioCommands {
    /// Generate a music audio track from a descriptive prompt.
    Generate {
        /// Input text prompt describing the music.
        prompt: String,

        /// Duration of the audio clip in seconds.
        #[arg(short = 'd', long = "duration")]
        duration: Option<u32>,

        /// Beats per minute tempo.
        #[arg(short = 'b', long = "bpm")]
        bpm: Option<u32>,
    },
}
