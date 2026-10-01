//! Core library for the syntropctl unified administration CLI.
//!
//! Provides pure Rust Varlink client communication, daemon discovery,
//! and operations targeting the syntropd AI subsystem daemons.

pub mod daemon;
pub mod error;
pub mod ops;
pub mod sensory;
pub mod varlink;

pub use daemon::{DaemonEndpoint, DaemonKind, DAEMONS};
pub use error::SyntropctlError;
pub use sensory::capture_audio;
pub use sensory::capture_frame;
pub use sensory::capture_screen;
pub use sensory::get_operator_presence;
pub use sensory::sensory_socket_path;
pub use sensory::AudioCaptureResult;
pub use sensory::ImageCaptureResult;
pub use sensory::OperatorPresenceResult;
pub use varlink::VarlinkClient;
