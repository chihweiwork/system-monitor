use super::{GpuStats, read_sysfs_string, read_sysfs_u64};
use crate::core::{Result, Error};
use std::path::{Path, PathBuf};
use std::fs;

pub struct AmdBackend {
    devices: Vec<PathBuf>,
}

impl AmdBackend {
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
                        // Check if it's an AMD device
                        let vendor_path = path.join("device/vendor");
                        if let Ok(vendor) = read_sysfs_string(&vendor_path) {
                            // AMD PCI vendor ID is 0x1002
                            if vendor.to_lowercase().contains("0x1002") || vendor.contains("1002") {
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
                    eprintln!("Failed to read AMD GPU stats: {}", e);
                }
            }
        }

        if stats.is_empty() {
            return Err(Error::CollectionError("No AMD GPU stats available".to_string()));
        }

        Ok(stats)
    }

    fn read_device_stats(&self, device_path: &Path, gpu_id: u32) -> Result<GpuStats> {
        // Read device name
        let name_path = device_path.join("device/device");
        let name = read_sysfs_string(&name_path)
            .unwrap_or_else(|_| "AMD GPU".to_string());

        // AMD exposes GPU utilization via sysfs
        let mut utilization_percent = 0.0;
        let gpu_busy_path = device_path.join("device/gpu_busy_percent");
        if let Ok(busy) = read_sysfs_u64(&gpu_busy_path) {
            utilization_percent = busy as f64;
        }

        // Read VRAM usage (AMD exposes this via sysfs)
        let mut memory_used_mb = 0;
        let mut memory_total_mb = 0;

        let mem_used_path = device_path.join("device/mem_info_vram_used");
        if let Ok(used) = read_sysfs_u64(&mem_used_path) {
            memory_used_mb = used / (1024 * 1024); // Convert from bytes to MB
        }

        let mem_total_path = device_path.join("device/mem_info_vram_total");
        if let Ok(total) = read_sysfs_u64(&mem_total_path) {
            memory_total_mb = total / (1024 * 1024); // Convert from bytes to MB
        }

        // Read temperature from hwmon
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

        Ok(GpuStats {
            name,
            vendor: "AMD".to_string(),
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
