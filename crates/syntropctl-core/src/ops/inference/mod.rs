//! Multimodal inference operations communicating with runtimed.

pub mod audio;
pub mod text;
pub mod video;
pub mod visual;

pub use audio::{generate_music, transcribe_audio, AudioGenerationOutput, TranscribeAudioOutput};
pub use text::{embed_text, generate_text, GenerationOutput};
pub use video::{generate_video, VideoGenerationOutput};
pub use visual::{generate_visual, VisualGenerationOutput};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_reexports() {
        let text_out = GenerationOutput {
            text: "ok".into(),
            prompt_tokens: 1,
            completion_tokens: 1,
            finish_reason: "stop".into(),
            duration_ms: 10,
        };
        assert_eq!(text_out.text, "ok");

        let visual_out = VisualGenerationOutput {
            image_path: "/tmp/img.png".into(),
            bytes: 512,
            width: 64,
            height: 64,
            format: "png".into(),
        };
        assert_eq!(visual_out.width, 64);
    }
}
