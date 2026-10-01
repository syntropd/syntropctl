//! Edge tests for sensory CLI arguments, socket failures, and mock RPC.

#[cfg(test)]
mod tests {
    use clap::Parser;
    use std::path::PathBuf;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    use syntropctl_cli::cli::{Cli, Commands, SensoryCommands};
    use syntropctl_core::sensory::*;


    #[test]
    fn test_parse_sensory_audio_args() {
        let args = vec![
            "syntropctl",
            "sensory",
            "audio",
            "--duration-ms",
            "500",
            "--sample-rate",
            "48000",
            "--out",
            "/tmp/audio.pcm",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Sensory {
                command:
                    SensoryCommands::Audio {
                        duration_ms,
                        sample_rate,
                        out,
                    },
            } => {
                assert_eq!(duration_ms, Some(500));
                assert_eq!(sample_rate, Some(48000));
                assert_eq!(out, Some(PathBuf::from("/tmp/audio.pcm")));
            }
            _ => panic!("Expected Sensory Audio"),
        }
    }

    #[test]
    fn test_parse_sensory_frame_and_screen_args() {
        let frame_args = vec![
            "syntropctl",
            "sensory",
            "frame",
            "--device",
            "/dev/video2",
            "--width",
            "1280",
            "--height",
            "720",
        ];
        let cli = Cli::try_parse_from(frame_args).unwrap();
        match cli.command {
            Commands::Sensory {
                command:
                    SensoryCommands::Frame {
                        device,
                        width,
                        height,
                        ..
                    },
            } => {
                assert_eq!(device.as_deref(), Some("/dev/video2"));
                assert_eq!(width, Some(1280));
                assert_eq!(height, Some(720));
            }
            _ => panic!("Expected Sensory Frame"),
        }

        let screen_args = vec!["syntropctl", "sensory", "screen", "--display", ":1"];
        let cli2 = Cli::try_parse_from(screen_args).unwrap();
        match cli2.command {
            Commands::Sensory {
                command: SensoryCommands::Screen { display, .. },
            } => {
                assert_eq!(display.as_deref(), Some(":1"));
            }
            _ => panic!("Expected Sensory Screen"),
        }
    }

    #[tokio::test]
    async fn test_sensory_fails_when_socket_missing() {
        let missing = std::env::temp_dir().join(format!("no-sensory-{}.sock", std::process::id()));
        std::env::set_var("SYNTROP_SENSORY_SOCKET", &missing);

        assert!(capture_audio(None, None).await.is_err());
        assert!(capture_frame(None, None, None).await.is_err());
        assert!(capture_screen(None).await.is_err());
        assert!(get_operator_presence().await.is_err());

        std::env::remove_var("SYNTROP_SENSORY_SOCKET");
    }

    #[tokio::test]
    async fn test_mock_sensory_presence_and_audio() {
        let sock_path = std::env::temp_dir().join(format!("mock-sensory-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&sock_path);
        let listener = UnixListener::bind(&sock_path).unwrap();
        std::env::set_var("SYNTROP_SENSORY_SOCKET", &sock_path);

        let server = tokio::spawn(async move {
            for _ in 0..2 {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut buf = [0u8; 1024];
                    let n = stream.read(&mut buf).await.unwrap();
                    let req_str = String::from_utf8_lossy(&buf[..n]);

                    let reply = if req_str.contains("GetOperatorPresence") {
                        serde_json::json!({
                            "parameters": {
                                "present": true,
                                "confidence": 0.92,
                                "reason": "acoustic energy detected"
                            }
                        })
                    } else {
                        serde_json::json!({
                            "parameters": {
                                "audio_pcm_base64": "AAAA",
                                "sample_rate": 16000,
                                "channels": 1
                            }
                        })
                    };

                    let mut bytes = serde_json::to_vec(&reply).unwrap();
                    bytes.push(0);
                    let _ = stream.write_all(&bytes).await;
                }
            }
        });

        let presence = get_operator_presence().await.unwrap();
        assert!(presence.present);
        assert!((presence.confidence - 0.92).abs() < 1e-4);

        let audio = capture_audio(Some(100), Some(16000)).await.unwrap();
        assert_eq!(audio.sample_rate, 16000);
        assert_eq!(audio.channels, 1);
        assert_eq!(audio.raw_bytes, vec![0, 0, 0]);

        let _ = server.await;
        std::env::remove_var("SYNTROP_SENSORY_SOCKET");
        let _ = std::fs::remove_file(&sock_path);
    }
}
