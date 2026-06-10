use super::{GpuStats, GpuProcess, GpuProcessType, read_sysfs_string, read_sysfs_u64};
use crate::core::{Result, Error};
use std::path::{Path, PathBuf};
use std::fs;

#[cfg(feature = "nvidia")]
use nvml_wrapper::Nvml;

pub struct NvidiaBackend {
    devices: Vec<PathBuf>,
    #[cfg(feature = "nvidia")]
    nvml: Option<Nvml>,
}

impl NvidiaBackend {
    pub fn new() -> Self {
        #[cfg(feature = "nvidia")]
        let nvml = Nvml::init().ok();

        Self {
            devices: Self::discover_devices(),
            #[cfg(feature = "nvidia")]
            nvml,
        }
    }

    fn discover_devices() -> Vec<PathBuf> {
        let mut devices = Vec::new();
        let drm_path = Path::new("/sys/class/drm");

        if let Ok(entries) = fs::read_dir(drm_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    // Look for card* directories (not renderD*)
                    if name_str.starts_with("card") && !name_str.contains("render") {
                        // Check if it's an NVIDIA device
                        let vendor_path = path.join("device/vendor");
                        if let Ok(vendor) = read_sysfs_string(&vendor_path) {
                            // NVIDIA PCI vendor ID is 0x10de
                            if vendor.to_lowercase().contains("0x10de") || vendor.contains("10de") {
                                devices.push(path);
                            }
                        }
                    }
                }
            }
        }

        devices
    }

    pub async fn detect() -> Result<bool> {
        let devices = Self::discover_devices();
        Ok(!devices.is_empty())
    }

    pub async fn collect_stats(&self) -> Result<Vec<GpuStats>> {
        #[cfg(feature = "nvidia")]
        {
            if let Some(nvml) = &self.nvml {
                return self.collect_stats_nvml(nvml);
            }
        }

        // Fallback to sysfs-based stats
        self.collect_stats_sysfs()
    }

    #[cfg(feature = "nvidia")]
    fn collect_stats_nvml(&self, nvml: &Nvml) -> Result<Vec<GpuStats>> {
        let mut stats = Vec::new();

        let device_count = nvml.device_count().map_err(|e| {
            Error::CollectionError(format!("Failed to get NVIDIA device count: {}", e))
        })?;

        for i in 0..device_count {
            let device = nvml.device_by_index(i).map_err(|e| {
                Error::CollectionError(format!("Failed to get NVIDIA device {}: {}", i, e))
            })?;

            // Get basic info
            let name = device.name().unwrap_or_else(|_| "NVIDIA GPU".to_string());
            let memory_info = device.memory_info().unwrap_or_else(|_| {
                nvml_wrapper::struct_wrappers::device::MemoryInfo {
                    total: 0,
                    free: 0,
                    used: 0,
                }
            });
            let utilization = device.utilization_rates().ok();
            let temperature = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu).ok();
            let power = device.power_usage().ok();

            // Get process list
            let mut processes = Vec::new();

            // Try to get graphics processes
            if let Ok(graphics_procs) = device.running_graphics_processes() {
                for proc_info in graphics_procs {
                    let memory_mb = match proc_info.used_gpu_memory {
                        nvml_wrapper::enums::device::UsedGpuMemory::Used(bytes) => bytes / (1024 * 1024),
                        nvml_wrapper::enums::device::UsedGpuMemory::Unavailable => 0,
                    };
                    processes.push(GpuProcess {
                        pid: proc_info.pid,
                        process_name: get_process_name(proc_info.pid),
                        gpu_memory_mb: memory_mb,
                        gpu_utilization: 0, // NVML doesn't provide per-process utilization
                        process_type: GpuProcessType::Graphics,
                    });
                }
            }

            // Try to get compute processes
            if let Ok(compute_procs) = device.running_compute_processes() {
                for proc_info in compute_procs {
                    let memory_mb = match proc_info.used_gpu_memory {
                        nvml_wrapper::enums::device::UsedGpuMemory::Used(bytes) => bytes / (1024 * 1024),
                        nvml_wrapper::enums::device::UsedGpuMemory::Unavailable => 0,
                    };
                    // Check if process already exists (could be both graphics and compute)
                    if let Some(existing) = processes.iter_mut().find(|p| p.pid == proc_info.pid) {
                        existing.process_type = GpuProcessType::Both;
                        existing.gpu_memory_mb += memory_mb;
                    } else {
                        processes.push(GpuProcess {
                            pid: proc_info.pid,
                            process_name: get_process_name(proc_info.pid),
                            gpu_memory_mb: memory_mb,
                            gpu_utilization: 0,
                            process_type: GpuProcessType::Compute,
                        });
                    }
                }
            }

            stats.push(GpuStats {
                name,
                vendor: "NVIDIA".to_string(),
                utilization_percent: utilization.map(|u| u.gpu as f64).unwrap_or(0.0),
                memory_used_mb: memory_info.used / (1024 * 1024),
                memory_total_mb: memory_info.total / (1024 * 1024),
                temperature_c: temperature.map(|t| t as f64).unwrap_or(0.0),
                power_watts: power.map(|p| p as f64 / 1000.0).unwrap_or(0.0),
                processes,
                gpu_id: i,
            });
        }

        if stats.is_empty() {
            return Err(Error::CollectionError("No NVIDIA GPU stats available".to_string()));
        }

        Ok(stats)
    }

    fn collect_stats_sysfs(&self) -> Result<Vec<GpuStats>> {
        let mut stats = Vec::new();

        for (gpu_id, device) in self.devices.iter().enumerate() {
            match self.read_device_stats(device, gpu_id as u32) {
                Ok(stat) => stats.push(stat),
                Err(e) => {
                    eprintln!("Failed to read NVIDIA GPU stats: {}", e);
                }
            }
        }

        if stats.is_empty() {
            return Err(Error::CollectionError("No NVIDIA GPU stats available".to_string()));
        }

        Ok(stats)
    }

    fn read_device_stats(&self, device_path: &Path, gpu_id: u32) -> Result<GpuStats> {
        // Read device name
        let name_path = device_path.join("device/device");
        let name = read_sysfs_string(&name_path)
            .unwrap_or_else(|_| "NVIDIA GPU".to_string());

        let utilization_percent = 0.0;
        let mut memory_used_mb = 0;
        let mut memory_total_mb = 0;

        // Try to read temperature from hwmon
        let mut temperature_c = 0.0;
        if let Ok(hwmon_entries) = fs::read_dir(device_path.join("device/hwmon")) {
            for hwmon in hwmon_entries.flatten() {
                let temp_input = hwmon.path().join("temp1_input");
                if let Ok(temp) = read_sysfs_u64(&temp_input) {
                    temperature_c = temp as f64 / 1000.0;
                    break;
                }
            }
        }

        // Try to read power usage
        let mut power_watts = 0.0;
        if let Ok(hwmon_entries) = fs::read_dir(device_path.join("device/hwmon")) {
            for hwmon in hwmon_entries.flatten() {
                let power_input = hwmon.path().join("power1_average");
                if let Ok(power) = read_sysfs_u64(&power_input) {
                    power_watts = power as f64 / 1_000_000.0;
                    break;
                }
            }
        }

        Ok(GpuStats {
            name,
            vendor: "NVIDIA".to_string(),
            utilization_percent,
            memory_used_mb,
            memory_total_mb,
            temperature_c,
            power_watts,
            processes: Vec::new(),
            gpu_id,
        })
    }
}

fn get_process_name(pid: u32) -> String {
    std::fs::read_to_string(format!("/proc/{}/comm", pid))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| format!("PID {}", pid))
}
