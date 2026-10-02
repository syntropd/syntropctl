//! Zero-disk multimodal HTTP streaming over routerd Unix domain socket.

use crate::error::SyntropctlError;
use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// Timeout for multimodal inference requests to routerd.
pub const MULTIMODAL_ROUTER_TIMEOUT: Duration = Duration::from_secs(60);

/// Resolve the routerd HTTP Unix domain socket path.
pub fn router_socket_path() -> PathBuf {
    if let Ok(val) = std::env::var("SYNTROP_ROUTER_SOCKET") {
        if !val.trim().is_empty() {
            return PathBuf::from(val);
        }
    }
    PathBuf::from("/run/syntrop/router.sock")
}

/// Transmit prompt (and optional base64 image) directly to routerd without disk I/O.
pub async fn query_router(
    prompt: &str,
    image_base64: Option<&str>,
    system_prompt: Option<&str>,
) -> Result<String, SyntropctlError> {
    let sock = router_socket_path();
    if !sock.exists() {
        return Err(SyntropctlError::DaemonUnavailable {
            daemon: "routerd".to_string(),
            socket: sock,
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "router.sock does not exist"),
        });
    }

    let mut messages = Vec::new();
    if let Some(sys) = system_prompt {
        messages.push(json!({
            "role": "system",
            "content": sys,
        }));
    }

    if let Some(img) = image_base64 {
        let image_data_url = format!("data:image/png;base64,{}", img);
        messages.push(json!({
            "role": "user",
            "content": [
                { "type": "text", "text": prompt },
                {
                    "type": "image_url",
                    "image_url": { "url": image_data_url }
                }
            ]
        }));
    } else {
        messages.push(json!({
            "role": "user",
            "content": prompt,
        }));
    }

    let request_body = json!({
        "model": "router:auto",
        "messages": messages,
        "max_tokens": 1024,
    });

    let body_bytes = serde_json::to_vec(&request_body)?;
    let http_req = format!(
        "POST /v1/chat/completions HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body_bytes.len()
    );

    let mut stream = UnixStream::connect(&sock).await.map_err(|e| {
        SyntropctlError::DaemonUnavailable {
            daemon: "routerd".to_string(),
            socket: sock.clone(),
            source: e,
        }
    })?;

    stream.write_all(http_req.as_bytes()).await?;
    stream.write_all(&body_bytes).await?;
    stream.flush().await?;

    let mut response_bytes = Vec::new();
    let read_fut = stream.read_to_end(&mut response_bytes);
    tokio::time::timeout(MULTIMODAL_ROUTER_TIMEOUT, read_fut)
        .await
        .map_err(|_| SyntropctlError::OperationFailed("Multimodal router request timed out".into()))??;

    parse_http_completion_response(&response_bytes)
}

/// Transmit text prompt directly to routerd without disk I/O.
pub async fn query_router_text(
    prompt: &str,
    system_prompt: Option<&str>,
) -> Result<String, SyntropctlError> {
    query_router(prompt, None, system_prompt).await
}

/// Transmit multimodal prompt and base64 PNG image directly to routerd without disk I/O.
pub async fn query_router_multimodal(
    prompt: &str,
    image_base64: &str,
    system_prompt: Option<&str>,
) -> Result<String, SyntropctlError> {
    query_router(prompt, Some(image_base64), system_prompt).await
}

/// Decode HTTP chunked transfer-encoding body into plain payload.
pub fn decode_chunked_body(mut body: &str) -> String {
    let mut out = String::new();
    while !body.is_empty() {
        let trimmed = body.trim_start();
        if trimmed.is_empty() || trimmed == "0" {
            break;
        }
        if let Some(pos) = trimmed.find("\r\n") {
            let line = &trimmed[..pos];
            let hex_len = line.split(';').next().unwrap_or(line).trim();
            if let Ok(len) = usize::from_str_radix(hex_len, 16) {
                if len == 0 {
                    break;
                }
                let data_start = pos + 2;
                if trimmed.len() >= data_start + len {
                    out.push_str(&trimmed[data_start..data_start + len]);
                    let after = &trimmed[data_start + len..];
                    body = after.strip_prefix("\r\n").unwrap_or(after);
                } else {
                    out.push_str(&trimmed[data_start..]);
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
    out
}

/// Parse HTTP status and choices[0].message.content from raw response bytes.
pub fn parse_http_completion_response(bytes: &[u8]) -> Result<String, SyntropctlError> {
    let text = String::from_utf8_lossy(bytes);
    let mut parts = text.splitn(2, "\r\n\r\n");
    let header_part = parts.next().unwrap_or("");
    let body_part = parts.next().unwrap_or("");

    let first_line = header_part.lines().next().unwrap_or("");
    if !first_line.contains("200") {
        return Err(SyntropctlError::OperationFailed(format!(
            "Router returned HTTP error: {} - body: {}",
            first_line,
            body_part.trim()
        )));
    }

    let trimmed_body = body_part.trim();
    let val: serde_json::Value = match serde_json::from_str(trimmed_body) {
        Ok(v) => v,
        Err(_) => {
            let decoded = decode_chunked_body(trimmed_body);
            serde_json::from_str(decoded.trim()).map_err(|err| {
                SyntropctlError::MalformedReply(format!("Failed to parse router completion JSON: {err}"))
            })?
        }
    };

    if let Some(content) = val
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c0| c0.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|txt| txt.as_str())
    {
        Ok(content.to_string())
    } else {
        Err(SyntropctlError::MalformedReply(
            "Missing choices[0].message.content in completion reply".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_completion_response_success() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"choices\":[{\"message\":{\"content\":\"Screen contains a terminal.\"}}]}";
        let res = parse_http_completion_response(raw).unwrap();
        assert_eq!(res, "Screen contains a terminal.");
    }

    #[test]
    fn test_parse_http_completion_response_chunked() {
        let raw = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n43\r\n{\"choices\":[{\"message\":{\"content\":\"Screen contains a terminal.\"}}]}\r\n0\r\n\r\n";
        let res = parse_http_completion_response(raw).unwrap();
        assert_eq!(res, "Screen contains a terminal.");
    }

    #[test]
    fn test_parse_http_completion_response_error() {
        let raw = b"HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"no provider\"}";
        let res = parse_http_completion_response(raw);
        assert!(res.is_err());
    }
}
