// Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
// Non-commercial use only. Attribution required.

//! Policy management module

use anyhow::Result;
use std::collections::HashMap;

use crate::config::{NetPathConfig, PolicyConfig};

/// Policy manager
pub struct PolicyManager {
    policies: HashMap<String, PolicyRuntime>,
}

/// Runtime policy state
#[derive(Debug, Clone)]
pub struct PolicyRuntime {
    pub name: String,
    pub config: PolicyConfig,
    pub active_members: Vec<String>,
}

impl PolicyManager {
    pub fn new(config: &NetPathConfig) -> Result<Self> {
        let mut policies = HashMap::new();

        for (name, policy_config) in &config.policies {
            let runtime = PolicyRuntime {
                name: name.clone(),
                config: policy_config.clone(),
                active_members: Vec::new(),
            };
            policies.insert(name.clone(), runtime);
        }

        Ok(Self { policies })
    }

    pub async fn init(&mut self, _config: &NetPathConfig) -> Result<()> {
        Ok(())
    }

    pub async fn reload(&mut self, config: &NetPathConfig) -> Result<()> {
        self.policies.clear();

        for (name, policy_config) in &config.policies {
            let runtime = PolicyRuntime {
                name: name.clone(),
                config: policy_config.clone(),
                active_members: Vec::new(),
            };
            self.policies.insert(name.clone(), runtime);
        }

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&PolicyRuntime> {
        self.policies.get(name)
    }
}
