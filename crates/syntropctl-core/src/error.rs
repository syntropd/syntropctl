//! Error definitions for the syntropctl core library.

use std::path::PathBuf;
use thiserror::Error;

/// Error conditions encountered during syntropctl operations.
#[derive(Error, Debug)]
pub enum SyntropctlError {
    /// I/O error occurred while interacting with sockets or the filesystem.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization or deserialization failure.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Target daemon socket is unavailable or non-responsive.
    #[error("Daemon '{daemon}' is unavailable at {socket}: {source}")]
    DaemonUnavailable {
        daemon: String,
        socket: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Protocol error returned by remote Varlink daemon.
    #[error("Varlink error '{error}': {parameters:?}")]
    ProtocolError {
        error: String,
        parameters: Option<serde_json::Value>,
    },

    /// Response received from daemon was malformed or missing fields.
    #[error("Malformed Varlink reply: {0}")]
    MalformedReply(String),

    /// General operational failure within a subsystem.
    #[error("Operation failed: {0}")]
    OperationFailed(String),

    /// Requested resource or daemon was not found.
    #[error("Not found: {0}")]
    NotFound(String),
}
