use super::{GpuStats, read_sysfs_string, read_sysfs_u64};
use crate::core::{Result, Error};
use std::path::{Path, PathBuf};
use std::fs;
use super::intel_process::collect_intel_processes;

pub struct IntelBackend {
    devices: Vec<PathBuf>,
}

impl IntelBackend {
    pub fn new() -> Self {
        Self {
            devices: Self::discover_devices(),
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
                    // Look for card* directories
                    if name_str.starts_with("card") && !name_str.contains("render") {
                        // Check if it's an Intel device
                        let vendor_path = path.join("device/vendor");
                        if let Ok(vendor) = read_sysfs_string(&vendor_path) {
                            // Intel PCI vendor ID is 0x8086
                            if vendor.to_lowercase().contains("0x8086") || vendor.contains("8086") {
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
        let mut stats = Vec::new();

        for (gpu_id, device) in self.devices.iter().enumerate() {
            match self.read_device_stats(device, gpu_id as u32) {
                Ok(stat) => stats.push(stat),
                Err(e) => {
                    eprintln!("Failed to read Intel GPU stats: {}", e);
                }
            }
        }

        if stats.is_empty() {
            return Err(Error::CollectionError("No Intel GPU stats available".to_string()));
        }

        Ok(stats)
    }

    fn read_device_stats(&self, device_path: &Path, gpu_id: u32) -> Result<GpuStats> {
        // Read device name
        let name_path = device_path.join("device/device");
        let name = read_sysfs_string(&name_path)
            .unwrap_or_else(|_| "Intel GPU".to_string());

        // Intel GPUs may expose frequency info
        let mut utilization_percent = 0.0;

        // Try to read current frequency and max frequency to estimate utilization
        let cur_freq_path = device_path.join("gt_cur_freq_mhz");
        let max_freq_path = device_path.join("gt_max_freq_mhz");

        if let (Ok(cur), Ok(max)) = (read_sysfs_u64(&cur_freq_path), read_sysfs_u64(&max_freq_path)) {
            if max > 0 {
                utilization_percent = (cur as f64 / max as f64) * 100.0;
            }
        }

        // Intel integrated GPUs share system memory, not easy to read VRAM
        // TODO: Future enhancement with i915 kernel driver APIs
        let memory_used_mb = 0;
        let memory_total_mb = 0;

        // Read temperature from hwmon if available
        let mut temperature_c = 0.0;
        if let Ok(hwmon_entries) = fs::read_dir(device_path.join("device/hwmon")) {
            for hwmon in hwmon_entries.flatten() {
                let temp_input = hwmon.path().join("temp1_input");
                if let Ok(temp) = read_sysfs_u64(&temp_input) {
                    temperature_c = temp as f64 / 1000.0; // Convert from millidegrees
                    break;
                }
            }
        }

        // Read power usage
        let mut power_watts = 0.0;
        if let Ok(hwmon_entries) = fs::read_dir(device_path.join("device/hwmon")) {
            for hwmon in hwmon_entries.flatten() {
                let power_input = hwmon.path().join("power1_average");
                if let Ok(power) = read_sysfs_u64(&power_input) {
                    power_watts = power as f64 / 1_000_000.0; // Convert from microwatts
                    break;
                }
            }
        }

        // Collect GPU processes
        let processes = collect_intel_processes(device_path);

        Ok(GpuStats {
            name,
            vendor: "Intel".to_string(),
            utilization_percent,
            memory_used_mb,
            memory_total_mb,
            temperature_c,
            power_watts,
            processes,
            gpu_id,
        })
    }
}
