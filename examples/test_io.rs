// Example demonstrating the IoCollector usage
// Run with: cargo run --example test_io

use system_monitor::collectors::{io::IoCollector, Collector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut collector = IoCollector::new();

    println!("Testing Disk I/O Collector\n");
    println!("Collecting baseline data...");
    collector.collect().await?;

    // Wait a bit to accumulate some I/O activity
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("\nCollecting I/O statistics...\n");
    let stats = collector.collect().await?;

    if stats.is_empty() {
        println!("No disk devices found.");
    } else {
        println!("{:<12} {:>15} {:>15} {:>15} {:>15}",
            "Device", "Read (MB/s)", "Write (MB/s)", "Total Reads", "Total Writes");
        println!("{}", "-".repeat(80));

        for stat in &stats {
            let read_mb = stat.read_bytes_per_sec / 1024.0 / 1024.0;
            let write_mb = stat.write_bytes_per_sec / 1024.0 / 1024.0;
            let total_read_mb = stat.bytes_read / 1024 / 1024;
            let total_write_mb = stat.bytes_written / 1024 / 1024;

            println!("{:<12} {:>15.2} {:>15.2} {:>15} {:>15}",
                stat.device, read_mb, write_mb, total_read_mb, total_write_mb);
        }

        println!("\nMonitored {} devices", stats.len());
    }

    Ok(())
}
