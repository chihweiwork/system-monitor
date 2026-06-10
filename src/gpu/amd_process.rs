use std::fs;
use std::path::Path;
use crate::gpu::{GpuProcess, GpuProcessType};

/// Collect AMD GPU processes by scanning /proc/*/fdinfo/*
pub fn collect_amd_processes(card_path: &Path) -> Vec<GpuProcess> {
    let mut processes = Vec::new();

    // Get card number from path (e.g., /sys/class/drm/card0 -> 0)
    let card_name = card_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    // Scan all processes
    if let Ok(proc_entries) = fs::read_dir("/proc") {
        for entry in proc_entries.flatten() {
            if let Some(pid_str) = entry.file_name().to_str() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    if let Some(proc) = scan_process_for_amd_gpu(pid, card_name) {
                        processes.push(proc);
                    }
                }
            }
        }
    }

    processes
}

fn scan_process_for_amd_gpu(pid: u32, card_name: &str) -> Option<GpuProcess> {
    let fd_dir = format!("/proc/{}/fd", pid);
    let fdinfo_dir = format!("/proc/{}/fdinfo", pid);

    // Scan all file descriptors
    let fd_entries = fs::read_dir(&fd_dir).ok()?;

    for fd_entry in fd_entries.flatten() {
        if let Some(fd_num) = fd_entry.file_name().to_str() {
            // Check if this FD points to our DRM device
            if is_drm_fd(&fd_entry.path(), card_name) {
                // Read fdinfo
                let fdinfo_path = format!("{}/{}", fdinfo_dir, fd_num);
                if let Ok(fdinfo) = fs::read_to_string(&fdinfo_path) {
                    if let Some(proc) = parse_amdgpu_fdinfo(pid, &fdinfo) {
                        return Some(proc);
                    }
                }
            }
        }
    }

    None
}

fn is_drm_fd(fd_path: &Path, card_name: &str) -> bool {
    if let Ok(link) = fs::read_link(fd_path) {
        let link_str = link.to_string_lossy();
        // Check if it points to /dev/dri/cardX or /dev/dri/renderDX
        link_str.contains("/dev/dri/") &&
            (link_str.contains(card_name) || link_str.contains(&format!("render{}", &card_name[4..])))
    } else {
        false
    }
}

fn parse_amdgpu_fdinfo(pid: u32, fdinfo: &str) -> Option<GpuProcess> {
    let mut gpu_memory_mb = 0u64;
    let mut gfx_time = 0u64;
    let mut compute_time = 0u64;

    for line in fdinfo.lines() {
        if line.starts_with("drm-memory-vram:") {
            // Format: drm-memory-vram: 123456 kB
            if let Some(value) = line.split_whitespace().nth(1) {
                if let Ok(kb) = value.parse::<u64>() {
                    gpu_memory_mb = kb / 1024;
                }
            }
        } else if line.starts_with("drm-engine-gfx:") {
            // Format: drm-engine-gfx: 123456 ns
            if let Some(value) = line.split_whitespace().nth(1) {
                gfx_time = value.parse::<u64>().unwrap_or(0);
            }
        } else if line.starts_with("drm-engine-compute:") {
            // Format: drm-engine-compute: 123456 ns
            if let Some(value) = line.split_whitespace().nth(1) {
                compute_time = value.parse::<u64>().unwrap_or(0);
            }
        }
    }

    // Only return process if it has any GPU activity
    if gpu_memory_mb > 0 || gfx_time > 0 || compute_time > 0 {
        let process_type = if gfx_time > 0 && compute_time > 0 {
            GpuProcessType::Both
        } else if gfx_time > 0 {
            GpuProcessType::Graphics
        } else {
            GpuProcessType::Compute
        };

        Some(GpuProcess {
            pid,
            process_name: get_process_name(pid),
            gpu_memory_mb,
            gpu_utilization: 0, // Cannot determine from fdinfo
            process_type,
        })
    } else {
        None
    }
}

fn get_process_name(pid: u32) -> String {
    fs::read_to_string(format!("/proc/{}/comm", pid))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| format!("PID {}", pid))
}
