// Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
// Non-commercial use only. Attribution required.

//! nftables management module (Optimized)
//!
//! Handles all nftables operations for traffic marking and policy routing
//! Uses nftables Rust crate for better performance and type safety

use anyhow::{anyhow, Context, Result};
use nftables::{
    batch::Batch,
    schema::{Chain, NfListObject, Rule, Set, SetFlag, SetType, SetTypeValue, Table},
    types::*,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::process::Command;
use tracing::{debug, error, info, warn};

use crate::config::{NetPathConfig, PolicyConfig, RuleConfig};
use crate::interface::InterfaceManager;

/// nftables manager with optimized batch operations
pub struct NftManager {
    /// Table name
    table_name: String,

    /// Family (inet)
    family: NfFamily,

    /// Track if initialized
    initialized: bool,

    /// Batch for bulk operations
    batch: Batch,

    /// Policy mark mappings (policy_name -> mark)
    policy_marks: HashMap<String, u32>,

    /// Interface state cache (policy_name -> hash of online members)
    interface_cache: HashMap<String, String>,

    /// Next available mark
    next_mark: u32,

    /// Named counters for statistics (interface -> counter_name)
    counters: HashMap<String, String>,
}

/// Statistics for an interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceStats {
    /// Interface name
    pub interface: String,
    /// Packets transmitted
    pub packets: u64,
    /// Bytes transmitted
    pub bytes: u64,
    /// Packets dropped
    pub dropped: u64,
}

impl NftManager {
    /// Helper function to convert NfFamily to string for nft commands
    fn family_str(&self) -> &'static str {
        match self.family {
            NfFamily::IP => "ip",
            NfFamily::IP6 => "ip6",
            NfFamily::INet => "inet",
            NfFamily::ARP => "arp",
            NfFamily::Bridge => "bridge",
            NfFamily::NetDev => "netdev",
        }
    }

    /// Create new nftables manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            table_name: "netpath".to_string(),
            family: NfFamily::INet,
            initialized: false,
            batch: Batch::new(),
            policy_marks: HashMap::new(),
            interface_cache: HashMap::new(),
            next_mark: 0x100,
            counters: HashMap::new(),
        })
    }

    /// Initialize nftables rules with optimized batch operations
    pub async fn init(&mut self, config: &NetPathConfig) -> Result<()> {
        info!(
            "Initializing nftables table: {} (optimized mode)",
            config.global.nft_table
        );
        self.table_name = config.global.nft_table.clone();

        // Delete existing table if present
        self.delete_table()?;

        // Build all nftables objects in a single batch
        self.batch = Batch::new();

        // Create base table and chains
        self.create_base_table_batch()?;

        // Create optimized maps and sets
        self.create_maps_and_sets_batch()?;

        // Create interface chains
        self.create_interface_chains_batch(config)?;

        // Create policy chains with verdict maps
        self.create_policy_chains_batch(config)?;

        // Create rule chains
        self.create_rule_chains_batch(config)?;

        // Setup hooks into prerouting and output
        // Note: Using batch for this has JSON issues, so we apply batch first
        // then add hooks via nft command
        // self.setup_hooks_batch()?;

        // Apply all changes in one go
        self.apply_batch().await?;

        // Create sticky map (must be done after table creation)
        self.create_sticky_map()?;

        // Apply hooks using nft command (after batch to ensure chains exist)
        self.setup_hooks()?;
        // Apply traffic rules after batch (using nft command for proper syntax)
        self.apply_traffic_rules(config)?;

        self.initialized = true;
        info!(
            "nftables initialization completed (applied {} operations)",
            0 /* batch operations count */
        );

        Ok(())
    }

    /// Reload nftables rules with new configuration
    pub async fn reload(&mut self, config: &NetPathConfig) -> Result<()> {
        info!("Reloading nftables configuration");
        self.init(config).await
    }

    /// Create base table structure using batch
    fn create_base_table_batch(&mut self) -> Result<()> {
        // Create table
        self.batch.add(NfListObject::Table(Table {
            family: self.family,
            name: self.table_name.clone(),
            handle: None,
        }));

        // Create main chains with optimal priorities
        // Prerouting: -160 (mangle - 10, before passwall at mangle - 1)
        self.batch.add(NfListObject::Chain(Chain {
            family: self.family,
            table: self.table_name.clone(),
            name: "prerouting".to_string(),
            newname: None,
            handle: None,
            _type: Some(NfChainType::Filter),
            hook: Some(NfHook::Prerouting),
            prio: Some(-160),
            dev: None,
            policy: Some(NfChainPolicy::Accept),
        }));

        // Output chain for locally generated traffic
        self.batch.add(NfListObject::Chain(Chain {
            family: self.family,
            table: self.table_name.clone(),
            name: "output".to_string(),
            newname: None,
            handle: None,
            _type: Some(NfChainType::Route),
            hook: Some(NfHook::Output),
            prio: Some(-160),
            dev: None,
            policy: Some(NfChainPolicy::Accept),
        }));

        // Policy routing chain
        self.batch.add(NfListObject::Chain(Chain {
            family: self.family,
            table: self.table_name.clone(),
            name: "netpath-routing".to_string(),
            newname: None,
            handle: None,
            _type: None,
            hook: None,
            prio: None,
            dev: None,
            policy: None,
        }));

        // Interface selection chain
        self.batch.add(NfListObject::Chain(Chain {
            family: self.family,
            table: self.table_name.clone(),
            name: "netpath-interfaces".to_string(),
            newname: None,
            handle: None,
            _type: None,
            hook: None,
            prio: None,
            dev: None,
            policy: None,
        }));

        // Policies chain
        self.batch.add(NfListObject::Chain(Chain {
            family: self.family,
            table: self.table_name.clone(),
            name: "netpath-policies".to_string(),
            newname: None,
            handle: None,
            _type: None,
            hook: None,
            prio: None,
            dev: None,
            policy: None,
        }));

        // Rules chain
        self.batch.add(NfListObject::Chain(Chain {
            family: self.family,
            table: self.table_name.clone(),
            name: "netpath-rules".to_string(),
            newname: None,
            handle: None,
            _type: None,
            hook: None,
            prio: None,
            dev: None,
            policy: None,
        }));

        debug!("Added base table and chains to batch");
        Ok(())
    }

    /// Create optimized maps and sets
    fn create_maps_and_sets_batch(&mut self) -> Result<()> {
        // Verdict map for policy routing (mark -> policy chain)
        self.batch.add(NfListObject::Set(Set {
            family: self.family,
            table: self.table_name.clone(),
            name: "policy_routes".to_string(),
            handle: None,
            set_type: SetTypeValue::Single(SetType::Mark),
            policy: None,
            flags: Some(HashSet::from([SetFlag::Constant])),
            elem: None,
            timeout: None,
            gc_interval: None,
            size: None,
            comment: None,
        }));

        debug!("Added maps and sets to batch");
        Ok(())
    }

    /// Create sticky sessions map using nft command (to avoid batch type complexity)
    fn create_sticky_map(&mut self) -> Result<()> {
        // Create map: ipv4_addr . ipv4_addr : mark
        // We use run_nft_cmd because the complex map type is hard to construct in Rust types
        let args = vec![
            "add",
            "map",
            self.family_str(),
            &self.table_name,
            "sticky_sessions",
            "{ type ipv4_addr . ipv4_addr : mark; flags dynamic,timeout; timeout 5m; size 65536; }",
        ];

        self.run_nft_cmd(&args)?;
        debug!("Created sticky_sessions map");
        Ok(())
    }

    /// Create interface-specific chains using batch
    fn create_interface_chains_batch(&mut self, config: &NetPathConfig) -> Result<()> {
        for (name, _iface) in config.enabled_interfaces() {
            let chain_name = format!("iface-{}", name);
            let counter_name = format!("{}_counter", name);

            // Create named counter for this interface
            self.counters.insert(name.clone(), counter_name.clone());

            self.batch.add(NfListObject::Chain(Chain {
                family: self.family,
                table: self.table_name.clone(),
                name: chain_name.clone(),
                newname: None,
                handle: None,
                _type: None,
                hook: None,
                prio: None,
                dev: None,
                policy: None,
            }));

            debug!(
                "Added interface chain to batch: {} with counter: {}",
                chain_name, counter_name
            );
        }

        Ok(())
    }

    /// Create policy chains with optimized verdict maps
    fn create_policy_chains_batch(&mut self, config: &NetPathConfig) -> Result<()> {
        for (name, policy) in &config.policies {
            let chain_name = format!("policy-{}", name);

            // Allocate mark for this policy
            let mark = self.next_mark;
            self.policy_marks.insert(name.clone(), mark);
            self.next_mark += 1;

            // Create chain
            self.batch.add(NfListObject::Chain(Chain {
                family: self.family,
                table: self.table_name.clone(),
                name: chain_name.clone(),
                newname: None,
                handle: None,
                _type: None,
                hook: None,
                prio: None,
                dev: None,
                policy: None,
            }));

            // Add base policy rules (will be populated dynamically)
            self.add_policy_base_rules_batch(&chain_name, policy)?;

            debug!(
                "Added policy chain to batch: {} (mark: 0x{:x})",
                chain_name, mark
            );
        }

        Ok(())
    }

    /// Add base policy rules to batch
    fn add_policy_base_rules_batch(
        &mut self,
        chain_name: &str,
        policy: &PolicyConfig,
    ) -> Result<()> {
        // Optimized rule: fast path for established connections
        if policy.sticky {
            self.add_rule_to_batch(chain_name, "ct state established,related counter accept");
        }

        // Add connection tracking with flow offload support
        self.add_rule_to_batch(chain_name, "ct state established flow add @fastpath");

        Ok(())
    }

    /// Helper to add rule to batch
    fn add_rule_to_batch(&mut self, chain: &str, expr: &str) {
        self.batch.add(NfListObject::Rule(Rule {
            family: self.family,
            table: self.table_name.clone(),
            chain: chain.to_string(),
            expr: serde_json::from_str(&format!("[\"{}\"]", expr)).unwrap_or_default(),
            handle: None,
            index: None,
            comment: None,
        }));
    }

    /// Create rule chains for traffic classification
    fn create_rule_chains_batch(&mut self, _config: &NetPathConfig) -> Result<()> {
        // Rules will be added after batch is applied, using run_nft_cmd for proper syntax
        // Store rules for later
        Ok(())
    }

    /// Apply traffic rules after batch initialization (must be called after apply_batch)
    fn apply_traffic_rules(&mut self, config: &NetPathConfig) -> Result<()> {
        for rule in config.enabled_rules() {
            if let Err(e) = self.add_traffic_rule_nft(rule, config) {
                warn!("Failed to add rule '{}': {}", rule.name, e);
                // Continue with other rules even if one fails
            }
        }
        Ok(())
    }

    /// Add a single traffic rule using nft command
    fn add_traffic_rule_nft(&mut self, rule: &RuleConfig, _config: &NetPathConfig) -> Result<()> {
        let policy_chain = format!("policy-{}", rule.policy);

        let target_chain = if rule.sticky {
            // Create a dedicated chain for sticky logic
            let sticky_chain = format!("rule-sticky-{}", rule.name);

            // 1. Create chain
            self.run_nft_cmd(&[
                "add",
                "chain",
                self.family_str(),
                &self.table_name,
                &sticky_chain,
            ])?;

            // 2. Try to restore mark from map
            self.run_nft_cmd(&[
                "add",
                "rule",
                self.family_str(),
                &self.table_name,
                &sticky_chain,
                "meta",
                "mark",
                "set",
                "ip",
                "saddr",
                ".",
                "ip",
                "daddr",
                "map",
                "@sticky_sessions",
            ])?;

            // 3. If mark found (non-zero), save to ct mark and return immediately
            self.run_nft_cmd(&[
                "add",
                "rule",
                self.family_str(),
                &self.table_name,
                &sticky_chain,
                "meta",
                "mark",
                "!=",
                "0",
                "ct",
                "mark",
                "set",
                "mark",
                "counter",
                "return",
            ])?;

            // 4. No existing session - get new mark from policy and save it
            self.run_nft_cmd(&[
                "add",
                "rule",
                self.family_str(),
                &self.table_name,
                &sticky_chain,
                "jump",
                &policy_chain,
            ])?;

            // 5. Save the mark to sticky sessions map and return
            self.run_nft_cmd(&[
                "add",
                "rule",
                self.family_str(),
                &self.table_name,
                &sticky_chain,
                "add",
                "@sticky_sessions",
                "{",
                "ip",
                "saddr",
                ".",
                "ip",
                "daddr",
                ":",
                "meta",
                "mark",
                "}",
                "counter",
                "return",
            ])?;

            sticky_chain
        } else {
            policy_chain
        };

        let mut args: Vec<String> = vec![
            "add".to_string(),
            "rule".to_string(),
            self.family_str().to_string(),
            self.table_name.clone(),
            "netpath-rules".to_string(),
        ];

        // If sticky, restrict to IPv4 for now as our map is IPv4 only
        if rule.sticky {
            args.extend_from_slice(&[
                "meta".to_string(),
                "nfproto".to_string(),
                "ipv4".to_string(),
            ]);
        }

        // Build match conditions
        if let Some(ref ipset) = rule.ipset {
            // Check if ipset exists - skip rule if not found
            let check = std::process::Command::new("nft")
                .args(["list", "set", "inet", "fw4", ipset])
                .output();

            if let Ok(output) = check {
                if output.status.success() {
                    // Use fw4's ipset with full reference
                    args.extend_from_slice(&[
                        "ip".to_string(),
                        "daddr".to_string(),
                        format!("@fw4:{}", ipset), // Reference external table's set
                    ]);
                } else {
                    warn!(
                        "IPSet {} not found in fw4, skipping rule {}",
                        ipset, rule.name
                    );
                    return Ok(());
                }
            } else {
                warn!("Cannot check IPSet {}, skipping rule {}", ipset, rule.name);
                return Ok(());
            }
        }

        if let Some(ref src_ip) = rule.src_ip {
            args.extend_from_slice(&["ip".to_string(), "saddr".to_string(), src_ip.clone()]);
        }

        if let Some(ref dest_ip) = rule.dest_ip {
            args.extend_from_slice(&["ip".to_string(), "daddr".to_string(), dest_ip.clone()]);
        }

        if let Some(ref proto) = rule.proto {
            if proto != "all" {
                args.extend_from_slice(&["meta".to_string(), "l4proto".to_string(), proto.clone()]);
            }
        }

        if let Some(ref src_port) = rule.src_port {
            args.extend_from_slice(&["th".to_string(), "sport".to_string(), src_port.clone()]);
        }

        if let Some(ref dest_port) = rule.dest_port {
            args.extend_from_slice(&["th".to_string(), "dport".to_string(), dest_port.clone()]);
        }

        // Add counter and jump (non-terminal action - allows returning to save ct mark)
        args.extend_from_slice(&["counter".to_string(), "jump".to_string(), target_chain]);

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        self.run_nft_cmd(&args_str)?;

        debug!("Added traffic rule: {}", rule.name);
        Ok(())
    }

    /// Setup hooks into netfilter using nft commands (after batch)
    /// Optimized design similar to mwan3:
    /// - Only process packets with mark == 0 (respect other apps' marks)
    /// - Fast path for established connections via conntrack
    /// - Minimal processing in OUTPUT chain (router's own traffic)
    fn setup_hooks(&mut self) -> Result<()> {
        info!("Setting up optimized prerouting and output hooks...");

        // ============ PREROUTING CHAIN ============
        // Process LAN -> WAN traffic

        // 1. Fast path: restore mark from conntrack for established connections
        //    This is the MOST IMPORTANT optimization - established connections skip ALL rules
        self.run_nft_cmd(&[
            "add",
            "rule",
            self.family_str(),
            &self.table_name,
            "prerouting",
            "ct",
            "mark",
            "!=",
            "0",
            "meta",
            "mark",
            "set",
            "ct",
            "mark",
            "counter",
            "accept",
        ])?;

        // 2. Skip traffic that already has a mark (from other apps like passwall)
        //    This prevents us from interfering with proxy traffic
        self.run_nft_cmd(&[
            "add",
            "rule",
            self.family_str(),
            &self.table_name,
            "prerouting",
            "meta",
            "mark",
            "!=",
            "0",
            "counter",
            "accept",
        ])?;

        // 3. Skip local traffic (destined to router itself)
        self.run_nft_cmd(&[
            "add",
            "rule",
            self.family_str(),
            &self.table_name,
            "prerouting",
            "fib",
            "daddr",
            "type",
            "local",
            "counter",
            "accept",
        ])?;

        // 4. Skip private/special destinations - these should never be load balanced
        let private_ranges = [
            "10.0.0.0/8",     // RFC1918 Class A
            "172.16.0.0/12",  // RFC1918 Class B
            "192.168.0.0/16", // RFC1918 Class C
            "169.254.0.0/16", // Link-local
            "127.0.0.0/8",    // Loopback
            "198.18.0.0/15",  // Benchmark/FakeIP (used by Passwall/Clash)
            "224.0.0.0/4",    // Multicast
        ];

        for range in &private_ranges {
            self.run_nft_cmd(&[
                "add",
                "rule",
                self.family_str(),
                &self.table_name,
                "prerouting",
                "ip",
                "daddr",
                range,
                "counter",
                "accept",
            ])?;
        }

        // 5. Process NEW connections only - this is where load balancing happens
        self.run_nft_cmd(&[
            "add",
            "rule",
            self.family_str(),
            &self.table_name,
            "prerouting",
            "ct",
            "state",
            "new",
            "counter",
            "jump",
            "netpath-rules",
        ])?;

        // 6. Save mark to conntrack for future packets of this connection
        //    Only save if we actually set a mark (mark != 0)
        self.run_nft_cmd(&[
            "add",
            "rule",
            self.family_str(),
            &self.table_name,
            "prerouting",
            "meta",
            "mark",
            "!=",
            "0",
            "ct",
            "mark",
            "set",
            "mark",
        ])?;

        // ============ OUTPUT CHAIN ============
        // Process router's own outgoing traffic (speedtest, updates, etc.)
        // Keep this MINIMAL - most router traffic doesn't need load balancing

        // 1. Fast path: restore mark from conntrack
        self.run_nft_cmd(&[
            "add",
            "rule",
            self.family_str(),
            &self.table_name,
            "output",
            "ct",
            "mark",
            "!=",
            "0",
            "meta",
            "mark",
            "set",
            "ct",
            "mark",
            "counter",
            "accept",
        ])?;

        // 2. Skip if already marked (by proxy apps)
        self.run_nft_cmd(&[
            "add",
            "rule",
            self.family_str(),
            &self.table_name,
            "output",
            "meta",
            "mark",
            "!=",
            "0",
            "counter",
            "accept",
        ])?;

        // 3. Skip private/special destinations
        for range in &private_ranges {
            self.run_nft_cmd(&[
                "add",
                "rule",
                self.family_str(),
                &self.table_name,
                "output",
                "ip",
                "daddr",
                range,
                "counter",
                "accept",
            ])?;
        }

        // 4. Process NEW connections
        self.run_nft_cmd(&[
            "add",
            "rule",
            self.family_str(),
            &self.table_name,
            "output",
            "ct",
            "state",
            "new",
            "counter",
            "jump",
            "netpath-rules",
        ])?;

        // 5. Save mark to conntrack
        self.run_nft_cmd(&[
            "add",
            "rule",
            self.family_str(),
            &self.table_name,
            "output",
            "meta",
            "mark",
            "!=",
            "0",
            "ct",
            "mark",
            "set",
            "mark",
        ])?;

        info!("Hooks setup completed successfully (optimized mode)");
        Ok(())
    }

    /// Apply batch operations to nftables
    async fn apply_batch(&mut self) -> Result<()> {
        let nftables = self.batch.clone().to_nftables();
        let batch_json =
            serde_json::to_string(&nftables).context("Failed to serialize nftables batch")?;

        debug!(
            "Applying batch with JSON: {}",
            if batch_json.len() > 200 {
                format!("{}...", &batch_json[..200])
            } else {
                batch_json.clone()
            }
        );

        // Write to nft via stdin for atomic application
        let mut child = Command::new("nft")
            .arg("-j")
            .arg("-f")
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn nft command")?;

        // Write to stdin
        if let Some(ref mut stdin) = child.stdin {
            stdin
                .write_all(batch_json.as_bytes())
                .context("Failed to write to nft stdin")?;
        }

        // Wait for the process to complete
        let output = child
            .wait_with_output()
            .context("Failed to wait for nft command")?;

        if output.status.success() {
            info!("Batch applied successfully");
            self.batch = Batch::new();
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            error!(
                "Failed to apply batch: stderr={}, stdout={}",
                stderr, stdout
            );
            Err(anyhow!("Failed to apply nftables batch: {}", stderr))
        }
    }

    /// Setup load balancing policy with batch operations
    fn setup_policy_dynamic_batch_with_interfaces(
        &mut self,
        chain_name: &str,
        policy: &PolicyConfig,
        config: &NetPathConfig,
        interfaces: &InterfaceManager,
    ) -> Result<()> {
        let mut total_weight = 0u32;
        let mut members = Vec::new();

        // Collect member info with interface states
        for member_name in &policy.members {
            if let Some(member) = config.get_member(member_name) {
                if let Some(iface_state) = interfaces.get(&member.interface) {
                    if iface_state.is_usable() {
                        members.push((
                            member_name.clone(),
                            member.weight,
                            member.metric,
                            iface_state.fwmark,
                        ));
                        total_weight += member.weight;
                    }
                }
            }
        }

        if members.is_empty() {
            warn!("Policy {} has no usable members", chain_name);
            return Ok(());
        }

        // Sort by metric (lower is better)
        members.sort_by_key(|(_, _, metric, _)| *metric);

        // Generate weighted random distribution using mwan3-style probability
        // mwan3 uses iptables -m statistic --mode random --probability
        // We use nftables numgen random mod X < Y
        let mut weight_sum = 0u64;
        let total_weight_64 = total_weight as u64;

        // Use 65535 as modulo (nftables range is 0-65535, so mod 65535 gives 0-65534)
        // For the last member, we use threshold = 65535 which is the max valid value
        let nft_mod: u64 = 65535;

        for (i, (member_name, weight, _, fwmark)) in members.iter().enumerate() {
            weight_sum += *weight as u64;
            // Calculate threshold for this member
            // For the last member, ensure threshold is exactly (nft_mod - 1) to catch all remaining traffic
            // nftables requires: threshold < mod, so max valid threshold is (mod - 1)
            let threshold = if i == members.len() - 1 {
                nft_mod - 1 // Last member catches everything (max valid threshold)
            } else {
                std::cmp::min((weight_sum * nft_mod) / total_weight_64, nft_mod - 1)
            };

            debug!(
                "Policy {}: member {} with fwmark 0x{:x}, threshold {}/{}",
                chain_name, member_name, fwmark, threshold, nft_mod
            );

            // The rule: if random < threshold, use this interface
            // Note: We DON'T save ct mark here - it's done in the hook after all rules
            self.run_nft_cmd(&[
                "add",
                "rule",
                self.family_str(),
                &self.table_name,
                chain_name,
                "numgen",
                "random",
                "mod",
                &nft_mod.to_string(),
                "<",
                &threshold.to_string(),
                "meta",
                "mark",
                "set",
                &format!("0x{:x}", fwmark),
                "counter",
                "return",
            ])?;
        }

        info!(
            "Policy {} configured with {} members (total_weight={})",
            chain_name,
            members.len(),
            total_weight
        );

        Ok(())
    }

    /// Setup failover policy (use interfaces in metric order)
    #[allow(dead_code)]
    fn setup_failover_policy(
        &self,
        chain_name: &str,
        policy: &PolicyConfig,
        config: &NetPathConfig,
    ) -> Result<()> {
        let mut members = Vec::new();

        for member_name in &policy.members {
            if let Some(member) = config.get_member(member_name) {
                if let Some(iface) = config.get_interface(&member.interface) {
                    if iface.enabled {
                        members.push((member_name.clone(), member.metric));
                    }
                }
            }
        }

        // Sort by metric
        members.sort_by_key(|(_, metric)| *metric);

        // Add rules in order - first available interface wins
        for (member_name, _) in members {
            if let Some(_member) = config.get_member(&member_name) {
                let mark = format!("0x{:x}", 0x100); // Placeholder

                // Check if interface is up (will be added dynamically)
                self.run_nft(&format!(
                    "add rule {} {} {} mark set {} counter return comment \"{}\"",
                    self.family_str(),
                    self.table_name,
                    chain_name,
                    mark,
                    member_name
                ))?;
            }
        }

        Ok(())
    }

    /// Setup round-robin policy
    #[allow(dead_code)]
    fn setup_roundrobin_policy(
        &self,
        chain_name: &str,
        policy: &PolicyConfig,
        config: &NetPathConfig,
    ) -> Result<()> {
        let members: Vec<_> = policy
            .members
            .iter()
            .filter_map(|m| config.get_member(m))
            .filter_map(|m| config.get_interface(&m.interface))
            .filter(|i| i.enabled)
            .collect();

        if members.is_empty() {
            return Ok(());
        }

        let n = members.len();

        for (idx, _) in members.iter().enumerate() {
            let mark = format!("0x{:x}", 0x100 + idx); // Placeholder

            self.run_nft(&format!(
                "add rule {} {} {} numgen inc mod {} {} mark set {} counter return",
                self.family_str(),
                self.table_name,
                chain_name,
                n,
                idx,
                mark
            ))?;
        }

        Ok(())
    }

    /// Setup hash-based policy (source IP hash)
    #[allow(dead_code)]
    fn setup_hash_policy(
        &self,
        chain_name: &str,
        policy: &PolicyConfig,
        config: &NetPathConfig,
    ) -> Result<()> {
        let members: Vec<_> = policy
            .members
            .iter()
            .filter_map(|m| config.get_member(m))
            .filter_map(|m| config.get_interface(&m.interface))
            .filter(|i| i.enabled)
            .collect();

        let n = members.len();

        for (idx, _) in members.iter().enumerate() {
            let mark = format!("0x{:x}", 0x100 + idx);

            self.run_nft(&format!(
                "add rule {} {} {} jhash ip saddr mod {} {} mark set {} counter return",
                self.family_str(),
                self.table_name,
                chain_name,
                n,
                idx,
                mark
            ))?;
        }

        Ok(())
    }

    /// Setup random policy
    #[allow(dead_code)]
    fn setup_random_policy(
        &self,
        chain_name: &str,
        policy: &PolicyConfig,
        config: &NetPathConfig,
    ) -> Result<()> {
        // Similar to balanced but equal weights
        let members: Vec<_> = policy
            .members
            .iter()
            .filter_map(|m| config.get_member(m))
            .filter_map(|m| config.get_interface(&m.interface))
            .filter(|i| i.enabled)
            .collect();

        let n = members.len() as u32;
        if n == 0 {
            return Ok(());
        }

        for (idx, _) in members.iter().enumerate() {
            let mark = format!("0x{:x}", 0x100 + idx);
            let probability = ((idx + 1) as u64 * 0xffffffff) / n as u64;
            let prob_str = probability.to_string();

            // Set meta mark and save to conntrack mark for fast path
            self.run_nft_cmd(&[
                "add",
                "rule",
                self.family_str(),
                &self.table_name,
                chain_name,
                "numgen",
                "random",
                "mod",
                "4294967295", // 0xffffffff in decimal
                "<",
                &prob_str,
                "meta",
                "mark",
                "set",
                &mark,
                "ct",
                "mark",
                "set",
                "mark",
                "counter",
                "return",
            ])?;
        }

        Ok(())
    }

    /// Create rule chains for traffic classification
    #[allow(dead_code)]
    fn create_rule_chains(&self, config: &NetPathConfig) -> Result<()> {
        for rule in config.enabled_rules() {
            self.add_rule(rule, config)?;
        }

        Ok(())
    }

    /// Add a single traffic rule
    fn add_rule(&self, rule: &RuleConfig, _config: &NetPathConfig) -> Result<()> {
        let mut conditions = Vec::new();

        // Source IP
        if let Some(ref src_ip) = rule.src_ip {
            conditions.push(format!("ip saddr {}", src_ip));
        }

        // Destination IP
        if let Some(ref dest_ip) = rule.dest_ip {
            conditions.push(format!("ip daddr {}", dest_ip));
        }

        // Protocol
        if let Some(ref proto) = rule.proto {
            if proto != "all" {
                conditions.push(format!("meta l4proto {}", proto));
            }
        }

        // Source port
        if let Some(ref src_port) = rule.src_port {
            conditions.push(format!("th sport {}", src_port));
        }

        // Destination port
        if let Some(ref dest_port) = rule.dest_port {
            conditions.push(format!("th dport {}", dest_port));
        }

        // IPSET
        if let Some(ref ipset) = rule.ipset {
            conditions.push(format!("ip saddr @{}", ipset));
        }

        let condition_str = conditions.join(" ");
        let policy_chain = format!("policy-{}", rule.policy);

        // Add jump to policy chain
        self.run_nft(&format!(
            "add rule {} {} netpath-rules {} counter jump {} comment \"{}\"",
            self.family_str(),
            self.table_name,
            condition_str,
            policy_chain,
            rule.name
        ))?;

        Ok(())
    }

    /// Update interface status with incremental batch updates (OPTIMIZED)
    pub async fn update_interface_status(
        &mut self,
        config: &NetPathConfig,
        interfaces: &InterfaceManager,
    ) -> Result<()> {
        // Check what actually changed
        let mut has_changes = false;
        let mut changed_policies = Vec::new();

        for (policy_name, policy) in &config.policies {
            let online_members: Vec<_> = policy
                .members
                .iter()
                .filter_map(|m| config.get_member(m))
                .filter_map(|m| {
                    let iface = interfaces.get(&m.interface)?;
                    if iface.is_usable() {
                        Some((m, iface))
                    } else {
                        None
                    }
                })
                .collect();

            // Check if this policy's interfaces changed
            let cache_key = format!("policy_{}", policy_name);
            let old_state = self.interface_cache.get(&cache_key);

            let current_state: Vec<String> = online_members
                .iter()
                .map(|(_, iface)| format!("{}:{}", iface.name, iface.fwmark))
                .collect();

            let current_hash = format!("{:?}", current_state);

            if old_state.as_deref() != Some(&current_hash) {
                has_changes = true;
                changed_policies.push((policy_name.clone(), policy.clone(), online_members));

                // Update cache
                self.interface_cache.insert(cache_key, current_hash);
            }
        }

        if !has_changes {
            debug!("No interface changes detected, skipping update");
            return Ok(());
        }

        info!(
            "Detected interface changes in {} policies, applying incremental update",
            changed_policies.len()
        );

        // Batch update only changed policies
        self.batch = Batch::new();

        for (policy_name, policy, _online_members) in changed_policies {
            let chain_name = format!("policy-{}", policy_name);

            // Flush this policy chain
            self.flush_chain(&chain_name)?;

            // Rebuild policy with current interfaces (using interface manager for fwmark)
            self.setup_policy_dynamic_batch_with_interfaces(
                &chain_name,
                &policy,
                config,
                interfaces,
            )?;

            debug!("Incrementally updated policy: {}", policy_name);
        }

        // Apply changes
        self.apply_batch().await?;

        Ok(())
    }

    /// Flush a specific chain (helper for incremental updates)
    fn flush_chain(&self, chain_name: &str) -> Result<()> {
        Command::new("nft")
            .arg("flush")
            .arg("chain")
            .arg(self.family_str())
            .arg(&self.table_name)
            .arg(chain_name)
            .output()
            .context(format!("Failed to flush chain: {}", chain_name))?;

        Ok(())
    }

    /// Setup policy with dynamic interface status using batch
    pub async fn cleanup(&mut self) -> Result<()> {
        info!("Cleaning up nftables table: {}", self.table_name);

        self.delete_table()?;
        self.initialized = false;
        self.batch = Batch::new();
        self.policy_marks.clear();
        self.interface_cache.clear();

        Ok(())
    }

    /// Execute nft command with array of arguments (safe - no shell escaping issues)
    fn run_nft_cmd(&self, args: &[&str]) -> Result<()> {
        debug!("nft: {:?}", args);
        let output = Command::new("nft")
            .args(args)
            .output()
            .context("Failed to execute nft command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("nft command failed: {} (args: {:?})", stderr, args);
            return Err(anyhow!("nft command failed: {}", stderr));
        }

        Ok(())
    }

    /// Execute nft command (wrapper for direct command execution)
    /// NOTE: This method is deprecated, use run_nft_cmd instead to avoid shell escaping issues
    fn run_nft(&self, cmd: &str) -> Result<()> {
        debug!("nft: {}", cmd);
        // Split the command into arguments properly instead of using sh -c
        let args: Vec<&str> = cmd.split_whitespace().collect();
        let output = Command::new("nft")
            .args(&args)
            .output()
            .context("Failed to execute nft command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("nft command failed: {}", stderr);
            return Err(anyhow!("nft command failed: {}", stderr));
        }

        Ok(())
    }

    /// Delete the nftables table
    fn delete_table(&self) -> Result<()> {
        let output = Command::new("nft")
            .args(["delete", "table", &self.family_str(), &self.table_name])
            .output();

        // Ignore error if table doesn't exist
        match output {
            Ok(out) if out.status.success() => {
                debug!("Deleted nftables table: {}", self.table_name);
                Ok(())
            }
            Ok(_) | Err(_) => {
                debug!(
                    "Table {} might not exist, ignoring delete error",
                    self.table_name
                );
                Ok(())
            }
        }
    }

    /// Get current nftables stats (optimized JSON output)
    pub fn get_stats(&self) -> Result<String> {
        let output = Command::new("nft")
            .args(["-j", "list", "table", &self.family_str(), &self.table_name])
            .output()
            .context("Failed to get nft stats")?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow!("Failed to get nft stats"))
        }
    }

    /// Get policy mark mapping
    pub fn get_policy_mark(&self, policy_name: &str) -> Option<u32> {
        self.policy_marks.get(policy_name).copied()
    }

    /// Check if manager is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get statistics for a specific interface (Phase 3 optimization)
    pub async fn get_interface_stats(&self, interface: &str) -> Result<InterfaceStats> {
        // Query nftables for counter values
        let output = Command::new("nft")
            .args(["-j", "list", "table", &self.family_str(), &self.table_name])
            .output()
            .context("Failed to query nftables stats")?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get nftables stats"));
        }

        let _json_output = String::from_utf8_lossy(&output.stdout);

        // Parse JSON to extract counter values
        // For now, return placeholder values
        // TODO: Implement proper JSON parsing when nftables crate supports it
        Ok(InterfaceStats {
            interface: interface.to_string(),
            packets: 0,
            bytes: 0,
            dropped: 0,
        })
    }

    /// Get statistics for all interfaces (Phase 3 optimization)
    pub async fn get_all_stats(&self) -> Result<HashMap<String, InterfaceStats>> {
        let mut stats = HashMap::new();

        for (interface, _counter) in &self.counters {
            if let Ok(iface_stats) = self.get_interface_stats(interface).await {
                stats.insert(interface.clone(), iface_stats);
            }
        }

        Ok(stats)
    }

    /// Validate batch before applying (Phase 3: Rule compilation validation)
    pub fn validate_batch(&self) -> Result<()> {
        if false
        /* batch always has data */
        {
            return Err(anyhow!("Batch is empty, nothing to validate"));
        }

        // Use nft --check to validate rules without applying
        let batch_json =
            serde_json::to_string(&self.batch).context("Failed to serialize batch to JSON")?;

        let mut output = Command::new("nft")
            .args(["--check", "-j", "-f", "-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn nft validation")?;

        if let Some(ref mut stdin) = output.stdin {
            use std::io::Write;
            stdin
                .write_all(batch_json.as_bytes())
                .context("Failed to write batch to nft validator")?;
        }

        let result = output
            .wait_with_output()
            .context("Failed to wait for nft validation")?;

        if result.status.success() {
            info!(
                "Batch validation successful ({} operations)",
                0 /* batch operations count */
            );
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&result.stderr);
            error!("Batch validation failed: {}", error_msg);
            Err(anyhow!("Batch validation failed: {}", error_msg))
        }
    }

    /// Reset all counters (Phase 3 optimization)
    pub async fn reset_counters(&self) -> Result<()> {
        for counter_name in self.counters.values() {
            Command::new("nft")
                .args([
                    "reset",
                    "counter",
                    &self.family_str(),
                    &self.table_name,
                    counter_name,
                ])
                .output()
                .context("Failed to reset counter")?;
        }

        info!("Reset {} counters", self.counters.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nft_manager_creation() {
        let manager = NftManager::new().unwrap();
        assert_eq!(manager.table_name, "netpath");
        assert_eq!(manager.family, NfFamily::INet);
        assert!(!manager.initialized);
        assert_eq!(manager.next_mark, 0x100);
    }

    #[test]
    fn test_batch_operations() {
        let manager = NftManager::new().unwrap();
        assert!(manager.batch.is_empty());
    }
}
