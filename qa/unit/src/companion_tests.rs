//! Unit tests for companion core operations: sensory/router streaming, actuator dispatch, and safety abort.

#[cfg(test)]
pub mod tests {
    use serde_json::json;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    use syntropctl_core::companion::*;

    #[test]
    fn test_companion_parse_direct_instructions() {
        let mv = parse_direct_instruction("move 0.1 0.9").unwrap();
        assert_eq!(mv, vec![UiAction::MoveMouse { x: 0.1, y: 0.9 }]);

        let clk = parse_direct_instruction("click 3").unwrap();
        assert_eq!(clk, vec![UiAction::Click { button: 3 }]);

        let typ = parse_direct_instruction("type uname -a").unwrap();
        assert_eq!(typ, vec![UiAction::TypeText { text: "uname -a".into() }]);

        let k = parse_direct_instruction("key 15").unwrap();
        assert_eq!(k.len(), 2);
        assert_eq!(k[0], UiAction::SendKey { key_code: 15, down: true });
        assert_eq!(k[1], UiAction::SendKey { key_code: 15, down: false });
    }

    #[tokio::test]
    async fn test_mock_router_multimodal_stream() {
        let temp_dir = std::env::temp_dir();
        let sock_path = temp_dir.join(format!("test-router-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&sock_path);

        let listener = UnixListener::bind(&sock_path).expect("bind mock router socket");
        std::env::set_var("SYNTROP_ROUTER_SOCKET", &sock_path);

        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 4096];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let req_text = String::from_utf8_lossy(&buf[..n]);
                assert!(req_text.contains("POST /v1/chat/completions"));
                assert!(req_text.contains("data:image/png;base64,"));

                let reply_body = json!({
                    "choices": [{
                        "message": { "content": "Grounded answer from mock routerd." }
                    }]
                });
                let reply_str = serde_json::to_string(&reply_body).unwrap_or_default();
                let http_reply = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    reply_str.len(),
                    reply_str
                );
                let _ = stream.write_all(http_reply.as_bytes()).await;
            }
        });

        let ans = query_router_multimodal("Describe screen", "iVBORw0KGgoAAAANSUhEUg==", None)
            .await
            .expect("multimodal query failed");
        assert_eq!(ans, "Grounded answer from mock routerd.");

        std::env::remove_var("SYNTROP_ROUTER_SOCKET");
        let _ = std::fs::remove_file(&sock_path);
    }

    #[tokio::test]
    async fn test_mock_actuator_dispatch() {
        let temp_dir = std::env::temp_dir();
        let sock_path = temp_dir.join(format!("test-actuator-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&sock_path);

        let listener = UnixListener::bind(&sock_path).expect("bind mock actuator socket");
        std::env::set_var("SYNTROP_ACTUATOR_SOCKET", &sock_path);

        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 1024];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                if n > 0 {
                    let reply = json!({ "parameters": {} });
                    let mut bytes = serde_json::to_vec(&reply).unwrap_or_default();
                    bytes.push(0x00);
                    let _ = stream.write_all(&bytes).await;
                }
            }
        });

        let move_res = move_mouse_abs(0.5, 0.5).await;
        assert!(move_res.is_ok());

        let click_res = click_mouse(1).await;
        assert!(click_res.is_ok());

        let type_res = type_text("echo test").await;
        assert!(type_res.is_ok());

        let key_res = send_key(28, true).await;
        assert!(key_res.is_ok());

        std::env::remove_var("SYNTROP_ACTUATOR_SOCKET");
        let _ = std::fs::remove_file(&sock_path);
    }

    #[tokio::test]
    async fn test_companion_execute_safety_abort() {
        std::env::set_var("SYNTROP_SIMULATE_PHYSICAL_INPUT", "1");

        let temp_dir = std::env::temp_dir();
        let sock_path = temp_dir.join(format!("test-sensory-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&sock_path);

        let listener = UnixListener::bind(&sock_path).expect("bind mock sensory socket");
        std::env::set_var("SYNTROP_SENSORY_SOCKET", &sock_path);

        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 1024];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                if n > 0 {
                    let reply = json!({
                        "parameters": {
                            "image_base64": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAACklEQVR4nGNiAAAABgADNjd8qAAAAABJRU5ErkJggg==",
                            "format": "png"
                        }
                    });
                    let mut bytes = serde_json::to_vec(&reply).unwrap_or_default();
                    bytes.push(0x00);
                    let _ = stream.write_all(&bytes).await;
                }
            }
        });

        let res = execute_instruction("click 0.5 0.5", None, false)
            .await
            .expect("execute should succeed with abort state");
        assert!(res.aborted);
        assert_eq!(res.executed_count, 0);
        assert!(res.abort_reason.unwrap().contains("Physical user input"));

        std::env::remove_var("SYNTROP_SIMULATE_PHYSICAL_INPUT");
        std::env::remove_var("SYNTROP_SENSORY_SOCKET");
        let _ = std::fs::remove_file(&sock_path);
    }
}
