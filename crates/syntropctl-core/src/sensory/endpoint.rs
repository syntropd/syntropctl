//! Sensory socket resolution and connectivity checks.

use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use std::path::{Path, PathBuf};

/// Resolve the Sensory1 Unix domain socket path.
pub fn sensory_socket_path() -> PathBuf {
    if let Ok(val) = std::env::var("SYNTROP_SENSORY_SOCKET") {
        if !val.trim().is_empty() {
            return PathBuf::from(val);
        }
    }
    let default_sock = PathBuf::from("/run/syntrop/io.syntrop.Sensory1");
    if default_sock.exists() {
        return default_sock;
    }
    if let Some(runtimed) = DaemonEndpoint::from_name("runtimed") {
        let runtimed_sock = runtimed.socket_path();
        if runtimed_sock.exists() {
            return runtimed_sock;
        }
    }
    default_sock
}

pub(crate) fn connect_check(sock: &Path) -> Result<(), SyntropctlError> {
    if !sock.exists() {
        return Err(SyntropctlError::DaemonUnavailable {
            daemon: "sensory".into(),
            socket: sock.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "Socket file does not exist"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensory_socket_resolution_with_env() {
        let custom = "/tmp/test-sensory.sock";
        std::env::set_var("SYNTROP_SENSORY_SOCKET", custom);
        assert_eq!(sensory_socket_path(), PathBuf::from(custom));
        std::env::remove_var("SYNTROP_SENSORY_SOCKET");
    }
}
