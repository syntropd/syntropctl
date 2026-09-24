//! Unit tests for Varlink client and mock RPC execution.

#[cfg(test)]
mod tests {
    use syntropctl_core::varlink::{VarlinkCall, VarlinkClient, VarlinkReply};
    use tempfile::tempdir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    #[tokio::test]
    async fn test_varlink_call_and_reply_success() {
        let dir = tempdir().unwrap();
        let sock_path = dir.path().join("test_varlink.sock");

        let listener = UnixListener::bind(&sock_path).unwrap();

        let server_task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = Vec::new();
            let mut byte = [0u8; 1];
            loop {
                stream.read_exact(&mut byte).await.unwrap();
                if byte[0] == 0 {
                    break;
                }
                buf.push(byte[0]);
            }

            let req: serde_json::Value = serde_json::from_slice(&buf).unwrap();
            assert_eq!(req["method"], "org.varlink.service.GetInfo");

            let reply = serde_json::json!({
                "parameters": {
                    "vendor": "Syntropd Project",
                    "product": "testd",
                    "version": "1.0.0",
                    "url": "https://github.com/syntropd",
                    "interfaces": ["org.varlink.service", "io.syntrop.Test1"]
                }
            });

            let mut reply_bytes = serde_json::to_vec(&reply).unwrap();
            reply_bytes.push(0);
            stream.write_all(&reply_bytes).await.unwrap();
        });

        let info = VarlinkClient::get_info(&sock_path).await.unwrap();
        assert_eq!(info.vendor, "Syntropd Project");
        assert_eq!(info.product, "testd");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.interfaces.len(), 2);

        server_task.await.unwrap();
    }

    #[test]
    fn test_varlink_call_serialization() {
        let call = VarlinkCall {
            method: "io.syntrop.Tool1.Execute",
            parameters: Some(serde_json::json!({"command": "uname"})),
        };

        let json = serde_json::to_string(&call).unwrap();
        assert!(json.contains("\"method\":\"io.syntrop.Tool1.Execute\""));
        assert!(json.contains("\"command\":\"uname\""));
    }

    #[test]
    fn test_varlink_reply_deserialization() {
        let json_str = r#"{"parameters":{"exit_code":0},"error":null}"#;
        let reply: VarlinkReply = serde_json::from_str(json_str).unwrap();
        assert!(reply.error.is_none());
        assert_eq!(reply.parameters.unwrap()["exit_code"], 0);
    }
}
