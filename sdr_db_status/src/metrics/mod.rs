mod collector;

pub use collector::MetricsCollector;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Overall cluster metrics aggregated from all nodes
#[derive(Debug, Clone)]
pub struct ClusterMetrics {
    pub timestamp: DateTime<Utc>,
    pub nodes: Vec<NodeStatus>,
    pub database: DatabaseStats,
    pub signals: SignalStats,
    pub system: SystemStats,
}

impl Default for ClusterMetrics {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            nodes: vec![
                NodeStatus::up("node-1", "192.168.1.101"),
                NodeStatus::up("node-2", "192.168.1.102"),
                NodeStatus::up("node-3", "192.168.1.103"),
            ],
            database: DatabaseStats::default(),
            signals: SignalStats::default(),
            system: SystemStats::default(),
        }
    }
}

/// Status of individual cluster node
#[derive(Debug, Clone)]
pub struct NodeStatus {
    pub name: String,
    pub address: String,
    pub is_up: bool,
    pub request_rate: f32, // requests/sec
    pub error_rate: f32,   // percentage
    pub uptime: Duration,
}

impl NodeStatus {
    pub fn up(name: &str, address: &str) -> Self {
        Self {
            name: name.to_string(),
            address: address.to_string(),
            is_up: true,
            request_rate: 0.0,
            error_rate: 0.0,
            uptime: Duration::from_secs(0),
        }
    }

    pub fn down(name: &str, address: &str) -> Self {
        Self {
            name: name.to_string(),
            address: address.to_string(),
            is_up: false,
            request_rate: 0.0,
            error_rate: 0.0,
            uptime: Duration::from_secs(0),
        }
    }
}

/// Database statistics
#[derive(Debug, Clone, Default)]
pub struct DatabaseStats {
    pub total_logs: u64,
    pub log_rate: f32, // logs/minute
    pub active_connections: u32,
    pub max_connections: u32,
    pub avg_latency_ms: u32,
}

/// RF signal statistics
#[derive(Debug, Clone)]
pub struct SignalStats {
    pub freq_range: (f32, f32), // MHz (min, max)
    pub mode_distribution: ModeDistribution,
    pub unique_grids: u32,
}

impl Default for SignalStats {
    fn default() -> Self {
        Self {
            freq_range: (88.5, 2400.0),
            mode_distribution: ModeDistribution::default(),
            unique_grids: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ModeDistribution {
    pub fm: u32,
    pub am: u32,
    pub usb: u32,
    pub lsb: u32,
    pub cw: u32,
}

impl ModeDistribution {
    pub fn total(&self) -> u32 {
        self.fm + self.am + self.usb + self.lsb + self.cw
    }

    pub fn fm_percent(&self) -> f32 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            (self.fm as f32 / total as f32) * 100.0
        }
    }

    pub fn am_percent(&self) -> f32 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            (self.am as f32 / total as f32) * 100.0
        }
    }

    pub fn other_percent(&self) -> f32 {
        100.0 - self.fm_percent() - self.am_percent()
    }
}

/// System resource statistics
#[derive(Debug, Clone, Default)]
pub struct SystemStats {
    pub cpu_percent: f32,
    pub memory_used_gb: f32,
    pub memory_total_gb: f32,
    pub cpu_history: Vec<f32>, // Last N readings for sparkline
}

impl SystemStats {
    pub fn memory_percent(&self) -> f32 {
        if self.memory_total_gb == 0.0 {
            0.0
        } else {
            (self.memory_used_gb / self.memory_total_gb) * 100.0
        }
    }
}

/// Health check response from API
#[derive(Debug, Deserialize, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
}

/// Logs query response from API
#[derive(Debug, Deserialize, Serialize)]
pub struct LogsResponse {
    pub count: u64,
    pub logs: Vec<LogEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub id: i32,
    pub frequency: f64,
    pub mode: Option<String>,
    pub grid: Option<String>,
    pub timestamp: String,
}
