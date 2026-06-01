use crate::core::Result;
use super::Collector;
use async_trait::async_trait;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct DiskStats {
    pub mount_point: String,
    pub device: String,
    pub fs_type: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
}

pub struct DiskCollector {
    ignore_list: Vec<String>,
}

impl DiskCollector {
    pub fn new() -> Self {
        Self {
            ignore_list: Vec::new(),
        }
    }

    fn read_mounts(&self) -> Result<Vec<MountInfo>> {
        // Read from /proc/self/mounts (preferred) or /etc/mtab
        let mount_path = if std::path::Path::new("/proc/self/mounts").exists() {
            "/proc/self/mounts"
        } else {
            "/etc/mtab"
        };

        let file = File::open(mount_path)?;
        let reader = BufReader::new(file);
        let mut mounts = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.len() < 3 {
                continue;
            }

            let device = parts[0].to_string();
            let mount_point = parts[1].to_string();
            let fs_type = parts[2].to_string();

            // Skip if in ignore list
            if self.ignore_list.contains(&mount_point) {
                continue;
            }

            // Filter out pseudo filesystems
            if Self::should_skip_filesystem(&fs_type) {
                continue;
            }

            mounts.push(MountInfo {
                device,
                mount_point,
                fs_type,
            });
        }

        Ok(mounts)
    }

    fn should_skip_filesystem(fs_type: &str) -> bool {
        // Skip pseudo filesystems and special mounts
        matches!(
            fs_type,
            "proc" | "sysfs" | "devtmpfs" | "devpts" | "tmpfs" |
            "cgroup" | "cgroup2" | "pstore" | "bpf" | "tracefs" |
            "debugfs" | "securityfs" | "hugetlbfs" | "mqueue" |
            "autofs" | "configfs" | "fusectl" | "selinuxfs" |
            "overlay" | "nsfs" | "binfmt_misc"
        )
    }

    fn get_disk_stats(&mut self, mount_info: &MountInfo) -> Option<DiskStats> {
        // Use statvfs to get filesystem statistics
        let mount_point_cstr = std::ffi::CString::new(mount_info.mount_point.as_bytes()).ok()?;

        unsafe {
            let mut stat: libc::statvfs = std::mem::zeroed();
            if libc::statvfs(mount_point_cstr.as_ptr(), &mut stat) != 0 {
                // If statvfs fails, add to ignore list
                self.ignore_list.push(mount_info.mount_point.clone());
                return None;
            }

            let block_size = stat.f_frsize as u64;
            let total_bytes = stat.f_blocks * block_size;
            let available_bytes = stat.f_bavail * block_size;
            let used_bytes = total_bytes - (stat.f_bfree * block_size);

            let usage_percent = if total_bytes > 0 {
                (used_bytes as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };

            Some(DiskStats {
                mount_point: mount_info.mount_point.clone(),
                device: mount_info.device.clone(),
                fs_type: mount_info.fs_type.clone(),
                total_bytes,
                used_bytes,
                available_bytes,
                usage_percent,
            })
        }
    }
}

#[derive(Debug)]
struct MountInfo {
    device: String,
    mount_point: String,
    fs_type: String,
}

#[async_trait]
impl Collector for DiskCollector {
    type Output = Vec<DiskStats>;

    async fn collect(&mut self) -> Result<Self::Output> {
        let mounts = self.read_mounts()?;
        let mut disk_stats = Vec::new();

        for mount in mounts {
            if let Some(stats) = self.get_disk_stats(&mount) {
                disk_stats.push(stats);
            }
        }

        Ok(disk_stats)
    }
}
