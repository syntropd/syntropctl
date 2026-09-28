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

#[cfg(test)]
mod tests {
    use super::*;

    fn json_error() -> serde_json::Error {
        serde_json::from_str::<serde_json::Value>("{oops").unwrap_err()
    }

    #[test]
    fn display_names_each_failure_mode() {
        let io = SyntropctlError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "gone"));
        assert_eq!(io.to_string(), "I/O error: gone");

        let json = SyntropctlError::Json(json_error());
        assert!(json.to_string().starts_with("JSON error: "), "{json}");

        let down = SyntropctlError::DaemonUnavailable {
            daemon: "toold".to_string(),
            socket: PathBuf::from("/run/syntrop/io.syntrop.Tool1"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "missing"),
        };
        assert_eq!(
            down.to_string(),
            "Daemon 'toold' is unavailable at /run/syntrop/io.syntrop.Tool1: missing"
        );

        let proto = SyntropctlError::ProtocolError {
            error: "io.syntrop.Model1.ModelNotFound".to_string(),
            parameters: Some(serde_json::json!({"model": "m"})),
        };
        assert!(proto.to_string().contains("io.syntrop.Model1.ModelNotFound"), "{proto}");

        assert_eq!(
            SyntropctlError::MalformedReply("empty".to_string()).to_string(),
            "Malformed Varlink reply: empty"
        );
        assert_eq!(
            SyntropctlError::OperationFailed("boom".to_string()).to_string(),
            "Operation failed: boom"
        );
        assert_eq!(
            SyntropctlError::NotFound("runtimed".to_string()).to_string(),
            "Not found: runtimed"
        );
    }

    #[test]
    fn io_and_json_convert_with_from() {
        let io: SyntropctlError =
            std::io::Error::new(std::io::ErrorKind::TimedOut, "slow").into();
        assert!(matches!(io, SyntropctlError::Io(_)));
        let json: SyntropctlError = json_error().into();
        assert!(matches!(json, SyntropctlError::Json(_)));
    }
}
