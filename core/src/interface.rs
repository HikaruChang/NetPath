// Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
// Non-commercial use only. Attribution required.

//! Interface management and state tracking

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::process::Command;

use crate::config::{InterfaceConfig, NetPathConfig};

/// Interface status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceStatus {
    Online,
    Offline,
    Disabled,
    Unknown,
}

impl std::fmt::Display for InterfaceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterfaceStatus::Online => write!(f, "online"),
            InterfaceStatus::Offline => write!(f, "offline"),
            InterfaceStatus::Disabled => write!(f, "disabled"),
            InterfaceStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Runtime interface state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceState {
    /// Interface name
    pub name: String,

    /// Network interface name (e.g., eth0)
    pub ifname: String,

    /// Current status
    pub status: InterfaceStatus,

    /// Previous status
    pub prev_status: InterfaceStatus,

    /// IPv4 address
    pub ipv4_addr: Option<IpAddr>,

    /// IPv6 address
    pub ipv6_addr: Option<IpAddr>,

    /// Gateway address
    pub gateway: Option<IpAddr>,

    /// Gateway IPv6
    pub gateway6: Option<IpAddr>,

    /// Route table number
    pub rtable: u32,

    /// Firewall mark
    pub fwmark: u32,

    /// Consecutive up count
    pub up_count: u32,

    /// Consecutive down count
    pub down_count: u32,

    /// Running average latency (ms)
    pub latency: f64,

    /// Packet loss percentage
    pub loss: f64,

    /// Last status change timestamp
    pub status_changed: DateTime<Utc>,

    /// Last check timestamp
    pub last_check: DateTime<Utc>,

    /// Total bytes sent
    pub tx_bytes: u64,

    /// Total bytes received
    pub rx_bytes: u64,

    /// Is interface enabled in config
    pub enabled: bool,
}

impl InterfaceState {
    pub fn new(name: &str, config: &InterfaceConfig, rtable: u32, fwmark: u32) -> Self {
        // Disabled interfaces should be marked as Disabled, not Online/Offline
        let initial_status = if !config.enabled {
            InterfaceStatus::Disabled
        } else {
            match config.initial_state.as_str() {
                "online" => InterfaceStatus::Online,
                "offline" => InterfaceStatus::Offline,
                _ => InterfaceStatus::Unknown,
            }
        };

        Self {
            name: name.to_string(),
            ifname: config.ifname.clone(),
            status: initial_status,
            prev_status: InterfaceStatus::Unknown,
            ipv4_addr: None,
            ipv6_addr: None,
            gateway: None,
            gateway6: None,
            rtable,
            fwmark,
            up_count: 0,
            down_count: 0,
            latency: 0.0,
            loss: 0.0,
            status_changed: Utc::now(),
            last_check: Utc::now(),
            tx_bytes: 0,
            rx_bytes: 0,
            enabled: config.enabled,
        }
    }

    /// Update interface addresses from system
    pub fn update_addresses(&mut self) -> Result<()> {
        // Get addresses using ip command
        let output = Command::new("ip")
            .args(["-j", "addr", "show", &self.ifname])
            .output()
            .context("Failed to get interface addresses")?;

        if output.status.success() {
            let json: serde_json::Value =
                serde_json::from_slice(&output.stdout).unwrap_or(serde_json::Value::Null);

            if let Some(iface) = json.as_array().and_then(|a| a.first()) {
                if let Some(addr_info) = iface.get("addr_info").and_then(|a| a.as_array()) {
                    for addr in addr_info {
                        let family = addr.get("family").and_then(|f| f.as_str());
                        let local = addr.get("local").and_then(|l| l.as_str());

                        if let (Some(family), Some(local)) = (family, local) {
                            if let Ok(ip) = local.parse::<IpAddr>() {
                                match family {
                                    "inet" => self.ipv4_addr = Some(ip),
                                    "inet6" if !local.starts_with("fe80") => {
                                        self.ipv6_addr = Some(ip)
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        // Update gateway
        self.update_gateway()?;

        Ok(())
    }

    /// Update gateway from routing table
    fn update_gateway(&mut self) -> Result<()> {
        let output = Command::new("ip")
            .args(["-j", "route", "show", "dev", &self.ifname])
            .output()
            .context("Failed to get routes")?;

        if output.status.success() {
            let json: serde_json::Value =
                serde_json::from_slice(&output.stdout).unwrap_or(serde_json::Value::Null);

            if let Some(routes) = json.as_array() {
                for route in routes {
                    let dst = route.get("dst").and_then(|d| d.as_str());
                    let gateway = route.get("gateway").and_then(|g| g.as_str());

                    if dst == Some("default") {
                        if let Some(gw) = gateway {
                            if let Ok(ip) = gw.parse::<IpAddr>() {
                                match ip {
                                    IpAddr::V4(_) => self.gateway = Some(ip),
                                    IpAddr::V6(_) => self.gateway6 = Some(ip),
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check interface link status from netifd
    pub fn check_link_status(&mut self) -> Result<bool> {
        // Query netifd for interface status
        // Use self.name (UCI section name) instead of self.ifname for ubus query
        let output = Command::new("ubus")
            .args([
                "call",
                &format!("network.interface.{}", self.name),
                "status",
            ])
            .output()
            .context("Failed to query interface status from netifd")?;

        if output.status.success() {
            let json: serde_json::Value =
                serde_json::from_slice(&output.stdout).unwrap_or(serde_json::Value::Null);

            // Check if interface is up
            let is_up = json.get("up").and_then(|v| v.as_bool()).unwrap_or(false);

            tracing::info!("Link check for {}: up={}", self.name, is_up);

            Ok(is_up)
        } else {
            tracing::warn!(
                "Failed to query ubus for {}: {}",
                self.name,
                String::from_utf8_lossy(&output.stderr)
            );
            // If ubus call fails, assume interface is down
            Ok(false)
        }
    }

    /// Update traffic statistics
    pub fn update_stats(&mut self) -> Result<()> {
        let output = Command::new("ip")
            .args(["-j", "-s", "link", "show", &self.ifname])
            .output()
            .context("Failed to get interface stats")?;

        if output.status.success() {
            let json: serde_json::Value =
                serde_json::from_slice(&output.stdout).unwrap_or(serde_json::Value::Null);

            if let Some(iface) = json.as_array().and_then(|a| a.first()) {
                if let Some(stats) = iface.get("stats64") {
                    if let Some(rx) = stats
                        .get("rx")
                        .and_then(|r| r.get("bytes"))
                        .and_then(|b| b.as_u64())
                    {
                        self.rx_bytes = rx;
                    }
                    if let Some(tx) = stats
                        .get("tx")
                        .and_then(|t| t.get("bytes"))
                        .and_then(|b| b.as_u64())
                    {
                        self.tx_bytes = tx;
                    }
                }
            }
        }

        Ok(())
    }

    /// Set interface status with transition tracking
    pub fn set_status(&mut self, status: InterfaceStatus) {
        if self.status != status {
            self.prev_status = self.status;
            self.status = status;
            self.status_changed = Utc::now();
        }
    }

    /// Check if interface is usable for routing
    pub fn is_usable(&self) -> bool {
        self.enabled && self.status == InterfaceStatus::Online
    }

    /// Update status based on enabled flag and link status
    pub fn update_status_from_link(&mut self) -> Result<()> {
        if !self.enabled {
            // Disabled interfaces should always be marked as Disabled
            self.set_status(InterfaceStatus::Disabled);
            return Ok(());
        }

        // For enabled interfaces, check actual link status
        match self.check_link_status() {
            Ok(is_up) => {
                if is_up {
                    // Only mark as Online if it was previously Offline or Unknown
                    if self.status != InterfaceStatus::Online {
                        tracing::info!(
                            "Interface {} link is up, changing status from {} to Online",
                            self.name,
                            self.status
                        );
                        self.set_status(InterfaceStatus::Online);
                    }
                } else {
                    // Link is down, mark as Offline
                    if self.status != InterfaceStatus::Offline {
                        tracing::info!(
                            "Interface {} link is down, changing status from {} to Offline",
                            self.name,
                            self.status
                        );
                        self.set_status(InterfaceStatus::Offline);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to check link status for {}: {}", self.name, e);
                // On error, mark as Unknown if not already
                if self.status != InterfaceStatus::Unknown {
                    self.set_status(InterfaceStatus::Unknown);
                }
            }
        }
        Ok(())
    }
}

/// Manager for all interfaces
pub struct InterfaceManager {
    /// Interface states indexed by name
    pub interfaces: HashMap<String, InterfaceState>,

    /// Next available route table
    next_rtable: u32,

    /// Next available fwmark
    next_fwmark: u32,
}

impl InterfaceManager {
    /// Create new interface manager
    pub fn new(config: &NetPathConfig) -> Result<Self> {
        let mut manager = Self {
            interfaces: HashMap::new(),
            next_rtable: config.global.rtable_offset,
            next_fwmark: config.global.fwmark_offset,
        };

        // Sort interface names for deterministic fwmark assignment
        let mut sorted_names: Vec<_> = config.interfaces.keys().collect();
        sorted_names.sort();

        // Initialize interfaces from config in sorted order
        // Note: We initialize ALL interfaces (including disabled ones) so they can be monitored
        for name in sorted_names {
            let iface_config = &config.interfaces[name];

            let rtable = manager.next_rtable;
            let fwmark = manager.next_fwmark;

            let mut state = InterfaceState::new(name, iface_config, rtable, fwmark);

            tracing::info!(
                "Interface {} (enabled={}): fwmark 0x{:x}, rtable {}",
                name,
                iface_config.enabled,
                fwmark,
                rtable
            );

            // Only setup routing for enabled interfaces
            if iface_config.enabled {
                // Setup ip rule for this interface
                if let Err(e) = manager.setup_ip_rule(fwmark, rtable) {
                    tracing::warn!("Failed to setup ip rule for {}: {}", name, e);
                }

                // Setup route table for this interface
                if let Err(e) = manager.setup_route_table(&iface_config.ifname, rtable) {
                    tracing::warn!("Failed to setup route table for {}: {}", name, e);
                }

                // Check initial link status for enabled interfaces
                let _ = state.update_status_from_link();
            }

            manager.interfaces.insert(name.clone(), state);

            manager.next_rtable += 1;
            manager.next_fwmark += 1;
        }

        Ok(manager)
    }

    /// Setup ip rule for fwmark -> table mapping
    fn setup_ip_rule(&self, fwmark: u32, table: u32) -> Result<()> {
        use std::process::Command;

        // Delete existing rule first (ignore errors)
        let _ = Command::new("ip")
            .args(["rule", "del", "fwmark", &format!("0x{:x}", fwmark)])
            .output();

        // Add new rule
        let priority = 500 + (fwmark - 0x100); // Base priority 500
        let output = Command::new("ip")
            .args([
                "rule",
                "add",
                "priority",
                &priority.to_string(),
                "fwmark",
                &format!("0x{:x}", fwmark),
                "lookup",
                &table.to_string(),
            ])
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "ip rule add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        tracing::info!("Added ip rule: fwmark 0x{:x} -> table {}", fwmark, table);
        Ok(())
    }

    /// Setup route table with default route via interface
    fn setup_route_table(&self, ifname: &str, table: u32) -> Result<()> {
        use std::process::Command;

        // Use jsonfilter to extract l3_device from ubus
        let dev_output = Command::new("sh")
            .args([
                "-c",
                &format!(
                    "ubus call network.interface.{} status 2>/dev/null | jsonfilter -e '@.l3_device'",
                    ifname
                ),
            ])
            .output()?;

        let l3_device = String::from_utf8_lossy(&dev_output.stdout)
            .trim()
            .to_string();

        // Use jsonfilter to extract gateway (nexthop from first route)
        let gw_output = Command::new("sh")
            .args([
                "-c",
                &format!(
                    "ubus call network.interface.{} status 2>/dev/null | jsonfilter -e '@.route[0].nexthop'",
                    ifname
                ),
            ])
            .output()?;

        let gateway = String::from_utf8_lossy(&gw_output.stdout)
            .trim()
            .to_string();

        if !l3_device.is_empty() && !gateway.is_empty() && gateway != "0.0.0.0" {
            // Flush existing routes
            let _ = Command::new("ip")
                .args(["route", "flush", "table", &table.to_string()])
                .output();

            // Add default route
            let output = Command::new("ip")
                .args([
                    "route",
                    "add",
                    "default",
                    "via",
                    &gateway,
                    "dev",
                    &l3_device,
                    "table",
                    &table.to_string(),
                ])
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.contains("File exists") {
                    tracing::warn!("ip route add failed for {}: {}", ifname, stderr);
                }
            } else {
                tracing::info!(
                    "Added route: table {} default via {} dev {}",
                    table,
                    gateway,
                    l3_device
                );
            }
        } else if !l3_device.is_empty() {
            // PPPoE may not have explicit gateway, use device only
            let _ = Command::new("ip")
                .args(["route", "flush", "table", &table.to_string()])
                .output();

            let output = Command::new("ip")
                .args([
                    "route",
                    "add",
                    "default",
                    "dev",
                    &l3_device,
                    "table",
                    &table.to_string(),
                ])
                .output()?;

            if output.status.success() {
                tracing::info!(
                    "Added route: table {} default dev {} (no gateway)",
                    table,
                    l3_device
                );
            }
        } else {
            tracing::warn!("No l3_device found for interface {}", ifname);
        }

        Ok(())
    }

    /// Get interface state by name
    pub fn get(&self, name: &str) -> Option<&InterfaceState> {
        self.interfaces.get(name)
    }

    /// Get mutable interface state
    pub fn get_mut(&mut self, name: &str) -> Option<&mut InterfaceState> {
        self.interfaces.get_mut(name)
    }

    /// Get all online interfaces
    pub fn online(&self) -> impl Iterator<Item = &InterfaceState> {
        self.interfaces
            .values()
            .filter(|i| i.status == InterfaceStatus::Online)
    }

    /// Get all usable interfaces
    pub fn usable(&self) -> impl Iterator<Item = &InterfaceState> {
        self.interfaces.values().filter(|i| i.is_usable())
    }

    /// Update all interface addresses
    pub fn update_all_addresses(&mut self) -> Result<()> {
        for state in self.interfaces.values_mut() {
            if state.enabled {
                let _ = state.update_addresses();
            }
        }
        Ok(())
    }

    /// Update all interface statistics
    pub fn update_all_stats(&mut self) -> Result<()> {
        for state in self.interfaces.values_mut() {
            if state.enabled {
                let _ = state.update_stats();
            }
        }
        Ok(())
    }

    /// Update all interface link status from netifd
    pub fn update_all_link_status(&mut self) -> Result<()> {
        for state in self.interfaces.values_mut() {
            // Monitor all interfaces to detect when they come back online
            let _ = state.update_status_from_link();
        }
        Ok(())
    }

    /// Reload interfaces from config
    pub fn reload(&mut self, config: &NetPathConfig) -> Result<()> {
        // Add new interfaces, update existing
        for (name, iface_config) in &config.interfaces {
            if let Some(state) = self.interfaces.get_mut(name) {
                // Update existing
                let was_enabled = state.enabled;
                state.enabled = iface_config.enabled;
                state.ifname = iface_config.ifname.clone();

                // If enabled status changed, update status accordingly
                if was_enabled != iface_config.enabled {
                    if !iface_config.enabled {
                        tracing::info!("Interface {} disabled, setting status to Disabled", name);
                        state.set_status(InterfaceStatus::Disabled);
                    } else {
                        tracing::info!("Interface {} enabled, will check link status", name);
                        let _ = state.update_status_from_link();
                    }
                }
            } else {
                // Add new
                let rtable = self.next_rtable;
                let fwmark = self.next_fwmark;

                let state = InterfaceState::new(name, iface_config, rtable, fwmark);
                self.interfaces.insert(name.clone(), state);

                self.next_rtable += 1;
                self.next_fwmark += 1;
            }
        }

        // Remove deleted interfaces
        self.interfaces
            .retain(|name, _| config.interfaces.contains_key(name));

        Ok(())
    }

    /// Get interface by fwmark
    pub fn get_by_fwmark(&self, fwmark: u32) -> Option<&InterfaceState> {
        self.interfaces.values().find(|i| i.fwmark == fwmark)
    }

    /// Get interface by route table
    pub fn get_by_rtable(&self, rtable: u32) -> Option<&InterfaceState> {
        self.interfaces.values().find(|i| i.rtable == rtable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TrackConfig;

    #[test]
    fn test_interface_state_new() {
        let config = InterfaceConfig {
            ifname: "eth0".to_string(),
            enabled: true,
            initial_state: "online".to_string(),
            family: "ipv4".to_string(),
            track: TrackConfig::default(),
            flush_conntrack: true,
        };

        let state = InterfaceState::new("wan", &config, 1000, 0x100);

        assert_eq!(state.name, "wan");
        assert_eq!(state.ifname, "eth0");
        assert_eq!(state.status, InterfaceStatus::Online);
        assert_eq!(state.rtable, 1000);
        assert_eq!(state.fwmark, 0x100);
    }
}
