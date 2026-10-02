//! Generative video commands.

pub mod generate;

use clap::Subcommand;
pub use generate::handle_video_generate;

/// Subcommands for short-form video generation operations.
#[derive(Subcommand, Debug)]
pub enum VideoCommands {
    /// Generate a short animated video clip from a prompt.
    Generate {
        /// Input text prompt describing the animation.
        prompt: String,

        /// Total count of frames to generate.
        #[arg(short = 'f', long = "frames")]
        frames: Option<usize>,

        /// Frame playback rate in frames per second.
        #[arg(long = "fps")]
        fps: Option<u32>,

        /// Generate keyframe storyboard strip instead of full temporal video.
        #[arg(long = "storyboard")]
        storyboard: Option<usize>,

        /// Allow graceful degradation to CPU storyboard keyframes on zero-VRAM hardware.
        #[arg(long = "allow-degrade")]
        allow_degrade: bool,
    },
}
