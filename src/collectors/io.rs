use crate::core::{Result, Error};
use super::Collector;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct IoStats {
    pub device: String,
    pub reads_completed: u64,
    pub writes_completed: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub read_bytes_per_sec: f64,
    pub write_bytes_per_sec: f64,
}

#[derive(Debug, Clone)]
pub struct ProcessIoStats {
    pub pid: u32,
    pub command: String,
    pub read_bytes: u64,
    pub write_bytes: u64,
}

#[derive(Debug, Clone)]
struct DiskSnapshot {
    reads_completed: u64,
    sectors_read: u64,
    writes_completed: u64,
    sectors_written: u64,
    timestamp: Instant,
}

pub struct IoCollector {
    prev_snapshots: HashMap<String, DiskSnapshot>,
}

impl IoCollector {
    pub fn new() -> Self {
        Self {
            prev_snapshots: HashMap::new(),
        }
    }

    fn parse_diskstats(&self, content: &str) -> Result<HashMap<String, DiskSnapshot>> {
        let mut snapshots = HashMap::new();
        let now = Instant::now();

        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 14 {
                continue;
            }

            let device = parts[2].to_string();

            // Skip loop devices and partitions for cleaner output
            // Only show main block devices (sda, nvme0n1, etc.)
            if device.starts_with("loop") || device.starts_with("ram") {
                continue;
            }

            // Parse disk stats
            // Field 3: reads completed
            // Field 5: sectors read
            // Field 7: writes completed
            // Field 9: sectors written
            let reads_completed = parts[3].parse::<u64>().unwrap_or(0);
            let sectors_read = parts[5].parse::<u64>().unwrap_or(0);
            let writes_completed = parts[7].parse::<u64>().unwrap_or(0);
            let sectors_written = parts[9].parse::<u64>().unwrap_or(0);

            snapshots.insert(
                device,
                DiskSnapshot {
                    reads_completed,
                    sectors_read,
                    writes_completed,
                    sectors_written,
                    timestamp: now,
                },
            );
        }

        Ok(snapshots)
    }

    fn calculate_rates(
        &self,
        device: &str,
        current: &DiskSnapshot,
        previous: &DiskSnapshot,
    ) -> (f64, f64) {
        let time_delta = current
            .timestamp
            .duration_since(previous.timestamp)
            .as_secs_f64();

        if time_delta == 0.0 {
            return (0.0, 0.0);
        }

        // Each sector is 512 bytes
        let bytes_read = current.sectors_read.saturating_sub(previous.sectors_read) * 512;
        let bytes_written = current.sectors_written.saturating_sub(previous.sectors_written) * 512;

        let read_rate = bytes_read as f64 / time_delta;
        let write_rate = bytes_written as f64 / time_delta;

        (read_rate, write_rate)
    }

    async fn collect_process_io(&self) -> Result<Vec<ProcessIoStats>> {
        // TODO: Implement per-process I/O stats from /proc/[pid]/io
        // This requires reading each process's io file
        Ok(Vec::new())
    }
}

#[async_trait]
impl Collector for IoCollector {
    type Output = Vec<IoStats>;

    async fn collect(&mut self) -> Result<Self::Output> {
        let content = fs::read_to_string("/proc/diskstats")
            .map_err(|e| Error::CollectionError(format!("Failed to read /proc/diskstats: {}", e)))?;

        let current_snapshots = self.parse_diskstats(&content)?;
        let mut stats = Vec::new();

        for (device, current) in &current_snapshots {
            let (read_rate, write_rate) = if let Some(previous) = self.prev_snapshots.get(device) {
                self.calculate_rates(device, current, previous)
            } else {
                (0.0, 0.0)
            };

            // Calculate total bytes (cumulative)
            let bytes_read = current.sectors_read * 512;
            let bytes_written = current.sectors_written * 512;

            stats.push(IoStats {
                device: device.clone(),
                reads_completed: current.reads_completed,
                writes_completed: current.writes_completed,
                bytes_read,
                bytes_written,
                read_bytes_per_sec: read_rate,
                write_bytes_per_sec: write_rate,
            });
        }

        // Update snapshots for next collection
        self.prev_snapshots = current_snapshots;

        // Sort by total I/O activity (read + write rate)
        stats.sort_by(|a, b| {
            let a_total = a.read_bytes_per_sec + a.write_bytes_per_sec;
            let b_total = b.read_bytes_per_sec + b.write_bytes_per_sec;
            b_total.partial_cmp(&a_total).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(stats)
    }
}
