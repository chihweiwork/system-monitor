use crate::core::{Result, Error};
use super::Collector;
use async_trait::async_trait;
use std::fs;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub interface: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_bytes_per_sec: f64,
    pub tx_bytes_per_sec: f64,
}

#[derive(Debug, Clone)]
struct InterfaceSnapshot {
    rx_bytes: u64,
    tx_bytes: u64,
    timestamp: Instant,
}

pub struct NetworkCollector {
    prev_snapshots: HashMap<String, InterfaceSnapshot>,
}

impl NetworkCollector {
    pub fn new() -> Self {
        Self {
            prev_snapshots: HashMap::new(),
        }
    }

    fn parse_net_dev(&self, content: &str) -> Result<Vec<(String, u64, u64, u64, u64)>> {
        let mut interfaces = Vec::new();

        // Skip first two header lines
        let lines: Vec<&str> = content.lines().skip(2).collect();

        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Format: "interface: rx_bytes rx_packets ... tx_bytes tx_packets ..."
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 10 {
                continue;
            }

            // Interface name (remove trailing colon)
            let interface = parts[0].trim_end_matches(':').to_string();

            // Skip loopback interface
            if interface == "lo" {
                continue;
            }

            // Parse statistics
            let rx_bytes = parts[1].parse::<u64>()
                .map_err(|_| Error::ParseError(format!("Invalid rx_bytes for {}", interface)))?;
            let rx_packets = parts[2].parse::<u64>()
                .map_err(|_| Error::ParseError(format!("Invalid rx_packets for {}", interface)))?;
            let tx_bytes = parts[9].parse::<u64>()
                .map_err(|_| Error::ParseError(format!("Invalid tx_bytes for {}", interface)))?;
            let tx_packets = parts[10].parse::<u64>()
                .map_err(|_| Error::ParseError(format!("Invalid tx_packets for {}", interface)))?;

            interfaces.push((interface, rx_bytes, tx_bytes, rx_packets, tx_packets));
        }

        Ok(interfaces)
    }

    fn calculate_bandwidth(
        &mut self,
        interface: &str,
        rx_bytes: u64,
        tx_bytes: u64,
        now: Instant,
    ) -> (f64, f64) {
        let mut rx_per_sec = 0.0;
        let mut tx_per_sec = 0.0;

        if let Some(prev) = self.prev_snapshots.get(interface) {
            let elapsed = now.duration_since(prev.timestamp).as_secs_f64();
            if elapsed > 0.0 {
                // Calculate bytes per second
                let rx_delta = rx_bytes.saturating_sub(prev.rx_bytes);
                let tx_delta = tx_bytes.saturating_sub(prev.tx_bytes);

                rx_per_sec = rx_delta as f64 / elapsed;
                tx_per_sec = tx_delta as f64 / elapsed;
            }
        }

        // Update snapshot
        self.prev_snapshots.insert(
            interface.to_string(),
            InterfaceSnapshot {
                rx_bytes,
                tx_bytes,
                timestamp: now,
            },
        );

        (rx_per_sec, tx_per_sec)
    }
}

#[async_trait]
impl Collector for NetworkCollector {
    type Output = Vec<NetworkStats>;

    async fn collect(&mut self) -> Result<Self::Output> {
        let content = fs::read_to_string("/proc/net/dev")
            .map_err(|e| Error::Io(e))?;

        let interfaces = self.parse_net_dev(&content)?;
        let now = Instant::now();

        let mut stats = Vec::new();
        for (interface, rx_bytes, tx_bytes, rx_packets, tx_packets) in interfaces {
            let (rx_bytes_per_sec, tx_bytes_per_sec) =
                self.calculate_bandwidth(&interface, rx_bytes, tx_bytes, now);

            stats.push(NetworkStats {
                interface,
                rx_bytes,
                tx_bytes,
                rx_packets,
                tx_packets,
                rx_bytes_per_sec,
                tx_bytes_per_sec,
            });
        }

        Ok(stats)
    }
}
