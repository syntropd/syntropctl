//! Environmental sensing operations communicating with Sensory1 interface.

pub mod audio;
pub mod endpoint;
pub mod presence;
pub mod visual;

pub use audio::{capture_audio, AudioCaptureResult};
pub use endpoint::sensory_socket_path;
pub use presence::{get_operator_presence, OperatorPresenceResult};
pub use visual::{capture_frame, capture_screen, ImageCaptureResult};
