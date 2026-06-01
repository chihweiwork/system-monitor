use crate::core::{Result, Error};
use super::Collector;
use async_trait::async_trait;
use std::fs;

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub free: u64,
    pub buffers: u64,
    pub cached: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub swap_free: u64,
}

pub struct MemoryCollector;

impl MemoryCollector {
    pub fn new() -> Self {
        Self
    }

    fn parse_meminfo(&self, content: &str) -> Result<MemoryStats> {
        let mut total = 0u64;
        let mut free = 0u64;
        let mut available = 0u64;
        let mut buffers = 0u64;
        let mut cached = 0u64;
        let mut swap_total = 0u64;
        let mut swap_free = 0u64;
        let mut got_available = false;

        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let label = parts[0];
            let value = parts[1].parse::<u64>()
                .map_err(|_| Error::ParseError(format!("Invalid value for {}", label)))?;

            // Values in /proc/meminfo are in kB, convert to bytes
            let value_bytes = value * 1024;

            match label {
                "MemTotal:" => total = value_bytes,
                "MemFree:" => free = value_bytes,
                "MemAvailable:" => {
                    available = value_bytes;
                    got_available = true;
                }
                "Buffers:" => buffers = value_bytes,
                "Cached:" => cached = value_bytes,
                "SwapTotal:" => swap_total = value_bytes,
                "SwapFree:" => swap_free = value_bytes,
                _ => {}
            }
        }

        // If MemAvailable is not present, estimate it
        if !got_available {
            available = free + cached;
        }

        let used = total.saturating_sub(available.min(total));
        let swap_used = swap_total.saturating_sub(swap_free);

        Ok(MemoryStats {
            total,
            used,
            available,
            free,
            buffers,
            cached,
            swap_total,
            swap_used,
            swap_free,
        })
    }
}

#[async_trait]
impl Collector for MemoryCollector {
    type Output = MemoryStats;

    async fn collect(&mut self) -> Result<Self::Output> {
        let content = fs::read_to_string("/proc/meminfo")
            .map_err(|e| Error::Io(e))?;

        self.parse_meminfo(&content)
    }
}
