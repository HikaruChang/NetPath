// Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
// Non-commercial use only. Attribution required.

//! Metrics collection module

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::interface::InterfaceManager;

/// Metrics data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

/// Metrics for an interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceMetrics {
    pub name: String,
    pub latency: Vec<MetricPoint>,
    pub loss: Vec<MetricPoint>,
    pub tx_rate: Vec<MetricPoint>,
    pub rx_rate: Vec<MetricPoint>,
}

/// Metrics collector
pub struct MetricsCollector {
    metrics: HashMap<String, InterfaceMetrics>,
    max_points: usize,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            max_points: 100, // Keep last 100 data points
        }
    }

    pub async fn collect(&mut self, interfaces: &InterfaceManager) {
        let now = Utc::now();

        for (name, iface) in &interfaces.interfaces {
            let metrics = self
                .metrics
                .entry(name.clone())
                .or_insert_with(|| InterfaceMetrics {
                    name: name.clone(),
                    latency: Vec::new(),
                    loss: Vec::new(),
                    tx_rate: Vec::new(),
                    rx_rate: Vec::new(),
                });

            // Add data points
            metrics.latency.push(MetricPoint {
                timestamp: now,
                value: iface.latency,
            });

            metrics.loss.push(MetricPoint {
                timestamp: now,
                value: iface.loss,
            });

            // Limit data points
            if metrics.latency.len() > self.max_points {
                metrics.latency.remove(0);
            }
            if metrics.loss.len() > self.max_points {
                metrics.loss.remove(0);
            }
        }
    }

    pub fn get_metrics(&self, interface: &str) -> Option<&InterfaceMetrics> {
        self.metrics.get(interface)
    }
}
