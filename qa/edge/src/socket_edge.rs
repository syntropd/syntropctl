//! Edge tests for socket connectivity and communication failures.

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;
    use syntropctl_core::error::SyntropctlError;
    use syntropctl_core::varlink::VarlinkClient;
    use tempfile::tempdir;
    use tokio::net::UnixListener;

    #[tokio::test]
    async fn test_nonexistent_socket_returns_daemon_unavailable() {
        let path = PathBuf::from("/tmp/nonexistent_socket_file_12345.sock");
        let res = VarlinkClient::call(&path, "io.syntrop.Test1.Ping", None, Duration::from_millis(100)).await;

        match res {
            Err(SyntropctlError::DaemonUnavailable { daemon, socket, .. }) => {
                assert_eq!(daemon, "io");
                assert_eq!(socket, path);
            }
            other => panic!("Expected DaemonUnavailable, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_socket_immediate_eof_returns_malformed_reply() {
        let dir = tempdir().unwrap();
        let sock_path = dir.path().join("eof.sock");

        let listener = UnixListener::bind(&sock_path).unwrap();

        let server_task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            use tokio::io::AsyncReadExt;
            let mut byte = [0u8; 1];
            loop {
                let n = stream.read(&mut byte).await.unwrap_or(0);
                if n == 0 || byte[0] == 0 {
                    break;
                }
            }
            drop(stream);
        });

        let res = VarlinkClient::call(&sock_path, "org.varlink.service.GetInfo", None, Duration::from_millis(500)).await;

        match res {
            Err(SyntropctlError::MalformedReply(msg)) => {
                assert!(msg.contains("Empty reply buffer"));
            }
            Err(SyntropctlError::Io(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::ConnectionReset);
            }
            other => panic!("Expected MalformedReply or Io(ConnectionReset), got {:?}", other),
        }

        server_task.await.unwrap();
    }
}
