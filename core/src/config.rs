// Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
// Non-commercial use only. Attribution required.

//! Configuration management for NetPath
//!
//! Supports UCI-style configuration for OpenWrt integration

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Default configuration path
const DEFAULT_CONFIG_PATH: &str = "/etc/netpath/config.toml";
const UCI_CONFIG_PATH: &str = "/etc/config/netpath";

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetPathConfig {
    /// Global settings
    #[serde(default)]
    pub global: GlobalConfig,

    /// Interface definitions
    #[serde(default)]
    pub interfaces: HashMap<String, InterfaceConfig>,

    /// Member definitions (interface groups)
    #[serde(default)]
    pub members: HashMap<String, MemberConfig>,

    /// Policy definitions
    #[serde(default)]
    pub policies: HashMap<String, PolicyConfig>,

    /// Rules for traffic classification
    #[serde(default)]
    pub rules: Vec<RuleConfig>,
}

/// Global configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Enable NetPath
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// nftables table name
    #[serde(default = "default_nft_table")]
    pub nft_table: String,

    /// Route table offset (starting number for custom tables)
    #[serde(default = "default_rtable_offset")]
    pub rtable_offset: u32,

    /// fwmark offset
    #[serde(default = "default_fwmark_offset")]
    pub fwmark_offset: u32,

    /// fwmark mask
    #[serde(default = "default_fwmark_mask")]
    pub fwmark_mask: u32,

    /// Local source routing
    #[serde(default)]
    pub local_source: bool,

    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// IPC socket path
    #[serde(default = "default_socket_path")]
    pub socket_path: String,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            nft_table: "netpath".to_string(),
            rtable_offset: 1000,
            fwmark_offset: 0x100,
            fwmark_mask: 0xff00,
            local_source: true,
            log_level: "info".to_string(),
            socket_path: "/var/run/netpath.sock".to_string(),
        }
    }
}

fn default_enabled() -> bool {
    true
}
fn default_nft_table() -> String {
    "netpath".to_string()
}
fn default_rtable_offset() -> u32 {
    1000
}
fn default_fwmark_offset() -> u32 {
    0x100
}
fn default_fwmark_mask() -> u32 {
    0xff00
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_socket_path() -> String {
    "/var/run/netpath.sock".to_string()
}

/// Interface configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceConfig {
    /// Interface name (e.g., eth0, wan)
    pub ifname: String,

    /// Enable this interface
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Initial state (online/offline)
    #[serde(default = "default_initial_state")]
    pub initial_state: String,

    /// Family (ipv4, ipv6, both)
    #[serde(default = "default_family")]
    pub family: String,

    /// Tracking configuration
    #[serde(default)]
    pub track: TrackConfig,

    /// Flush connections on failover
    #[serde(default = "default_flush")]
    pub flush_conntrack: bool,
}

fn default_initial_state() -> String {
    "online".to_string()
}
fn default_family() -> String {
    "ipv4".to_string()
}
fn default_flush() -> bool {
    true
}

/// Tracking configuration for interface health check
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrackConfig {
    /// Tracking method (ping, arping, httping, dns, none)
    #[serde(default = "default_track_method")]
    pub method: String,

    /// Tracking targets (IP addresses or domains)
    #[serde(default)]
    pub targets: Vec<String>,

    /// Reliability threshold (how many targets must respond)
    #[serde(default = "default_reliability")]
    pub reliability: u32,

    /// Number of pings per target
    #[serde(default = "default_count")]
    pub count: u32,

    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u32,

    /// Interval between checks in seconds
    #[serde(default = "default_interval")]
    pub interval: u32,

    /// Failures before marking down
    #[serde(default = "default_down")]
    pub down: u32,

    /// Successes before marking up
    #[serde(default = "default_up")]
    pub up: u32,
}

fn default_track_method() -> String {
    "ping".to_string()
}
fn default_reliability() -> u32 {
    1
}
fn default_count() -> u32 {
    1
}
fn default_timeout() -> u32 {
    4
}
fn default_interval() -> u32 {
    10
}
fn default_down() -> u32 {
    5
}
fn default_up() -> u32 {
    5
}

/// Member configuration (interface in a policy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberConfig {
    /// Interface name
    pub interface: String,

    /// Metric (lower is preferred)
    #[serde(default = "default_metric")]
    pub metric: u32,

    /// Weight for load balancing
    #[serde(default = "default_weight")]
    pub weight: u32,
}

fn default_metric() -> u32 {
    1
}
fn default_weight() -> u32 {
    1
}

/// Policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Policy members
    pub members: Vec<String>,

    /// Load balancing mode
    #[serde(default = "default_lb_mode")]
    pub mode: LoadBalanceMode,

    /// Sticky connections
    #[serde(default)]
    pub sticky: bool,

    /// Sticky timeout in seconds
    #[serde(default = "default_sticky_timeout")]
    pub sticky_timeout: u32,

    /// Last resort action
    #[serde(default)]
    pub last_resort: LastResort,
}

fn default_lb_mode() -> LoadBalanceMode {
    LoadBalanceMode::Balanced
}
fn default_sticky_timeout() -> u32 {
    600
}

/// Load balancing modes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LoadBalanceMode {
    /// Balance across all interfaces
    Balanced,
    /// Use in order of metric (failover mode)
    Failover,
    /// Round-robin
    RoundRobin,
    /// Random selection
    Random,
    /// Weighted random
    Weighted,
    /// Hash-based (source IP)
    Hash,
}

impl Default for LoadBalanceMode {
    fn default() -> Self {
        Self::Balanced
    }
}

/// Last resort action when all interfaces are down
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LastResort {
    /// Use unreachable
    Unreachable,
    /// Use blackhole
    Blackhole,
    /// Use default route
    Default,
}

impl Default for LastResort {
    fn default() -> Self {
        Self::Unreachable
    }
}

/// Traffic classification rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    /// Rule name
    pub name: String,

    /// Source IP/CIDR
    #[serde(default)]
    pub src_ip: Option<String>,

    /// Destination IP/CIDR
    #[serde(default)]
    pub dest_ip: Option<String>,

    /// Source port(s)
    #[serde(default)]
    pub src_port: Option<String>,

    /// Destination port(s)
    #[serde(default)]
    pub dest_port: Option<String>,

    /// Protocol (tcp, udp, icmp, all)
    #[serde(default)]
    pub proto: Option<String>,

    /// Policy to use
    pub policy: String,

    /// Enable this rule
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Sticky connections for this rule
    #[serde(default)]
    pub sticky: bool,

    /// IPSET name (for matching)
    #[serde(default)]
    pub ipset: Option<String>,

    /// Timeout for dynamic rules
    #[serde(default)]
    pub timeout: Option<u32>,

    /// Rule priority (lower is higher priority)
    #[serde(default = "default_priority")]
    pub priority: u32,
}

fn default_priority() -> u32 {
    100
}

impl NetPathConfig {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        // Try TOML config first
        if Path::new(DEFAULT_CONFIG_PATH).exists() {
            let content =
                fs::read_to_string(DEFAULT_CONFIG_PATH).context("Failed to read config file")?;
            let config: NetPathConfig =
                toml::from_str(&content).context("Failed to parse config file")?;
            return Ok(config);
        }

        // Try UCI config
        if Path::new(UCI_CONFIG_PATH).exists() {
            return Self::load_uci();
        }

        // Return default config
        Ok(Self::default())
    }

    /// Load from UCI configuration
    fn load_uci() -> Result<Self> {
        use std::process::Command;

        let mut config = NetPathConfig::default();

        // Parse UCI config using uci command
        // Global settings
        if let Ok(output) = Command::new("uci").args(["show", "netpath"]).output() {
            let content = String::from_utf8_lossy(&output.stdout);
            config.parse_uci_content(&content)?;
        }

        Ok(config)
    }

    /// Parse UCI format content
    fn parse_uci_content(&mut self, content: &str) -> Result<()> {
        use std::collections::HashMap;

        // First pass: identify sections by type
        // Format: netpath.section_name=section_type
        let mut section_types: HashMap<String, String> = HashMap::new();
        // Store properties by section: section_name -> (field, value)
        let mut section_props: HashMap<String, HashMap<String, String>> = HashMap::new();

        for line in content.lines() {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                continue;
            }

            let key = parts[0].trim();
            let value = parts[1].trim().trim_matches('\'');

            // Skip if not netpath config
            if !key.starts_with("netpath.") {
                continue;
            }

            let rest = key.trim_start_matches("netpath.");

            // Check if this is a section type declaration (netpath.name=type)
            if !rest.contains('.') {
                section_types.insert(rest.to_string(), value.to_string());
                continue;
            }

            // This is a property: section_name.field=value
            let prop_parts: Vec<&str> = rest.splitn(2, '.').collect();
            if prop_parts.len() == 2 {
                let section_name = prop_parts[0];
                let field = prop_parts[1];
                section_props
                    .entry(section_name.to_string())
                    .or_default()
                    .insert(field.to_string(), value.to_string());
            }
        }

        // Process sections by type
        for (section_name, section_type) in &section_types {
            let props = section_props.get(section_name).cloned().unwrap_or_default();

            match section_type.as_str() {
                "globals" => self.parse_globals_props(&props),
                "interface" => self.parse_interface_props(section_name, &props),
                "member" => self.parse_member_props(section_name, &props),
                "policy" => self.parse_policy_props(section_name, &props),
                "rule" => self.parse_rule_props(section_name, &props),
                _ => {}
            }
        }

        // Sort rules by priority (ascending)
        self.rules.sort_by_key(|r| r.priority);

        Ok(())
    }

    fn parse_globals_props(&mut self, props: &HashMap<String, String>) {
        if let Some(v) = props.get("enabled") {
            self.global.enabled = v == "1";
        }
        if let Some(v) = props.get("nft_table") {
            self.global.nft_table = v.clone();
        }
        if let Some(v) = props.get("rtable_offset") {
            self.global.rtable_offset = v.parse().unwrap_or(1000);
        }
        if let Some(v) = props.get("fwmark_offset") {
            self.global.fwmark_offset = if let Some(hex) = v.strip_prefix("0x") {
                u32::from_str_radix(hex, 16).unwrap_or(0x100)
            } else {
                v.parse().unwrap_or(0x100)
            };
        }
        if let Some(v) = props.get("local_source") {
            self.global.local_source = v == "1";
        }
    }

    fn parse_interface_props(&mut self, name: &str, props: &HashMap<String, String>) {
        let enabled = props.get("enabled").map(|v| v == "1").unwrap_or(true);
        let ifname = props
            .get("ifname")
            .cloned()
            .unwrap_or_else(|| name.to_string());

        let track = TrackConfig {
            method: props
                .get("track_method")
                .cloned()
                .unwrap_or_else(|| "ping".to_string()),
            targets: Self::parse_list(props.get("track_targets")),
            reliability: props
                .get("track_reliability")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            count: props
                .get("track_count")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            timeout: props
                .get("track_timeout")
                .and_then(|v| v.parse().ok())
                .unwrap_or(4),
            interval: props
                .get("track_interval")
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            down: props
                .get("track_down")
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            up: props
                .get("track_up")
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
        };

        let iface = InterfaceConfig {
            enabled,
            ifname,
            family: props
                .get("family")
                .cloned()
                .unwrap_or_else(|| "ipv4".to_string()),
            initial_state: props
                .get("initial_state")
                .cloned()
                .unwrap_or_else(|| "online".to_string()),
            track,
            flush_conntrack: props
                .get("flush_conntrack")
                .map(|v| v == "1")
                .unwrap_or(true),
        };

        self.interfaces.insert(name.to_string(), iface);
    }

    fn parse_member_props(&mut self, name: &str, props: &HashMap<String, String>) {
        let interface = props.get("interface").cloned().unwrap_or_default();
        let weight = props
            .get("weight")
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);

        let member = MemberConfig {
            interface,
            weight,
            metric: props
                .get("metric")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
        };

        self.members.insert(name.to_string(), member);
    }

    fn parse_policy_props(&mut self, name: &str, props: &HashMap<String, String>) {
        let mode_str = props
            .get("mode")
            .cloned()
            .unwrap_or_else(|| "balanced".to_string());
        let mode = match mode_str.as_str() {
            "balanced" => LoadBalanceMode::Balanced,
            "failover" | "fo" => LoadBalanceMode::Failover,
            "random" => LoadBalanceMode::Random,
            "roundrobin" | "rr" => LoadBalanceMode::RoundRobin,
            "weighted" => LoadBalanceMode::Weighted,
            "hash" => LoadBalanceMode::Hash,
            _ => LoadBalanceMode::Balanced,
        };

        let last_resort_str = props
            .get("last_resort")
            .cloned()
            .unwrap_or_else(|| "unreachable".to_string());
        let last_resort = match last_resort_str.as_str() {
            "unreachable" => LastResort::Unreachable,
            "blackhole" => LastResort::Blackhole,
            "default" => LastResort::Default,
            _ => LastResort::Unreachable,
        };

        let policy = PolicyConfig {
            mode,
            members: Self::parse_list(props.get("members")),
            last_resort,
            sticky: props.get("sticky").map(|v| v == "1").unwrap_or(false),
            sticky_timeout: props
                .get("sticky_timeout")
                .and_then(|v| v.parse().ok())
                .unwrap_or(600),
        };

        self.policies.insert(name.to_string(), policy);
    }

    fn parse_rule_props(&mut self, name: &str, props: &HashMap<String, String>) {
        let enabled = props.get("enabled").map(|v| v == "1").unwrap_or(true);
        let policy = props.get("policy").cloned().unwrap_or_default();

        if policy.is_empty() {
            return; // Skip rules without a policy
        }

        let rule = RuleConfig {
            name: name.to_string(),
            src_ip: props.get("src_ip").filter(|s| !s.is_empty()).cloned(),
            dest_ip: props
                .get("dest_ip")
                .filter(|s| !s.is_empty() && *s != "0.0.0.0/0" && *s != "::/0")
                .cloned(),
            src_port: props.get("src_port").filter(|s| !s.is_empty()).cloned(),
            dest_port: props.get("dest_port").filter(|s| !s.is_empty()).cloned(),
            proto: props
                .get("proto")
                .filter(|s| !s.is_empty() && *s != "all")
                .cloned(),
            policy,
            enabled,
            sticky: props.get("sticky").map(|v| v == "1").unwrap_or(false),
            ipset: props.get("ipset").filter(|s| !s.is_empty()).cloned(),
            timeout: props.get("timeout").and_then(|v| v.parse().ok()),
            priority: props
                .get("priority")
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
        };

        self.rules.push(rule);
    }

    /// Parse UCI list format: 'val1' 'val2' 'val3'
    fn parse_list(value: Option<&String>) -> Vec<String> {
        value
            .map(|v| {
                v.split('\'')
                    .filter(|s| !s.is_empty() && *s != " ")
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        // Ensure directory exists
        if let Some(parent) = Path::new(DEFAULT_CONFIG_PATH).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(DEFAULT_CONFIG_PATH, content).context("Failed to write config file")?;

        Ok(())
    }

    /// Get interface by name
    pub fn get_interface(&self, name: &str) -> Option<&InterfaceConfig> {
        self.interfaces.get(name)
    }

    /// Get policy by name
    pub fn get_policy(&self, name: &str) -> Option<&PolicyConfig> {
        self.policies.get(name)
    }

    /// Get member by name
    pub fn get_member(&self, name: &str) -> Option<&MemberConfig> {
        self.members.get(name)
    }

    /// Get all enabled interfaces
    pub fn enabled_interfaces(&self) -> impl Iterator<Item = (&String, &InterfaceConfig)> {
        self.interfaces.iter().filter(|(_, c)| c.enabled)
    }

    /// Get all enabled rules
    pub fn enabled_rules(&self) -> impl Iterator<Item = &RuleConfig> {
        self.rules.iter().filter(|r| r.enabled)
    }
}

impl Default for NetPathConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl NetPathConfig {
    pub fn new() -> Self {
        Self {
            global: GlobalConfig::default(),
            interfaces: HashMap::new(),
            members: HashMap::new(),
            policies: HashMap::new(),
            rules: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NetPathConfig::default();
        assert!(config.global.enabled);
        assert_eq!(config.global.nft_table, "netpath");
    }

    #[test]
    fn test_load_balance_mode_serialize() {
        let mode = LoadBalanceMode::Balanced;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"balanced\"");
    }
}
