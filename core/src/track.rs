// Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
// Non-commercial use only. Attribution required.

//! Interface health tracking module

use anyhow::Result;

use crate::config::{NetPathConfig, TrackConfig};
use crate::interface::InterfaceManager;

/// Track manager for monitoring interface health
pub struct TrackManager {
    tracks: Vec<TrackInstance>,
}

/// Instance of a tracking configuration
#[derive(Debug, Clone)]
struct TrackInstance {
    interface: String,
    config: TrackConfig,
    consecutive_success: u32,
    consecutive_failure: u32,
}

impl TrackManager {
    pub fn new(config: &NetPathConfig) -> Result<Self> {
        let mut tracks = Vec::new();

        for (name, iface) in &config.interfaces {
            if iface.enabled && iface.track.method != "none" {
                let instance = TrackInstance {
                    interface: name.clone(),
                    config: iface.track.clone(),
                    consecutive_success: 0,
                    consecutive_failure: 0,
                };
                tracks.push(instance);
            }
        }

        Ok(Self { tracks })
    }

    pub fn reload(&mut self, config: &NetPathConfig) -> Result<()> {
        self.tracks.clear();

        for (name, iface) in &config.interfaces {
            if iface.enabled && iface.track.method != "none" {
                let instance = TrackInstance {
                    interface: name.clone(),
                    config: iface.track.clone(),
                    consecutive_success: 0,
                    consecutive_failure: 0,
                };
                self.tracks.push(instance);
            }
        }

        Ok(())
    }

    /// Check all tracked interfaces
    pub async fn check_all(&mut self, _interfaces: &InterfaceManager) -> Result<()> {
        // Simplified version: health checks disabled for compilation
        // TODO: Implement proper health checking
        Ok(())
    }

    /// Get track status for an interface
    pub fn get_status(&self, interface: &str) -> Option<(u32, u32)> {
        self.tracks.iter()
            .find(|t| t.interface == interface)
            .map(|t| (t.consecutive_success, t.consecutive_failure))
    }

    /// Should interface be marked as up?
    pub fn should_mark_up(&self, interface: &str) -> bool {
        if let Some(track) = self.tracks.iter().find(|t| t.interface == interface) {
            track.consecutive_success >= track.config.up
        } else {
            false
        }
    }

    /// Should interface be marked as down?
    pub fn should_mark_down(&self, interface: &str) -> bool {
        if let Some(track) = self.tracks.iter().find(|t| t.interface == interface) {
            track.consecutive_failure >= track.config.down
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_manager_creation() {
        let config = NetPathConfig::default();
        let manager = TrackManager::new(&config);
        assert!(manager.is_ok());
    }
}
