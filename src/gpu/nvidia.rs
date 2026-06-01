use super::{GpuStats, read_sysfs_string, read_sysfs_u64};
use crate::core::{Result, Error};
use std::path::{Path, PathBuf};
use std::fs;

pub struct NvidiaBackend {
    devices: Vec<PathBuf>,
}

impl NvidiaBackend {
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
        let mut stats = Vec::new();

        for device in &self.devices {
            match self.read_device_stats(device) {
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

    fn read_device_stats(&self, device_path: &Path) -> Result<GpuStats> {
        // Read device name
        let name_path = device_path.join("device/device");
        let name = read_sysfs_string(&name_path)
            .unwrap_or_else(|_| "NVIDIA GPU".to_string());

        // Try to read GPU utilization (sysfs may not expose this directly)
        // TODO: For now we'll use a placeholder. Future: integrate NVML library
        let utilization_percent = 0.0;

        // Try to read memory info from hwmon if available
        let mut memory_used_mb = 0;
        let mut memory_total_mb = 0;

        // TODO: NVIDIA doesn't expose VRAM via sysfs easily
        // Future: Use NVML (libnvidia-ml) for accurate stats

        // Try to read temperature from hwmon
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

        // Try to read power usage
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
            vendor: "NVIDIA".to_string(),
            utilization_percent,
            memory_used_mb,
            memory_total_mb,
            temperature_c,
            power_watts,
        })
    }
}
