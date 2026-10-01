//! Sensory subcommands shim for ambient audio, frame, screen, and presence.

pub mod audio_cmd;
pub mod frame_cmd;
pub mod presence_cmd;
pub mod screen_cmd;

pub use audio_cmd::handle_audio;
pub use frame_cmd::handle_frame;
pub use presence_cmd::handle_presence;
pub use screen_cmd::handle_screen;
