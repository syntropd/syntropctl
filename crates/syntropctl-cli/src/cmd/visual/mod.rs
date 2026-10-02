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
    },
}
