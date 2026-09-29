//! Unified model catalog query merging modeld CAS inventory and runtimed state.

use crate::daemon::DaemonEndpoint;
use crate::error::SyntropctlError;
use crate::varlink::{VarlinkClient, DEFAULT_RPC_TIMEOUT};
use serde::{Deserialize, Serialize};

/// Unified model metadata combining storage and memory residency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub name: String,
    pub status: String,
    pub size_bytes: u64,
    pub parameter_count: Option<u64>,
    pub backend: Option<String>,
    pub context_window: Option<usize>,
}

/// Content-Addressable Storage quota and inventory statistics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageStats {
    pub total_bytes: u64,
    pub model_count: usize,
    pub pinned_count: usize,
}

/// Query storage quota and CAS statistics from modeld.
pub async fn query_storage_stats() -> Result<StorageStats, SyntropctlError> {
    if let Some(modeld_ep) = DaemonEndpoint::from_name("modeld") {
        let sock = modeld_ep.socket_path();
        if sock.exists() {
            if let Ok(res) = VarlinkClient::call(
                &sock,
                "io.syntrop.Model1.GetStorageStats",
                None,
                DEFAULT_RPC_TIMEOUT,
            )
            .await
            {
                let total_bytes = res.get("total_bytes").and_then(|v| v.as_u64()).unwrap_or(0);
                let model_count = res.get("model_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let pinned_count = res.get("pinned_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                return Ok(StorageStats {
                    total_bytes,
                    model_count,
                    pinned_count,
                });
            }
        }
    }
    Ok(StorageStats::default())
}

/// Query models across modeld (storage) and runtimed (execution).
pub async fn query_models() -> Result<Vec<ModelEntry>, SyntropctlError> {
    let mut entries = Vec::new();

    // 1. Query runtimed for active loaded models
    if let Some(runtimed_ep) = DaemonEndpoint::from_name("runtimed") {
        let sock = runtimed_ep.socket_path();
        if sock.exists() {
            if let Ok(res) = VarlinkClient::call(
                &sock,
                "io.syntrop.Runtime1.ListLoadedModels",
                None,
                DEFAULT_RPC_TIMEOUT,
            )
            .await
            {
                if let Some(models) = res.get("models").and_then(|v| v.as_array()) {
                    for m in models {
                        if let Some(name) = m.get("name").and_then(|v| v.as_str()) {
                            let size = m.get("memory_bytes").and_then(|v| v.as_u64()).unwrap_or(0);
                            let params = m.get("parameter_count").and_then(|v| v.as_u64());
                            let backend = m.get("compute_backend").and_then(|v| v.as_str()).map(|s| s.to_string());
                            let ctx = m.get("context_window").and_then(|v| v.as_u64()).map(|v| v as usize);

                            entries.push(ModelEntry {
                                name: name.to_string(),
                                status: "loaded".to_string(),
                                size_bytes: size,
                                parameter_count: params,
                                backend,
                                context_window: ctx,
                            });
                        }
                    }
                }
            }
        }
    }

    // 2. Query modeld for cached storage models
    if let Some(modeld_ep) = DaemonEndpoint::from_name("modeld") {
        let sock = modeld_ep.socket_path();
        if sock.exists() {
            if let Ok(res) = VarlinkClient::call(
                &sock,
                "io.syntrop.Model1.List",
                None,
                DEFAULT_RPC_TIMEOUT,
            )
            .await
            {
                if let Some(models) = res.get("models").and_then(|v| v.as_array()) {
                    for m in models {
                        let display_name = if let Some(id) = m.get("id").and_then(|v| v.as_str()) {
                            id.to_string()
                        } else if let (Some(n), Some(t)) = (
                            m.get("name").and_then(|v| v.as_str()),
                            m.get("tag").and_then(|v| v.as_str()),
                        ) {
                            format!("{}:{}", n, t)
                        } else if let Some(name) = m.get("name").and_then(|v| v.as_str()) {
                            name.to_string()
                        } else {
                            continue;
                        };

                        if !entries.iter().any(|e| e.name == display_name) {
                            let size = m.get("size_bytes").and_then(|v| v.as_u64()).unwrap_or(0);
                            entries.push(ModelEntry {
                                name: display_name,
                                status: "cached".to_string(),
                                size_bytes: size,
                                parameter_count: None,
                                backend: None,
                                context_window: None,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(entries)
}
