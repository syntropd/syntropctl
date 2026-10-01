//! Environmental sensing operations communicating with Sensory1 interface.

pub mod ops;

pub use ops::capture_audio;
pub use ops::capture_frame;
pub use ops::capture_screen;
pub use ops::get_operator_presence;
pub use ops::sensory_socket_path;
pub use ops::AudioCaptureResult;
pub use ops::ImageCaptureResult;
pub use ops::OperatorPresenceResult;
