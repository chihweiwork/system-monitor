// Utility functions for the system monitor

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Format bytes to human-readable string (KB, MB, GB, TB)
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f = bytes as f64;
    let unit_idx = (bytes_f.log2() / 10.0).floor() as usize;
    let unit_idx = unit_idx.min(UNITS.len() - 1);

    let value = bytes_f / 1024_f64.powi(unit_idx as i32);

    if value >= 100.0 {
        format!("{:.0} {}", value, UNITS[unit_idx])
    } else if value >= 10.0 {
        format!("{:.1} {}", value, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", value, UNITS[unit_idx])
    }
}

/// Format bytes per second to human-readable string
pub fn format_bandwidth(bytes_per_sec: f64) -> String {
    const UNITS: &[&str] = &["B/s", "KB/s", "MB/s", "GB/s"];

    if bytes_per_sec < 1.0 {
        return "0 B/s".to_string();
    }

    let unit_idx = (bytes_per_sec.log2() / 10.0).floor() as usize;
    let unit_idx = unit_idx.min(UNITS.len() - 1);

    let value = bytes_per_sec / 1024_f64.powi(unit_idx as i32);

    if value >= 100.0 {
        format!("{:.0} {}", value, UNITS[unit_idx])
    } else if value >= 10.0 {
        format!("{:.1} {}", value, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", value, UNITS[unit_idx])
    }
}

/// Format duration to human-readable string
pub fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {:02}:{:02}:{:02}", days, hours, minutes, secs)
    } else if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

/// Get current timestamp in milliseconds
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis() as u64
}

/// Calculate percentage safely (avoid division by zero)
pub fn safe_percentage(value: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (value as f64 / total as f64) * 100.0
    }
}

/// Clamp value between min and max
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_format_bandwidth() {
        assert_eq!(format_bandwidth(0.0), "0 B/s");
        assert_eq!(format_bandwidth(512.0), "512 B/s");
        assert_eq!(format_bandwidth(1024.0), "1.00 KB/s");
        assert_eq!(format_bandwidth(1048576.0), "1.00 MB/s");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "00:30");
        assert_eq!(format_duration(90), "01:30");
        assert_eq!(format_duration(3661), "01:01:01");
        assert_eq!(format_duration(90061), "1d 01:01:01");
    }

    #[test]
    fn test_safe_percentage() {
        assert_eq!(safe_percentage(0, 0), 0.0);
        assert_eq!(safe_percentage(50, 100), 50.0);
        assert_eq!(safe_percentage(100, 100), 100.0);
    }
}
