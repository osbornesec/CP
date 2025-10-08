// Test runner for Test 38: Speed optimization verification
use cpinfo_parser::{CpinfoParser, PerformanceConfig};
use std::io::Write as _;
use tempfile::NamedTempFile;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    println!("\u{1f680} Running Test 38: Processing speed optimization verification");

    // Create a 100MB test file for speed testing
    let test_file = create_speed_test_file(100 * 1024 * 1024); // 100MB
    let file_path = test_file.path();

    // Configure for speed optimization
    let mut speed_config = PerformanceConfig::default();
    speed_config.buffer_size = 16384; // 16KB buffer for faster processing
    speed_config.enable_speed_monitoring = true;

    let start_time = std::time::Instant::now();
    let result = CpinfoParser::parse_with_speed_monitoring(file_path, &speed_config)?;
    let elapsed = start_time.elapsed();

    // Calculate processing speed
    let file_size_mb = 100.0;
    let processing_speed_mb_per_sec = file_size_mb / elapsed.as_secs_f64();

    // Display results
    println!("\u{2705} Test 38 Results:");
    println!("   File size: {file_size_mb:.2} MB");
    println!("   Processing speed: {processing_speed_mb_per_sec:.2} MB/s");
    println!("   Sections per second: {:.2}", result.sections_per_second);
    println!("   Bytes per second: {:.0}", result.bytes_per_second);
    println!("   Total sections: {}", result.total_sections);
    println!("   Duration: {} ms", result.processing_duration_ms);
    println!("   Elapsed time: {elapsed:?}");

    // Verify performance targets
    if processing_speed_mb_per_sec >= 100.0 {
        println!(
            "\u{1f389} Test 38: PASSED - Speed target achieved ({processing_speed_mb_per_sec:.2} MB/s >= 100 MB/s)"
        );
    } else {
        println!(
            "\u{26a0}\u{fe0f}  Test 38: Speed target not met ({processing_speed_mb_per_sec:.2} MB/s < 100 MB/s)"
        );
    }

    if result.sections_per_second > 10.0 {
        println!(
            "\u{1f389} Section processing rate is excellent ({:.2} sections/s)",
            result.sections_per_second
        );
    } else {
        println!(
            "\u{26a0}\u{fe0f}  Section processing rate could be improved ({:.2} sections/s)",
            result.sections_per_second
        );
    }

    return Ok(());
}

fn create_speed_test_file(size_bytes: usize) -> NamedTempFile {
    let mut temp_file = NamedTempFile::with_suffix(".info").unwrap();

    // Write cpinfo header
    writeln!(temp_file, "Check Point Support Information - Speed Test").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "Generated: 2024-01-15 10:30:00").unwrap();
    writeln!(temp_file, "Version: R81.20 - Build 030").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    let mut bytes_written = 250;
    let mut section_num = 1;

    while bytes_written < size_bytes {
        let section_name = format!("Speed Test Section {section_num}");
        let section_content = generate_section_content(4096); // 4KB per section

        writeln!(temp_file, "{section_name}").unwrap();
        writeln!(temp_file, "==============================================").unwrap();
        writeln!(temp_file, "{section_content}").unwrap();
        writeln!(temp_file, "==============================================").unwrap();

        bytes_written += section_name.len() + section_content.len() + 100;
        section_num += 1;

        if section_num > 25000 {
            break;
        }
    }

    temp_file.flush().unwrap();
    return temp_file;
}

fn generate_section_content(target_size: usize) -> String {
    let mut content = String::new();
    let base_lines = vec![
        "System Information:",
        "  Product: Check Point Security Gateway",
        "  Version: R81.20",
        "  Build: 914000250",
        "  Kernel: R81.20",
        "",
        "Performance Metrics:",
        "  CPU Usage: 45.2%",
        "  Memory Usage: 67.8%",
        "  Connections: 15000",
        "  Throughput: 890.5 Mbps",
        "  Packets/sec: 150000",
        "  Concurrent sessions: 5000",
        "",
        "Network Configuration:",
        "  Interface eth0: 192.168.1.100/24",
        "  Interface eth1: 10.0.0.1/8",
        "  Gateway: 192.168.1.1",
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
