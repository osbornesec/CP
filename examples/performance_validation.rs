//! Performance validation for the cpinfo parser
//!
//! This example validates the high-performance parser implementation
//! using real cpinfo files and measures throughput.

use cpinfo_parser::CpinfoParser;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn core::error::Error>> {
    println!("\u{1f680} CPInfo Parser Performance Validation");
    println!("========================================");

    let parser = CpinfoParser::new();

    // Test files with their expected approximate sizes
    let test_files = vec![
        (
            "samples/6803352_SG1-s02-02_1_7_2025_19_10.info",
            "Small (8MB)",
        ),
        ("samples/fw-02_vs0.tgz.info", "Medium (219MB)"),
        ("samples/FW1cpinfo.info", "Large (535MB)"),
    ];

    for (file_path, description) in test_files {
        if !std::path::Path::new(file_path).exists() {
            println!("\u{26a0}\u{fe0f}  Skipping {description} - file not found: {file_path}");
            continue;
        }

        let file_size = std::fs::metadata(file_path)?.len();
        println!(
            "\n\u{1f4c1} Testing {} - {:.1} MB",
            description,
            file_size as f64 / 1_000_000.0
        );

        // Test single-threaded performance
        println!("   Single-threaded parsing...");
        let start = Instant::now();
        let result = parser.parse_file(file_path).await?;
        let duration = start.elapsed();
        let throughput_mbps = (file_size as f64 / duration.as_secs_f64()) / 1_000_000.0;

        println!(
            "   \u{2705} Parsed {} sections in {:.2}s",
            result.section_count,
            duration.as_secs_f64()
        );
        println!("   \u{1f4ca} Throughput: {throughput_mbps:.1} MB/s");

        // Check memory usage
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            if let Some(line) = status.lines().find(|line| line.starts_with("VmRSS:")) {
                if let Some(memory_kb) = line.split_whitespace().nth(1) {
                    if let Ok(memory_kb) = memory_kb.parse::<u64>() {
                        let memory_mb = memory_kb / 1024;
                        println!("   \u{1f4be} Memory usage: {memory_mb} MB");
                        if memory_mb > 100 {
                            println!("   \u{26a0}\u{fe0f}  Memory exceeds 100MB target!");
                        }
                    }
                }
            }
        }

        // Check if we meet the 400 MB/s target
        if throughput_mbps > 400.0 {
            println!("   \u{1f3af} PASS: Exceeds 400 MB/s target!");
        } else {
            println!("   \u{26a0}\u{fe0f}  Below 400 MB/s target (got {throughput_mbps:.1} MB/s)");
        }

        // Test concurrent performance
        println!("   Concurrent parsing...");
        let start = Instant::now();
        let result = parser.parse_file_concurrent(file_path).await?;
        let duration = start.elapsed();
        let concurrent_throughput_mbps = (file_size as f64 / duration.as_secs_f64()) / 1_000_000.0;

        println!(
            "   \u{2705} Parsed {} sections in {:.2}s",
            result.section_count,
            duration.as_secs_f64()
        );
        println!("   \u{1f4ca} Concurrent Throughput: {concurrent_throughput_mbps:.1} MB/s");

        // Check if we meet the 1000 MB/s target for larger files
        if file_size > 100_000_000 {
            // > 100MB
            if concurrent_throughput_mbps > 1000.0 {
                println!("   \u{1f3af} PASS: Exceeds 1 GB/s target!");
            } else {
                println!(
                    "   \u{26a0}\u{fe0f}  Below 1 GB/s target (got {concurrent_throughput_mbps:.1} MB/s)"
                );
            }
        }

        // Calculate improvement
        let improvement = (concurrent_throughput_mbps / throughput_mbps - 1.0) * 100.0;
        if improvement > 0.0 {
            println!("   \u{1f4c8} Concurrent improvement: +{improvement:.1}%");
        } else {
            println!("   \u{1f4c9} Concurrent slower by: {:.1}%", -improvement);
        }
    }

    println!("\n\u{1f3c1} Performance validation complete!");
    Ok(())
}
