// Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
// Non-commercial use only. Attribution required.

//! Route table management using netlink

use anyhow::{Context, Result};
use futures::stream::TryStreamExt;
use rtnetlink::{new_connection, Handle};
use std::net::IpAddr;
use tracing::{debug, info, warn};

use crate::config::NetPathConfig;

/// Route manager
pub struct RouteManager {
    handle: Handle,
}

impl RouteManager {
    pub fn new() -> Result<Self> {
        let (connection, handle, _) =
            new_connection().context("Failed to create netlink connection")?;

        // Spawn the connection in the background
        tokio::spawn(connection);

        Ok(Self { handle })
    }

    pub async fn init(&mut self, _config: &NetPathConfig) -> Result<()> {
        info!("Initializing route manager");
        Ok(())
    }

    pub async fn reload(&mut self, _config: &NetPathConfig) -> Result<()> {
        info!("Reloading route configuration");
        Ok(())
    }

    pub async fn cleanup(&mut self) -> Result<()> {
        info!("Cleaning up routing tables");
        Ok(())
    }

    /// Add a route to a specific table
    pub async fn add_route(
        &self,
        table: u32,
        _dest: Option<IpAddr>,
        gateway: IpAddr,
        dev: &str,
    ) -> Result<()> {
        debug!(
            "Adding route: table={}, gateway={}, dev={}",
            table, gateway, dev
        );

        // Get interface index
        let mut links = self
            .handle
            .link()
            .get()
            .match_name(dev.to_string())
            .execute();

        if let Some(link) = links.try_next().await? {
            let _ifindex = link.header.index;

            // Note: rtnetlink 0.15+ API changed significantly
            // Route adding functionality temporarily disabled
            warn!("Route adding not implemented for rtnetlink 0.15+ API");
            return Ok(());

            /*
            info!("Route added successfully");
            */
        } else {
            warn!("Interface {} not found", dev);
        }

        Ok(())
    }

    /// Delete routes from a table
    pub async fn delete_table_routes(&self, table: u32) -> Result<()> {
        debug!("Deleting routes from table {}", table);

        // Simplified: skip route deletion due to API complexity
        warn!("Route deletion not fully implemented for rtnetlink 0.15+");
        Ok(())
    }

    /// Add policy routing rule
    pub async fn add_rule(&self, priority: u32, fwmark: u32, table: u32) -> Result<()> {
        debug!(
            "Adding rule: priority={}, fwmark=0x{:x}, table={}",
            priority, fwmark, table
        );

        self.handle
            .rule()
            .add()
            .v4()
            .table_id(table)
            .fw_mark(fwmark)
            .priority(priority)
            .execute()
            .await
            .context("Failed to add routing rule")?;

        Ok(())
    }

    /// Delete policy routing rule
    pub async fn delete_rule(&self, priority: u32, fwmark: u32) -> Result<()> {
        debug!(
            "Deleting rule: priority={}, fwmark=0x{:x}",
            priority, fwmark
        );

        // Simplified: skip rule deletion due to API complexity
        warn!("Rule deletion not fully implemented for rtnetlink 0.15+");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_route_manager_creation() {
        let manager = RouteManager::new();
        assert!(manager.is_ok());
    }
}
