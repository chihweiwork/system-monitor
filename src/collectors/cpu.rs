use crate::core::{Result, Error};
use super::Collector;
use async_trait::async_trait;
use std::fs;

#[derive(Debug, Clone)]
pub struct CoreStats {
    pub core_id: usize,
    pub usage_percent: f64,
    pub user_time: u64,
    pub system_time: u64,
    pub idle_time: u64,
    pub iowait_time: u64,
    pub irq_time: u64,
    pub softirq_time: u64,
    pub steal_time: u64,
}

#[derive(Debug, Clone)]
pub struct CpuStats {
    pub usage_percent: f64,
    pub user_time: u64,
    pub nice_time: u64,
    pub system_time: u64,
    pub idle_time: u64,
    pub iowait_time: u64,
    pub irq_time: u64,
    pub softirq_time: u64,
    pub steal_time: u64,
    pub cores: Vec<CoreStats>,
    pub core_count: usize,
}

#[derive(Debug, Clone)]
pub struct CpuTimes {
    pub total: u64,
    pub idle: u64,
}

#[derive(Debug, Clone)]
struct CorePrevState {
    core_id: usize,
    prev_total: u64,
    prev_idle: u64,
}

pub struct CpuCollector {
    prev_total: u64,
    prev_idle: u64,
    prev_cores: Vec<CorePrevState>,
}

impl CpuCollector {
    pub fn new() -> Self {
        Self {
            prev_total: 0,
            prev_idle: 0,
            prev_cores: Vec::new(),
        }
    }

    fn parse_cpu_line(&self, line: &str) -> Result<(u64, u64, u64, u64, u64, u64, u64, u64)> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return Err(Error::ParseError("Insufficient CPU time fields".to_string()));
        }

        let user = parts[1].parse::<u64>()
            .map_err(|_| Error::ParseError("Invalid user time".to_string()))?;
        let nice = parts[2].parse::<u64>()
            .map_err(|_| Error::ParseError("Invalid nice time".to_string()))?;
        let system = parts[3].parse::<u64>()
            .map_err(|_| Error::ParseError("Invalid system time".to_string()))?;
        let idle = parts[4].parse::<u64>()
            .map_err(|_| Error::ParseError("Invalid idle time".to_string()))?;

        let iowait = if parts.len() > 5 { parts[5].parse::<u64>().unwrap_or(0) } else { 0 };
        let irq = if parts.len() > 6 { parts[6].parse::<u64>().unwrap_or(0) } else { 0 };
        let softirq = if parts.len() > 7 { parts[7].parse::<u64>().unwrap_or(0) } else { 0 };
        let steal = if parts.len() > 8 { parts[8].parse::<u64>().unwrap_or(0) } else { 0 };

        Ok((user, nice, system, idle, iowait, irq, softirq, steal))
    }

    fn parse_proc_stat(&self, content: &str) -> Result<CpuStats> {
        let mut lines = content.lines();

        let first_line = lines
            .next()
            .ok_or_else(|| Error::ParseError("Empty /proc/stat".to_string()))?;

        if !first_line.starts_with("cpu ") {
            return Err(Error::ParseError("Invalid /proc/stat format".to_string()));
        }

        // Parse aggregate CPU line
        let (user, nice, system, idle, iowait, irq, softirq, steal) =
            self.parse_cpu_line(first_line)?;

        // Parse per-core CPU lines
        let mut cores = Vec::new();
        for line in lines {
            if !line.starts_with("cpu") {
                break;
            }

            // Extract core ID from "cpuN"
            if let Some(cpu_part) = line.split_whitespace().next() {
                if cpu_part == "cpu" {
                    continue; // Skip aggregate line if encountered again
                }

                if let Some(core_id_str) = cpu_part.strip_prefix("cpu") {
                    if let Ok(core_id) = core_id_str.parse::<usize>() {
                        let (core_user, _nice, core_system, core_idle,
                             core_iowait, core_irq, core_softirq, core_steal) =
                            self.parse_cpu_line(line)?;

                        cores.push(CoreStats {
                            core_id,
                            usage_percent: 0.0, // Will be calculated
                            user_time: core_user,
                            system_time: core_system,
                            idle_time: core_idle,
                            iowait_time: core_iowait,
                            irq_time: core_irq,
                            softirq_time: core_softirq,
                            steal_time: core_steal,
                        });
                    }
                }
            }
        }

        let core_count = cores.len();

        Ok(CpuStats {
            usage_percent: 0.0, // Will be calculated
            user_time: user,
            nice_time: nice,
            system_time: system,
            idle_time: idle,
            iowait_time: iowait,
            irq_time: irq,
            softirq_time: softirq,
            steal_time: steal,
            cores,
            core_count,
        })
    }

    fn calculate_usage(&mut self, stats: &mut CpuStats) -> f64 {
        let total = stats.user_time + stats.nice_time + stats.system_time +
                   stats.idle_time + stats.iowait_time + stats.irq_time +
                   stats.softirq_time + stats.steal_time;

        let idle_with_iowait = stats.idle_time + stats.iowait_time;

        if self.prev_total == 0 {
            self.prev_total = total;
            self.prev_idle = idle_with_iowait;

            // Initialize per-core prev state
            self.prev_cores.clear();
            for core in &stats.cores {
                let core_total = core.user_time + core.system_time + core.idle_time +
                                core.iowait_time + core.irq_time + core.softirq_time +
                                core.steal_time;
                let core_idle = core.idle_time + core.iowait_time;

                self.prev_cores.push(CorePrevState {
                    core_id: core.core_id,
                    prev_total: core_total,
                    prev_idle: core_idle,
                });
            }

            return 0.0;
        }

        let total_delta = total.saturating_sub(self.prev_total).max(1);
        let idle_delta = idle_with_iowait.saturating_sub(self.prev_idle);

        self.prev_total = total;
        self.prev_idle = idle_with_iowait;

        let usage = ((total_delta - idle_delta) as f64 / total_delta as f64 * 100.0)
            .clamp(0.0, 100.0);

        // Calculate per-core usage
        for core in &mut stats.cores {
            let core_total = core.user_time + core.system_time + core.idle_time +
                            core.iowait_time + core.irq_time + core.softirq_time +
                            core.steal_time;
            let core_idle = core.idle_time + core.iowait_time;

            // Find or create prev state for this core
            let prev_state = self.prev_cores.iter_mut()
                .find(|p| p.core_id == core.core_id);

            if let Some(prev) = prev_state {
                let core_total_delta = core_total.saturating_sub(prev.prev_total).max(1);
                let core_idle_delta = core_idle.saturating_sub(prev.prev_idle);

                core.usage_percent = ((core_total_delta - core_idle_delta) as f64 /
                                     core_total_delta as f64 * 100.0).clamp(0.0, 100.0);

                prev.prev_total = core_total;
                prev.prev_idle = core_idle;
            } else {
                // New core detected - add to tracking
                self.prev_cores.push(CorePrevState {
                    core_id: core.core_id,
                    prev_total: core_total,
                    prev_idle: core_idle,
                });
                core.usage_percent = 0.0;
            }
        }

        usage
    }
}

#[async_trait]
impl Collector for CpuCollector {
    type Output = CpuStats;

    async fn collect(&mut self) -> Result<Self::Output> {
        let content = fs::read_to_string("/proc/stat")
            .map_err(|e| Error::Io(e))?;

        let mut stats = self.parse_proc_stat(&content)?;
        stats.usage_percent = self.calculate_usage(&mut stats);

        Ok(stats)
    }
}
