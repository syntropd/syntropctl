//! Status command handler.

use anyhow::Result;
use syntropctl_core::daemon::{DaemonEndpoint, DAEMONS};
use syntropctl_core::ops::{check_daemon_status, collect_fleet_status};

use crate::format::{print_json, print_status_table};

pub async fn handle_status(target_daemon: Option<String>, json: bool) -> Result<()> {
    let statuses = match target_daemon {
        Some(name) => {
            let ep = DaemonEndpoint::from_name(&name).ok_or_else(|| {
                anyhow::anyhow!("Unknown daemon '{}'. Valid daemons: sentry, inferenced, modeld, contextd, toold, runtimed", name)
            })?;
            vec![check_daemon_status(ep).await]
        }
        None => collect_fleet_status(&DAEMONS).await,
    };

    if json {
        print_json(&statuses);
    } else {
        print_status_table(&statuses);
    }

    Ok(())
}
