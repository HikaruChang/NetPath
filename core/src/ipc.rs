// Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
// Non-commercial use only. Attribution required.

//! IPC server for runtime control

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, error, info};

use crate::AppState;

/// IPC command
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
enum IpcCommand {
    Status,
    Reload,
    Enable { interface: String },
    Disable { interface: String },
    Interfaces,
    Policies,
    Rules,
    Stats,
}

/// IPC response
#[derive(Debug, Serialize)]
struct IpcResponse {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

/// IPC server
pub struct IpcServer {
    state: Arc<AppState>,
}

impl IpcServer {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn run(&self) -> Result<()> {
        let socket_path = {
            let config = self.state.config.read().await;
            config.global.socket_path.clone()
        };

        // Remove old socket if exists
        let _ = std::fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path)?;
        info!("IPC server listening on {}", socket_path);

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let state = Arc::clone(&self.state);
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(stream, state).await {
                            error!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

async fn handle_client(stream: UnixStream, state: Arc<AppState>) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        let response = match serde_json::from_str::<IpcCommand>(&line) {
            Ok(cmd) => {
                debug!("Received command: {:?}", cmd);
                handle_command(cmd, &state).await
            }
            Err(e) => IpcResponse {
                success: false,
                message: format!("Invalid command: {}", e),
                data: None,
            },
        };

        let response_json = serde_json::to_string(&response)?;
        writer.write_all(response_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;

        line.clear();
    }

    Ok(())
}

async fn handle_command(cmd: IpcCommand, state: &Arc<AppState>) -> IpcResponse {
    match cmd {
        IpcCommand::Status => {
            let interfaces = state.interfaces.read().await;
            let stats: Vec<_> = interfaces
                .interfaces
                .values()
                .map(|i| {
                    serde_json::json!({
                        "name": i.name,
                        "status": i.status.to_string(),
                        "latency": i.latency,
                        "loss": i.loss,
                    })
                })
                .collect();

            IpcResponse {
                success: true,
                message: "OK".to_string(),
                data: Some(serde_json::json!({ "interfaces": stats })),
            }
        }

        IpcCommand::Reload => match state.reload().await {
            Ok(_) => IpcResponse {
                success: true,
                message: "Configuration reloaded".to_string(),
                data: None,
            },
            Err(e) => IpcResponse {
                success: false,
                message: format!("Reload failed: {}", e),
                data: None,
            },
        },

        IpcCommand::Enable { interface } => {
            let mut interfaces = state.interfaces.write().await;
            if let Some(iface) = interfaces.get_mut(&interface) {
                iface.enabled = true;
                IpcResponse {
                    success: true,
                    message: format!("Interface {} enabled", interface),
                    data: None,
                }
            } else {
                IpcResponse {
                    success: false,
                    message: format!("Interface {} not found", interface),
                    data: None,
                }
            }
        }

        IpcCommand::Disable { interface } => {
            let mut interfaces = state.interfaces.write().await;
            if let Some(iface) = interfaces.get_mut(&interface) {
                iface.enabled = false;
                IpcResponse {
                    success: true,
                    message: format!("Interface {} disabled", interface),
                    data: None,
                }
            } else {
                IpcResponse {
                    success: false,
                    message: format!("Interface {} not found", interface),
                    data: None,
                }
            }
        }

        IpcCommand::Interfaces => {
            let interfaces = state.interfaces.read().await;
            let data: Vec<_> = interfaces
                .interfaces
                .values()
                .map(|i| {
                    serde_json::json!({
                        "name": i.name,
                        "ifname": i.ifname,
                        "status": i.status.to_string(),
                        "enabled": i.enabled,
                        "ipv4": i.ipv4_addr.map(|a| a.to_string()),
                        "gateway": i.gateway.map(|g| g.to_string()),
                        "rtable": i.rtable,
                        "fwmark": format!("0x{:x}", i.fwmark),
                        "tx_bytes": i.tx_bytes,
                        "rx_bytes": i.rx_bytes,
                    })
                })
                .collect();

            IpcResponse {
                success: true,
                message: "OK".to_string(),
                data: Some(serde_json::json!({ "interfaces": data })),
            }
        }

        IpcCommand::Policies => {
            let config = state.config.read().await;
            let data: Vec<_> = config
                .policies
                .keys()
                .map(|name| serde_json::json!({ "name": name }))
                .collect();

            IpcResponse {
                success: true,
                message: "OK".to_string(),
                data: Some(serde_json::json!({ "policies": data })),
            }
        }

        IpcCommand::Rules => {
            let config = state.config.read().await;
            let data: Vec<_> = config
                .rules
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "name": r.name,
                        "policy": r.policy,
                        "enabled": r.enabled,
                    })
                })
                .collect();

            IpcResponse {
                success: true,
                message: "OK".to_string(),
                data: Some(serde_json::json!({ "rules": data })),
            }
        }

        IpcCommand::Stats => {
            let nft: tokio::sync::RwLockReadGuard<crate::nft::NftManager> = state.nft.read().await;
            match nft.get_stats() {
                Ok(stats) => IpcResponse {
                    success: true,
                    message: "OK".to_string(),
                    data: Some(serde_json::json!({ "nft_stats": stats })),
                },
                Err(e) => IpcResponse {
                    success: false,
                    message: format!("Failed to get stats: {}", e),
                    data: None,
                },
            }
        }
    }
}
