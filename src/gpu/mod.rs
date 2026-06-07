// GPU monitoring with multi-vendor support
// Architecture inspired by nvtop

pub mod nvidia;
pub mod amd;
pub mod intel;

use crate::core::{Result, Error};
use crate::collectors::Collector;
use async_trait::async_trait;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum GpuProcessType {
    Graphics,     // Graphics rendering
    Compute,      // Compute tasks
    Both,         // Both graphics and compute
}

#[derive(Debug, Clone)]
pub struct GpuProcess {
    pub pid: u32,
    pub process_name: String,
    pub gpu_memory_mb: u64,        // GPU memory usage (MB)
    pub gpu_utilization: u32,      // GPU utilization (0-100)
    pub process_type: GpuProcessType,
}

#[derive(Debug, Clone)]
pub struct GpuStats {
    pub name: String,
    pub vendor: String,  // "NVIDIA", "AMD", "Intel"
    pub utilization_percent: f64,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub temperature_c: f64,
    pub power_watts: f64,
    pub processes: Vec<GpuProcess>,  // Processes running on this GPU
    pub gpu_id: u32,                 // GPU ID
}

// Note: We can't use async fn in traits for dyn dispatch
// Instead, we'll use concrete types for each backend
pub struct GpuCollector {
    nvidia_backend: Option<nvidia::NvidiaBackend>,
    amd_backend: Option<amd::AmdBackend>,
    intel_backend: Option<intel::IntelBackend>,
}

impl GpuCollector {
    pub async fn new() -> Self {
        let nvidia_backend = if nvidia::NvidiaBackend::detect().await.unwrap_or(false) {
            Some(nvidia::NvidiaBackend::new())
        } else {
            None
        };

        let amd_backend = if amd::AmdBackend::detect().await.unwrap_or(false) {
            Some(amd::AmdBackend::new())
        } else {
            None
        };

        let intel_backend = if intel::IntelBackend::detect().await.unwrap_or(false) {
            Some(intel::IntelBackend::new())
        } else {
            None
        };

        Self {
            nvidia_backend,
            amd_backend,
            intel_backend,
        }
    }

    pub fn has_gpus(&self) -> bool {
        self.nvidia_backend.is_some() || self.amd_backend.is_some() || self.intel_backend.is_some()
    }
}

#[async_trait]
impl Collector for GpuCollector {
    type Output = Vec<GpuStats>;

    async fn collect(&mut self) -> Result<Self::Output> {
        let mut all_stats = Vec::new();

        // Collect from NVIDIA
        if let Some(backend) = &self.nvidia_backend {
            match backend.collect_stats().await {
                Ok(mut stats) => all_stats.append(&mut stats),
                Err(e) => eprintln!("NVIDIA backend error: {}", e),
            }
        }

        // Collect from AMD
        if let Some(backend) = &self.amd_backend {
            match backend.collect_stats().await {
                Ok(mut stats) => all_stats.append(&mut stats),
                Err(e) => eprintln!("AMD backend error: {}", e),
            }
        }

        // Collect from Intel
        if let Some(backend) = &self.intel_backend {
            match backend.collect_stats().await {
                Ok(mut stats) => all_stats.append(&mut stats),
                Err(e) => eprintln!("Intel backend error: {}", e),
            }
        }

        Ok(all_stats)
    }
}

// Utility functions for sysfs GPU detection
pub(crate) fn read_sysfs_string(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .map_err(|e| Error::IoError(e))
}

pub(crate) fn read_sysfs_u64(path: &Path) -> Result<u64> {
    let content = read_sysfs_string(path)?;
    content.parse::<u64>()
        .map_err(|_| Error::ParseError(format!("Failed to parse u64 from {}", path.display())))
}

pub(crate) fn read_sysfs_f64(path: &Path) -> Result<f64> {
    let content = read_sysfs_string(path)?;
    content.parse::<f64>()
        .map_err(|_| Error::ParseError(format!("Failed to parse f64 from {}", path.display())))
}
