//! Info command handler for Varlink introspection.

use anyhow::Result;
use syntropctl_core::daemon::{DaemonEndpoint, DAEMONS};
use syntropctl_core::varlink::VarlinkClient;

use crate::format::print_json;

pub async fn handle_info(daemon: Option<String>, json: bool) -> Result<()> {
    match daemon {
        Some(name) => {
            let ep = DaemonEndpoint::from_name(&name).ok_or_else(|| {
                anyhow::anyhow!("Unknown daemon '{}'. Valid: sentry, inferenced, modeld, contextd, toold, runtimed", name)
            })?;
            let sock = ep.socket_path();
            if !sock.exists() {
                anyhow::bail!("Daemon '{}' socket not found at {}", ep.name, sock.display());
            }

            let info = VarlinkClient::get_info(&sock).await?;
            let mut descriptions = Vec::new();

            for iface in &info.interfaces {
                if iface != "org.varlink.service" {
                    if let Ok(desc) = VarlinkClient::get_interface_description(&sock, iface).await {
                        descriptions.push((iface.clone(), desc));
                    }
                }
            }

            if json {
                print_json(&serde_json::json!({
                    "info": info,
                    "interfaces": descriptions,
                }));
            } else {
                println!("=== Daemon: {} ===", ep.name);
                println!("Product:    {}", info.product);
                println!("Vendor:     {}", info.vendor);
                println!("Version:    {}", info.version);
                println!("URL:        {}", info.url);
                println!("\nInterfaces:");
                for iface in &info.interfaces {
                    println!("  - {}", iface);
                }
                for (iface, desc) in descriptions {
                    println!("\n--- Interface: {} ---\n{}", iface, desc);
                }
            }
        }
        None => {
            for ep in &DAEMONS {
                let sock = ep.socket_path();
                if sock.exists() {
                    if let Ok(info) = VarlinkClient::get_info(&sock).await {
                        println!("Daemon: {:<12} | Product: {:<16} | Version: {}", ep.name, info.product, info.version);
                        for iface in &info.interfaces {
                            println!("    * {}", iface);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
