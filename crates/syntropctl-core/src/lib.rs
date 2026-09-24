//! Core library for the syntropctl unified administration CLI.
//!
//! Provides pure Rust Varlink client communication, daemon discovery,
//! and operations targeting the syntropd AI subsystem daemons.

pub mod daemon;
pub mod error;
pub mod ops;
pub mod varlink;

pub use daemon::{DaemonEndpoint, DaemonKind, DAEMONS};
pub use error::SyntropctlError;
pub use varlink::VarlinkClient;
