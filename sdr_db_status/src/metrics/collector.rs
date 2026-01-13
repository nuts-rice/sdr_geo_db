use super::{
    ClusterMetrics, DatabaseStats, HealthResponse, LogsResponse, NodeStatus, SignalStats,
    SystemStats,
};
use anyhow::Result;
use chrono::Utc;
use std::time::Duration;
use sysinfo::System;

/// Collects metrics from cluster nodes and local system
pub struct MetricsCollector {
    api_urls: Vec<String>,
    http_client: reqwest::Client,
    system: System,
}

impl MetricsCollector {
    pub fn new(api_urls: Vec<String>) -> Self {
        Self {
            api_urls,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
            system: System::new_all(),
        }
    }

    /// Collect all metrics from cluster and system
    pub async fn collect(&mut self) -> Result<ClusterMetrics> {
        // Refresh system info
        self.system.refresh_all();

        // Collect node status from each API
        let mut nodes = Vec::new();
        for url in &self.api_urls {
            match self.check_node_health(url).await {
                Ok(status) => nodes.push(status),
                Err(e) => {
                    eprintln!("Failed to check node {}: {}", url, e);
                    nodes.push(NodeStatus::down(&extract_node_name(url), url));
                }
            }
        }

        // Collect database stats from first available node
        let db_stats = self.collect_database_stats().await?;

        // Collect signal stats
        let signal_stats = self.collect_signal_stats().await?;

        // Collect local system stats
        let system_stats = self.collect_system_stats();

        Ok(ClusterMetrics {
            timestamp: Utc::now(),
            nodes,
            database: db_stats,
            signals: signal_stats,
            system: system_stats,
        })
    }

    async fn check_node_health(&self, url: &str) -> Result<NodeStatus> {
        let health_url = format!("{}/health", url);
        let response = self.http_client.get(&health_url).send().await?;

        if response.status().is_success() {
            let _health: HealthResponse = response.json().await?;
            Ok(NodeStatus::up(&extract_node_name(url), url))
        } else {
            Ok(NodeStatus::down(&extract_node_name(url), url))
        }
    }

    async fn collect_database_stats(&self) -> Result<DatabaseStats> {
        // Find first healthy node
        for url in &self.api_urls {
            if let Ok(stats) = self.fetch_db_stats(url).await {
                return Ok(stats);
            }
        }

        // Return defaults if no nodes available
        Ok(DatabaseStats::default())
    }

    async fn fetch_db_stats(&self, url: &str) -> Result<DatabaseStats> {
        let logs_url = format!("{}/logs?limit=1", url);
        let response = self.http_client.get(&logs_url).send().await?;
        let logs: LogsResponse = response.json().await?;

        Ok(DatabaseStats {
            total_logs: logs.count,
            log_rate: 0.0,         // TODO: Calculate from time-series data
            active_connections: 0, // TODO: Add metrics endpoint
            max_connections: 10,
            avg_latency_ms: 0, // TODO: Measure request latency
        })
    }

    async fn collect_signal_stats(&self) -> Result<SignalStats> {
        // TODO: Implement signal statistics collection
        // This would query the API for mode distribution, frequency ranges, etc.
        Ok(SignalStats::default())
    }

    fn collect_system_stats(&self) -> SystemStats {
        let cpu_percent = self.system.global_cpu_usage();
        let memory_used = self.system.used_memory() as f32 / 1_073_741_824.0; // Convert to GB
        let memory_total = self.system.total_memory() as f32 / 1_073_741_824.0;

        SystemStats {
            cpu_percent,
            memory_used_gb: memory_used,
            memory_total_gb: memory_total,
            cpu_history: vec![cpu_percent], // TODO: Maintain history
        }
    }
}

fn extract_node_name(url: &str) -> String {
    // Extract node name from URL (e.g., "http://node-1:8080" -> "node-1")
    url.split("://")
        .nth(1)
        .and_then(|s| s.split(':').next())
        .unwrap_or("unknown")
        .to_string()
}
