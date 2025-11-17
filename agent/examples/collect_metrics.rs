// 测试程序：展示实际系统指标采集
use vespera_agent::collector::SystemCollector;

fn main() {
    println!("=== Vespera Agent - System Metrics Test ===\n");

    let mut collector = SystemCollector::new("test-node".to_string());

    println!("Collecting system metrics...\n");

    for i in 1..=3 {
        println!("--- Sample {} ---", i);
        let metrics = collector.collect();

        println!("Node ID: {}", metrics.node_id);
        println!("Timestamp: {}", metrics.timestamp);
        println!();

        println!("CPU:");
        println!("  Usage: {:.2}%", metrics.cpu_usage);
        println!();

        println!("Memory:");
        println!(
            "  Total: {:.2} GB",
            metrics.memory_total as f64 / 1024.0 / 1024.0 / 1024.0
        );
        println!(
            "  Used: {:.2} GB ({:.1}%)",
            metrics.memory_used as f64 / 1024.0 / 1024.0 / 1024.0,
            metrics.memory_usage_percent()
        );
        println!(
            "  Swap Total: {:.2} GB",
            metrics.swap_total as f64 / 1024.0 / 1024.0 / 1024.0
        );
        println!(
            "  Swap Used: {:.2} GB",
            metrics.swap_used as f64 / 1024.0 / 1024.0 / 1024.0
        );
        println!();

        println!("Disk:");
        println!(
            "  Total: {:.2} GB",
            metrics.disk_total as f64 / 1024.0 / 1024.0 / 1024.0
        );
        println!(
            "  Used: {:.2} GB ({:.1}%)",
            metrics.disk_used as f64 / 1024.0 / 1024.0 / 1024.0,
            metrics.disk_usage_percent()
        );
        println!();

        println!("Network (delta since last sample):");
        println!(
            "  Received: {:.2} KB",
            metrics.network_in as f64 / 1024.0
        );
        println!(
            "  Sent: {:.2} KB",
            metrics.network_out as f64 / 1024.0
        );
        println!();

        println!("System Load:");
        println!("  1 min: {:.2}", metrics.load_1);
        println!("  5 min: {:.2}", metrics.load_5);
        println!("  15 min: {:.2}", metrics.load_15);
        println!();

        println!("Uptime: {} seconds ({:.1} hours)", metrics.uptime, metrics.uptime as f64 / 3600.0);
        println!();

        if i < 3 {
            println!("Waiting 2 seconds before next sample...\n");
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    println!("=== Test Complete ===");
}
