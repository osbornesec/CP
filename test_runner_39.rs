// Test runner for Test 39: Concurrent processing verification
use cpinfo_parser::{CpinfoParser, PerformanceConfig};
use std::io::Write as _;
use tempfile::NamedTempFile;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    println!("\u{1f680} Running Test 39: Concurrent file processing verification");

    // Create 5 test files for concurrent processing (50MB each)
    let test_files: Vec<_> = (0..5)
        .map(|i| {
            return create_concurrent_test_file(50 * 1024 * 1024, &format!("concurrent_file_{i}"));
            // 50MB each
        })
        .collect();

    let file_paths: Vec<_> = test_files
        .iter()
        .map(tempfile::NamedTempFile::path)
        .collect();

    // Configure for concurrent processing
    let mut concurrent_config = PerformanceConfig::default();
    concurrent_config.max_memory_mb = 200; // Allow more memory for concurrent operations
    concurrent_config.max_concurrent_files = 3;
    concurrent_config.enable_memory_monitoring = true;
    concurrent_config.enable_concurrent_processing = true;

    let start_time = std::time::Instant::now();
    let result = CpinfoParser::parse_concurrent(file_paths, concurrent_config)?;
    let elapsed = start_time.elapsed();

    // Calculate performance metrics
    let total_size_mb = 5.0 * 50.0; // 250MB total
    let processing_speed = total_size_mb / elapsed.as_secs_f64();

    // Display results
    println!("\u{2705} Test 39 Results:");
    println!("   Total files: 5");
    println!("   Files processed: {}", result.files_processed);
    println!("   Total size: {total_size_mb:.2} MB");
    println!("   Concurrent processing speed: {processing_speed:.2} MB/s");
    println!(
        "   Average files per second: {:.2}",
        result.average_files_per_second
    );
    println!("   Peak memory usage: {:.2} MB", result.peak_memory_mb);
    println!("   Total sections extracted: {}", result.total_sections);
    println!("   Duration: {} ms", result.processing_duration_ms);
    println!("   Elapsed time: {elapsed:?}");

    // Verify performance targets
    if result.files_processed == 5 {
        println!("\u{1f389} Test 39: PASSED - All files processed successfully");
    } else {
        println!(
            "\u{26a0}\u{fe0f}  Test 39: Files processed mismatch ({} out of 5)",
            result.files_processed
        );
    }

    if processing_speed >= 100.0 {
        println!(
            "\u{1f389} Concurrent processing speed target achieved ({processing_speed:.2} MB/s >= 100 MB/s)"
        );
    } else {
        println!(
            "\u{26a0}\u{fe0f}  Concurrent processing speed below target ({processing_speed:.2} MB/s < 100 MB/s)"
        );
    }

    if result.peak_memory_mb <= 200.0 {
        println!(
            "\u{1f389} Memory usage within concurrent limits ({:.2} MB <= 200 MB)",
            result.peak_memory_mb
        );
    } else {
        println!(
            "\u{26a0}\u{fe0f}  Memory usage exceeded concurrent limits ({:.2} MB > 200 MB)",
            result.peak_memory_mb
        );
    }

    return Ok(());
}

fn create_concurrent_test_file(size_bytes: usize, identifier: &str) -> NamedTempFile {
    let mut temp_file = NamedTempFile::with_suffix(".info").unwrap();

    // Write cpinfo header with identifier
    writeln!(temp_file, "Check Point Support Information - {identifier}").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "Generated: 2024-01-15 10:30:00").unwrap();
    writeln!(temp_file, "Version: R81.20 - Build 030").unwrap();
    writeln!(temp_file, "File ID: {identifier}").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    let mut bytes_written = 250; // Approximate header size
    let mut section_num = 1;

    // Generate sections for concurrent file
    while bytes_written < size_bytes {
        let section_name = format!("{identifier} Section {section_num}");
        let section_content = generate_concurrent_section_content(4096); // 4KB per section

        writeln!(temp_file, "{section_name}").unwrap();
        writeln!(temp_file, "==============================================").unwrap();
        writeln!(temp_file, "{section_content}").unwrap();
        writeln!(temp_file, "==============================================").unwrap();

        bytes_written += section_name.len() + section_content.len() + 100;
        section_num += 1;

        if section_num > 12000 {
            break;
        }
    }

    temp_file.flush().unwrap();
    return temp_file;
}

fn generate_concurrent_section_content(target_size: usize) -> String {
    let mut content = String::new();
    let base_lines = vec![
        "System Information:",
        "  Product: Check Point Security Gateway",
        "  Version: R81.20",
        "  Build: 914000250",
        "  Kernel: R81.20",
        "",
        "Concurrent Processing Performance:",
        "  Worker Thread: Active",
        "  Memory Usage: Optimized",
        "  Cache Status: Enabled",
        "  Buffer Pool: Available",
        "",
        "Network Configuration:",
        "  Interface eth0: 192.168.1.100/24",
        "  Interface eth1: 10.0.0.1/8",
        "  Gateway: 192.168.1.1",
        "",
        "Security Policy:",
        "  Rules: 1500",
        "  Objects: 2500",
        "  Last Policy Install: 2024-01-15 09:15:30",
        "",
        "Performance Metrics:",
        "  CPU Usage: 45.2%",
        "  Memory Usage: 67.8%",
        "  Connections: 15000",
        "  Throughput: 890.5 Mbps",
        "",
    ];

    let mut current_size = 0;
    let mut line_index = 0;

    while current_size < target_size {
        let line = &base_lines[line_index % base_lines.len()];
        content.push_str(line);
        content.push('\n');
        current_size += line.len() + 1;
        line_index += 1;
    }

    return content;
}
