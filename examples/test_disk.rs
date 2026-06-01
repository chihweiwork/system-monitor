// Test program for disk collector
use system_monitor::collectors::{disk::DiskCollector, Collector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut collector = DiskCollector::new();

    println!("Collecting disk statistics...\n");

    let disks = collector.collect().await?;

    if disks.is_empty() {
        println!("No disks found!");
    } else {
        println!("Found {} disk(s):\n", disks.len());

        for disk in &disks {
            println!("Mount Point: {}", disk.mount_point);
            println!("Device:      {}", disk.device);
            println!("Filesystem:  {}", disk.fs_type);
            println!("Total:       {:.2} GB", disk.total_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
            println!("Used:        {:.2} GB", disk.used_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
            println!("Available:   {:.2} GB", disk.available_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
            println!("Usage:       {:.1}%", disk.usage_percent);
            println!();
        }
    }

    Ok(())
}
