//! Edge tests for Varlink protocol violations and errors.

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use syntropctl_core::error::SyntropctlError;
    use syntropctl_core::varlink::VarlinkClient;
    use tempfile::tempdir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    #[tokio::test]
    async fn test_remote_varlink_error_propagated() {
        let dir = tempdir().unwrap();
        let sock_path = dir.path().join("error.sock");

        let listener = UnixListener::bind(&sock_path).unwrap();

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut byte = [0u8; 1];
            loop {
                stream.read_exact(&mut byte).await.unwrap();
                if byte[0] == 0 {
                    break;
                }
            }

            let reply = serde_json::json!({
                "error": "io.syntrop.Model1.ModelNotFound",
                "parameters": {
                    "model": "nonexistent"
                }
            });

            let mut reply_bytes = serde_json::to_vec(&reply).unwrap();
            reply_bytes.push(0);
            stream.write_all(&reply_bytes).await.unwrap();
        });

        let res = VarlinkClient::call(
            &sock_path,
            "io.syntrop.Model1.GetModel",
            Some(serde_json::json!({"model": "nonexistent"})),
            Duration::from_millis(500),
        )
        .await;

        match res {
            Err(SyntropctlError::ProtocolError { error, parameters }) => {
                assert_eq!(error, "io.syntrop.Model1.ModelNotFound");
                assert_eq!(parameters.unwrap()["model"], "nonexistent");
            }
            other => panic!("Expected ProtocolError, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_malformed_json_reply_fails_deserialization() {
        let dir = tempdir().unwrap();
        let sock_path = dir.path().join("bad_json.sock");

        let listener = UnixListener::bind(&sock_path).unwrap();

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut byte = [0u8; 1];
            loop {
                stream.read_exact(&mut byte).await.unwrap();
                if byte[0] == 0 {
                    break;
                }
            }

            let bad_bytes = b"NOT_VALID_JSON\0";
            stream.write_all(bad_bytes).await.unwrap();
        });

        let res = VarlinkClient::call(&sock_path, "test.Method", None, Duration::from_millis(500)).await;

        match res {
            Err(SyntropctlError::Json(_)) => (),
            other => panic!("Expected Json error, got {:?}", other),
        }
    }
}
