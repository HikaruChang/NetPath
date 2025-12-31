// Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
// Non-commercial use only. Attribution required.

//! NetPath - High-performance multi-WAN manager using nftables
//!
//! A complete rewrite of mwan3 using modern technologies:
//! - nftables instead of iptables
//! - Rust for memory safety and performance
//! - Async I/O for efficient resource usage

mod config;
mod interface;
mod ipc;
mod metrics;
mod policy;
mod track;

// Conditional compilation for platform-specific modules
#[cfg(target_os = "linux")]
mod nft;
#[cfg(target_os = "linux")]
mod route;

#[cfg(not(target_os = "linux"))]
#[path = "nft_stub.rs"]
mod nft;
#[cfg(not(target_os = "linux"))]
#[path = "route_stub.rs"]
mod route;

use anyhow::Result;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use config::NetPathConfig;
use interface::InterfaceManager;
use ipc::IpcServer;
use metrics::MetricsCollector;
use nft::NftManager;
use policy::PolicyManager;
use route::RouteManager;
use track::TrackManager;

/// Application state shared across components
pub struct AppState {
    pub config: RwLock<NetPathConfig>,
    pub interfaces: RwLock<InterfaceManager>,
    pub nft: RwLock<NftManager>,
    pub policies: RwLock<PolicyManager>,
    pub routes: RwLock<RouteManager>,
    pub tracker: RwLock<TrackManager>,
    pub metrics: RwLock<MetricsCollector>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let config = NetPathConfig::load()?;

        Ok(Self {
            interfaces: RwLock::new(InterfaceManager::new(&config)?),
            nft: RwLock::new(NftManager::new()?),
            policies: RwLock::new(PolicyManager::new(&config)?),
            routes: RwLock::new(RouteManager::new()?),
            tracker: RwLock::new(TrackManager::new(&config)?),
            metrics: RwLock::new(MetricsCollector::new()),
            config: RwLock::new(config),
        })
    }

    /// Initialize all subsystems
    pub async fn init(&self) -> Result<()> {
        info!("Initializing NetPath subsystems...");

        // Initialize nftables rules
        {
            let mut nft: tokio::sync::RwLockWriteGuard<NftManager> = self.nft.write().await;
            let config = self.config.read().await;
            nft.init(&config).await?;
        }

        // Initialize routes
        {
            let mut routes = self.routes.write().await;
            let config = self.config.read().await;
            routes.init(&config).await?;
        }

        // Initialize policies
        {
            let mut policies = self.policies.write().await;
            let config = self.config.read().await;
            policies.init(&config).await?;
        }

        info!("All subsystems initialized successfully");
        Ok(())
    }

    /// Reload configuration
    pub async fn reload(&self) -> Result<()> {
        info!("Reloading configuration...");

        let new_config = NetPathConfig::load()?;

        // Update all managers
        {
            let mut config = self.config.write().await;
            *config = new_config.clone();
        }

        {
            let mut nft = self.nft.write().await;
            nft.reload(&new_config).await?;
        }

        {
            let mut routes = self.routes.write().await;
            routes.reload(&new_config).await?;
        }

        {
            let mut policies = self.policies.write().await;
            policies.reload(&new_config).await?;
        }

        {
            let mut tracker = self.tracker.write().await;
            tracker.reload(&new_config)?;
        }

        info!("Configuration reloaded successfully");
        Ok(())
    }

    /// Cleanup all resources
    pub async fn cleanup(&self) -> Result<()> {
        info!("Cleaning up NetPath resources...");

        {
            let mut nft: tokio::sync::RwLockWriteGuard<NftManager> = self.nft.write().await;
            nft.cleanup().await?;
        }

        {
            let mut routes = self.routes.write().await;
            routes.cleanup().await?;
        }

        info!("Cleanup completed");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    info!(
        "NetPath v{} - Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.",
        env!("CARGO_PKG_VERSION")
    );
    info!("Starting NetPath multi-WAN manager...");

    // Create application state
    let state = Arc::new(AppState::new().await?);

    // Initialize subsystems
    state.init().await?;

    // Start interface tracking
    let tracker_state = Arc::clone(&state);
    let tracker_handle = tokio::spawn(async move {
        info!("Interface tracking loop started");
        loop {
            {
                // Update interface link status from netifd
                let mut interfaces = tracker_state.interfaces.write().await;
                if let Err(e) = interfaces.update_all_link_status() {
                    error!("Link status update error: {}", e);
                }
            }

            {
                let mut tracker = tracker_state.tracker.write().await;
                let interfaces = tracker_state.interfaces.read().await;
                if let Err(e) = tracker.check_all(&interfaces).await {
                    error!("Tracker error: {}", e);
                }
            }

            // Update nftables based on interface status
            {
                let interfaces = tracker_state.interfaces.read().await;
                let mut nft: tokio::sync::RwLockWriteGuard<NftManager> =
                    tracker_state.nft.write().await;
                let config = tracker_state.config.read().await;
                if let Err(e) = nft.update_interface_status(&config, &interfaces).await {
                    error!("NFT update error: {}", e);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    // Start IPC server
    let ipc_state = Arc::clone(&state);
    let ipc_handle = tokio::spawn(async move {
        let server = IpcServer::new(ipc_state);
        if let Err(e) = server.run().await {
            error!("IPC server error: {}", e);
        }
    });

    // Start metrics collector
    let metrics_state = Arc::clone(&state);
    let metrics_handle = tokio::spawn(async move {
        loop {
            {
                let interfaces = metrics_state.interfaces.read().await;
                let mut metrics = metrics_state.metrics.write().await;
                metrics.collect(&interfaces).await;
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });

    info!("NetPath is running");

    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Received shutdown signal");
        }
        Err(e) => {
            error!("Error waiting for shutdown signal: {}", e);
        }
    }

    // Cleanup
    state.cleanup().await?;

    // Cancel background tasks
    tracker_handle.abort();
    ipc_handle.abort();
    metrics_handle.abort();

    info!("NetPath stopped");
    Ok(())
}
