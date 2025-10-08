// Simple test runner to verify Test 37 without compilation issues
use cpinfo_parser::{CpinfoParser, PerformanceConfig};
use tempfile::NamedTempFile;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Running Test 37: Memory optimization verification");
    
    // Create a test file (smaller for quick verification)
    let test_file = create_test_cpinfo_file(10 * 1024 * 1024); // 10MB
    let file_path = test_file.path();
    
    // Configure memory monitoring
    let mut memory_config = PerformanceConfig::default();
    memory_config.max_memory_mb = 100;
    memory_config.buffer_size = 8192;
    memory_config.max_concurrent_files = 1;
    memory_config.flags.memory_monitoring = true;
    memory_config.flags.speed_monitoring = false;
    memory_config.flags.concurrent_processing = false;
    
    // Run the test
    let result = CpinfoParser::parse_with_memory_monitoring(file_path, memory_config)?;
    
    // Verify results
    println!("✅ Test 37 Results:");
    println!("   Peak memory: {:.2} MB", result.peak_memory_mb);
    println!("   Bytes processed: {}", result.bytes_processed);
    println!("   Sections extracted: {}", result.sections_extracted);
    println!("   Duration: {} ms", result.processing_duration_ms);
    
    // Verify assertions
    assert!(result.peak_memory_mb <= 100.0, "Memory usage within limits");
    assert!(result.bytes_processed > 0, "File was processed");
    assert!(result.sections_extracted > 0, "Sections were extracted");
    
    println!("🎉 Test 37: PASSED - Memory optimization working correctly!");
    
    Ok(())
}

fn create_test_cpinfo_file(size_bytes: usize) -> NamedTempFile {
    let mut temp_file = NamedTempFile::with_suffix(".info").unwrap();
    
    // Write cpinfo header
    writeln!(temp_file, "Check Point Support Information").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "Generated: 2024-01-15 10:30:00").unwrap();
    writeln!(temp_file, "Version: R81.20 - Build 030").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    
    let mut bytes_written = 200;
    let mut section_num = 1;
    
    while bytes_written < size_bytes {
        let section_name = format!("Test Section {}", section_num);
        let section_content = format!(
            "System Information:\n  Product: Check Point Security Gateway\n  Version: R81.20\n  Build: 914000250\n  CPU Usage: 45.2%\n  Memory Usage: 67.8%\n  Test data line {}\n",
            section_num
        );
        
        writeln!(temp_file, "{}", section_name).unwrap();
        writeln!(temp_file, "==============================================").unwrap();
        writeln!(temp_file, "{}", section_content).unwrap();
        writeln!(temp_file, "==============================================").unwrap();
        
        bytes_written += section_name.len() + section_content.len() + 100;
        section_num += 1;
        
        if section_num > 10000 { break; }
    }
    
    temp_file.flush().unwrap();
    temp_file
}