use crate::core::{Result, Error};
use super::Collector;
use async_trait::async_trait;
use std::fs;
use std::collections::HashMap;
use std::ffi::CStr;

#[derive(Debug, Clone)]
pub struct ProcessStats {
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub cpu_percent: f64,
    pub memory_kb: u64,
    pub memory_percent: f64,
    pub status: String,
    pub cmdline: String,
    pub ppid: u32,
    pub threads: u32,
    pub cwd: String,
    pub fd_count: usize,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
    pub io_read_rate: f64,
    pub io_write_rate: f64,
}

pub struct ProcessCollector {
    prev_times: HashMap<u32, (u64, u64)>, // pid -> (utime, stime)
    prev_io: HashMap<u32, (u64, u64, std::time::Instant)>, // pid -> (read_bytes, write_bytes, timestamp)
    prev_total_time: u64,
    total_memory_kb: u64,
}

impl ProcessCollector {
    pub fn new() -> Self {
        let total_memory_kb = Self::get_total_memory().unwrap_or(0);
        Self {
            prev_times: HashMap::new(),
            prev_io: HashMap::new(),
            prev_total_time: 0,
            total_memory_kb,
        }
    }

    fn get_total_memory() -> Result<u64> {
        let content = fs::read_to_string("/proc/meminfo")
            .map_err(|e| Error::Io(e))?;

        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return parts[1].parse::<u64>()
                        .map_err(|_| Error::ParseError("Invalid MemTotal".to_string()));
                }
            }
        }

        Err(Error::ParseError("MemTotal not found".to_string()))
    }

    fn get_total_cpu_time() -> Result<u64> {
        let content = fs::read_to_string("/proc/stat")
            .map_err(|e| Error::Io(e))?;

        let first_line = content.lines().next()
            .ok_or_else(|| Error::ParseError("Empty /proc/stat".to_string()))?;

        if !first_line.starts_with("cpu ") {
            return Err(Error::ParseError("Invalid /proc/stat format".to_string()));
        }

        let parts: Vec<&str> = first_line.split_whitespace().collect();
        let mut total = 0u64;

        for i in 1..parts.len() {
            if let Ok(val) = parts[i].parse::<u64>() {
                total += val;
            }
        }

        Ok(total)
    }

    fn read_process_stat(&self, pid: u32) -> Result<(String, u32, String, u64, u64, u32)> {
        let path = format!("/proc/{}/stat", pid);
        let content = fs::read_to_string(&path)
            .map_err(|e| Error::Io(e))?;

        // Parse /proc/[pid]/stat format
        // Fields: pid (comm) state ppid ... utime stime ... num_threads
        let start = content.find('(').ok_or_else(|| Error::ParseError("No opening paren".to_string()))?;
        let end = content.rfind(')').ok_or_else(|| Error::ParseError("No closing paren".to_string()))?;

        let name = content[start + 1..end].to_string();
        let rest = &content[end + 2..];
        let parts: Vec<&str> = rest.split_whitespace().collect();

        if parts.len() < 18 {
            return Err(Error::ParseError("Insufficient fields in stat".to_string()));
        }

        let state = parts[0].to_string();
        let ppid = parts[1].parse::<u32>()
            .map_err(|_| Error::ParseError("Invalid ppid".to_string()))?;
        let utime = parts[11].parse::<u64>()
            .map_err(|_| Error::ParseError("Invalid utime".to_string()))?;
        let stime = parts[12].parse::<u64>()
            .map_err(|_| Error::ParseError("Invalid stime".to_string()))?;
        let threads = parts[17].parse::<u32>()
            .map_err(|_| Error::ParseError("Invalid num_threads".to_string()))?;

        Ok((name, ppid, state, utime, stime, threads))
    }

    fn read_process_status(&self, pid: u32) -> Result<u64> {
        let path = format!("/proc/{}/status", pid);
        let content = fs::read_to_string(&path)
            .map_err(|e| Error::Io(e))?;

        for line in content.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return parts[1].parse::<u64>()
                        .map_err(|_| Error::ParseError("Invalid VmRSS".to_string()));
                }
            }
        }

        Ok(0)
    }

    fn read_cmdline(&self, pid: u32) -> String {
        let path = format!("/proc/{}/cmdline", pid);
        fs::read_to_string(&path)
            .ok()
            .map(|s| s.replace('\0', " ").trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("[{}]", pid))
    }

    fn read_uid(&self, pid: u32) -> u32 {
        let path = format!("/proc/{}/status", pid);
        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                if line.starts_with("Uid:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        return parts[1].parse::<u32>().unwrap_or(65534); // 65534 = nobody
                    }
                }
            }
        }
        65534 // Default to nobody
    }

    fn uid_to_username(&self, uid: u32) -> String {
        // Method 1: Use libc::getpwuid() for NSS/LDAP/AD support
        unsafe {
            let passwd_ptr = libc::getpwuid(uid);
            if !passwd_ptr.is_null() {
                let passwd = &*passwd_ptr;
                if !passwd.pw_name.is_null() {
                    if let Ok(cstr) = CStr::from_ptr(passwd.pw_name).to_str() {
                        return cstr.to_string();
                    }
                }
            }
        }

        // Method 2: Fallback to /etc/passwd (for systems without NSS or cached lookup)
        if let Ok(content) = fs::read_to_string("/etc/passwd") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 {
                    if let Ok(file_uid) = parts[2].parse::<u32>() {
                        if file_uid == uid {
                            return parts[0].to_string();
                        }
                    }
                }
            }
        }

        // Method 3: Final fallback to UID as string
        uid.to_string()
    }

    fn read_process_io(&self, pid: u32) -> (u64, u64) {
        let path = format!("/proc/{}/io", pid);
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return (0, 0), // Permission denied or process gone
        };

        let mut read_bytes = 0u64;
        let mut write_bytes = 0u64;

        for line in content.lines() {
            if line.starts_with("read_bytes:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    read_bytes = value.parse().unwrap_or(0);
                }
            } else if line.starts_with("write_bytes:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    write_bytes = value.parse().unwrap_or(0);
                }
            }
        }

        (read_bytes, write_bytes)
    }

    fn read_cwd(&self, pid: u32) -> String {
        let path = format!("/proc/{}/cwd", pid);
        fs::read_link(&path)
            .ok()
            .and_then(|p| p.to_str().map(String::from))
            .unwrap_or_else(|| "N/A".to_string())
    }

    fn read_fd_count(&self, pid: u32) -> usize {
        let path = format!("/proc/{}/fd", pid);
        fs::read_dir(&path)
            .ok()
            .map(|entries| entries.count())
            .unwrap_or(0)
    }

    fn collect_process(&mut self, pid: u32, total_time_delta: u64) -> Option<ProcessStats> {
        // Read process info
        let (name, ppid, status, utime, stime, threads) = self.read_process_stat(pid).ok()?;
        let memory_kb = self.read_process_status(pid).unwrap_or(0);
        let cmdline = self.read_cmdline(pid);
        let cwd = self.read_cwd(pid);
        let fd_count = self.read_fd_count(pid);
        let uid = self.read_uid(pid);
        let user = self.uid_to_username(uid);

        // Calculate CPU percentage
        let current_time = utime + stime;
        let cpu_percent = if let Some(&(prev_utime, prev_stime)) = self.prev_times.get(&pid) {
            let prev_total = prev_utime + prev_stime;
            if total_time_delta > 0 && current_time > prev_total {
                let process_time_delta = current_time - prev_total;
                (process_time_delta as f64 / total_time_delta as f64) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Update previous times
        self.prev_times.insert(pid, (utime, stime));

        // Calculate memory percentage
        let memory_percent = if self.total_memory_kb > 0 {
            (memory_kb as f64 / self.total_memory_kb as f64) * 100.0
        } else {
            0.0
        };

        // Read I/O stats
        let (io_read_bytes, io_write_bytes) = self.read_process_io(pid);
        let now = std::time::Instant::now();

        // Calculate I/O rates
        let (io_read_rate, io_write_rate) = if let Some(&(prev_read, prev_write, prev_time)) = self.prev_io.get(&pid) {
            let elapsed = now.duration_since(prev_time).as_secs_f64();
            if elapsed > 0.0 {
                let read_rate = (io_read_bytes.saturating_sub(prev_read) as f64) / elapsed;
                let write_rate = (io_write_bytes.saturating_sub(prev_write) as f64) / elapsed;
                (read_rate, write_rate)
            } else {
                (0.0, 0.0)
            }
        } else {
            (0.0, 0.0)
        };

        // Update previous I/O values
        self.prev_io.insert(pid, (io_read_bytes, io_write_bytes, now));

        Some(ProcessStats {
            pid,
            name,
            user,
            cpu_percent,
            memory_kb,
            memory_percent,
            status,
            cmdline,
            ppid,
            threads,
            cwd,
            fd_count,
            io_read_bytes,
            io_write_bytes,
            io_read_rate,
            io_write_rate,
        })
    }
}

#[async_trait]
impl Collector for ProcessCollector {
    type Output = Vec<ProcessStats>;

    async fn collect(&mut self) -> Result<Self::Output> {
        let mut processes = Vec::new();

        // Get current total CPU time
        let current_total_time = Self::get_total_cpu_time()?;
        let total_time_delta = if self.prev_total_time > 0 {
            current_total_time.saturating_sub(self.prev_total_time)
        } else {
            1 // Avoid division by zero on first run
        };
        self.prev_total_time = current_total_time;

        // Read /proc to get list of PIDs
        let proc_dir = fs::read_dir("/proc")
            .map_err(|e| Error::Io(e))?;

        for entry in proc_dir {
            let entry = entry.map_err(|e| Error::Io(e))?;
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();

            // Check if directory name is a number (PID)
            if let Ok(pid) = name.parse::<u32>() {
                if let Some(stats) = self.collect_process(pid, total_time_delta) {
                    processes.push(stats);
                }
            }
        }

        Ok(processes)
    }
}
