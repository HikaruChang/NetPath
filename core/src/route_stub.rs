// Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
// Non-commercial use only. Attribution required.

//! Stub implementation of RouteManager for non-Linux platforms

use anyhow::Result;
use crate::config::NetPathConfig;

#[derive(Debug)]
pub struct RouteManager;

impl RouteManager {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub async fn init(&mut self, _config: &NetPathConfig) -> Result<()> {
        Ok(())
    }

    pub async fn reload(&mut self, _config: &NetPathConfig) -> Result<()> {
        Ok(())
    }

    pub async fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}
