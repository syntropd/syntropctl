//! Generative visual commands.

pub mod generate;

use clap::Subcommand;
pub use generate::handle_visual_generate;

/// Subcommands for generative visual operations.
#[derive(Subcommand, Debug)]
pub enum VisualCommands {
    /// Generate an image from a prompt.
    Generate {
        /// Input text prompt describing the image.
        prompt: String,

        /// Model identifier to invoke.
        #[arg(short = 'm', long = "model")]
        model: Option<String>,

        /// LoRA adapter spec in name:weight format.
        #[arg(short = 'l', long = "lora")]
        lora: Option<String>,

        /// Target dimensions in WxH format (e.g. 512x512).
        #[arg(short = 's', long = "size")]
        size: Option<String>,

        /// Generate keyframe storyboard strip instead of single high-res image.
        #[arg(long = "storyboard")]
        storyboard: Option<usize>,

        /// Allow graceful degradation to CPU storyboard keyframes on zero-VRAM hardware.
        #[arg(long = "allow-degrade")]
        allow_degrade: bool,
    },
}
