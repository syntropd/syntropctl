//! Pure Rust asynchronous Varlink IPC client over Unix domain sockets.

use crate::error::SyntropctlError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::timeout;

/// Timeout for standard Varlink RPC invocations.
pub const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(10);

/// Structure representing a Varlink method call frame.
#[derive(Serialize, Debug)]
pub struct VarlinkCall<'a> {
    pub method: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Structure representing a Varlink method reply frame.
#[derive(Deserialize, Debug)]
pub struct VarlinkReply {
    pub parameters: Option<serde_json::Value>,
    pub error: Option<String>,
    pub continues: Option<bool>,
}

/// Vendor and interface metadata returned by org.varlink.service.GetInfo.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServiceInfo {
    pub vendor: String,
    pub product: String,
    pub version: String,
    pub url: String,
    pub interfaces: Vec<String>,
}

/// Low-level Varlink client for executing calls over Unix domain sockets.
pub struct VarlinkClient;

impl VarlinkClient {
    /// Send a Varlink call and wait for the reply frame with a timeout.
    pub async fn call(
        socket_path: &Path,
        method: &str,
        parameters: Option<serde_json::Value>,
        timeout_dur: Duration,
    ) -> Result<serde_json::Value, SyntropctlError> {
        let stream = UnixStream::connect(socket_path)
            .await
            .map_err(|e| SyntropctlError::DaemonUnavailable {
                daemon: method.split('.').next().unwrap_or("daemon").to_string(),
                socket: socket_path.to_path_buf(),
                source: e,
            })?;

        let (mut reader, mut writer) = stream.into_split();

        let req = VarlinkCall { method, parameters };
        let mut req_bytes = serde_json::to_vec(&req)?;
        req_bytes.push(0);

        timeout(timeout_dur, writer.write_all(&req_bytes))
            .await
            .map_err(|_| SyntropctlError::OperationFailed("Varlink write timed out".into()))??;

        let mut buf = Vec::with_capacity(1024);
        let mut byte = [0u8; 1];

        let read_future = async {
            loop {
                let n = reader.read(&mut byte).await?;
                if n == 0 {
                    break;
                }
                if byte[0] == 0 {
                    break;
                }
                buf.push(byte[0]);
            }
            Ok::<(), std::io::Error>(())
        };

        timeout(timeout_dur, read_future)
            .await
            .map_err(|_| SyntropctlError::OperationFailed("Varlink read timed out".into()))??;

        if buf.is_empty() {
            return Err(SyntropctlError::MalformedReply("Empty reply buffer".into()));
        }

        let reply: VarlinkReply = serde_json::from_slice(&buf)?;
        if let Some(err) = reply.error {
            return Err(SyntropctlError::ProtocolError {
                error: err,
                parameters: reply.parameters,
            });
        }

        Ok(reply.parameters.unwrap_or(serde_json::Value::Null))
    }

    /// Query service introspection metadata via org.varlink.service.GetInfo.
    pub async fn get_info(socket_path: &Path) -> Result<ServiceInfo, SyntropctlError> {
        let res = Self::call(
            socket_path,
            "org.varlink.service.GetInfo",
            None,
            Duration::from_millis(1500),
        )
        .await?;
        let info: ServiceInfo = serde_json::from_value(res)?;
        Ok(info)
    }

    /// Query interface description via org.varlink.service.GetInterfaceDescription.
    pub async fn get_interface_description(
        socket_path: &Path,
        interface: &str,
    ) -> Result<String, SyntropctlError> {
        let params = serde_json::json!({ "interface": interface });
        let res = Self::call(
            socket_path,
            "org.varlink.service.GetInterfaceDescription",
            Some(params),
            Duration::from_millis(1500),
        )
        .await?;
        let desc = res
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        Ok(desc)
    }
}
