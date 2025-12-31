// Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
// Non-commercial use only. Attribution required.

//! Stub implementation of NftManager for non-Linux platforms

use anyhow::Result;
use crate::config::NetPathConfig;
use crate::interface::InterfaceManager;

#[derive(Debug)]
pub struct NftManager;

impl NftManager {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub async fn init(&mut self, _config: &NetPathConfig) -> Result<()> {
        Ok(())
    }

    pub async fn reload(&mut self, _config: &NetPathConfig) -> Result<()> {
        Ok(())
    }

    pub async fn update_interface_status(&mut self, _config: &NetPathConfig, _interfaces: &InterfaceManager) -> Result<()> {
        Ok(())
    }

    pub fn get_stats(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({}))
    }

    pub async fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}
