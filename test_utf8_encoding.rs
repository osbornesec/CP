#!/usr/bin/env rust-script

//! Test script to verify UTF-8 encoding fix for streaming parser
//! 
//! This script tests the new streaming implementation's ability to handle:
//! 1. Mixed UTF-8 and binary content
//! 2. Files with invalid UTF-8 sequences
//! 3. Graceful fallback to lossy conversion

use std::fs;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing UTF-8 encoding fix for streaming parser");
    
    // Create test directory
    let test_dir = "test_utf8_encoding";
    fs::create_dir_all(test_dir)?;
    
    // Test 1: Create a file with mixed UTF-8 and binary content
    let test_file_mixed = format!("{}/mixed_content.cpinfo", test_dir);
    create_mixed_content_file(&test_file_mixed)?;
    
    // Test 2: Create a file with invalid UTF-8 sequences
    let test_file_invalid = format!("{}/invalid_utf8.cpinfo", test_dir);
    create_invalid_utf8_file(&test_file_invalid)?;
    
    // Test 3: Create a normal UTF-8 file for comparison
    let test_file_normal = format!("{}/normal_utf8.cpinfo", test_dir);
    create_normal_utf8_file(&test_file_normal)?;
    
    println!("\n📁 Created test files in: {}", test_dir);
    println!("   - mixed_content.cpinfo (UTF-8 + binary data)");
    println!("   - invalid_utf8.cpinfo (invalid UTF-8 sequences)");
    println!("   - normal_utf8.cpinfo (normal UTF-8 content)");
    
    println!("\n🔬 Test these files with the parser to verify UTF-8 handling:");
    println!("   cargo run --bin test_streaming -- {}/mixed_content.cpinfo test_output_mixed", test_dir);
    println!("   cargo run --bin test_streaming -- {}/invalid_utf8.cpinfo test_output_invalid", test_dir);
    println!("   cargo run --bin test_streaming -- {}/normal_utf8.cpinfo test_output_normal", test_dir);
    
    Ok(())
}

fn create_mixed_content_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::create(path)?;
    
    // Write normal UTF-8 content
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "General Info").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "This is normal UTF-8 content.").unwrap();
    writeln!(file, "Host name: test-host").unwrap();
    writeln!(file, "IP address: 192.168.1.100").unwrap();
    writeln!(file, "").unwrap();
    
    // Insert some binary data that would cause UTF-8 issues
    let binary_data = vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD];
    file.write_all(&binary_data)?;
    writeln!(file, "").unwrap();
    writeln!(file, "More normal text after binary data").unwrap();
    
    // Add another section
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Network Config").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Interface: eth0").unwrap();
    
    // More binary data
    let more_binary = vec![0x80, 0x81, 0x82, 0x83, 0x84, 0x85];
    file.write_all(&more_binary)?;
    writeln!(file, "").unwrap();
    writeln!(file, "Gateway: 192.168.1.1").unwrap();
    
    Ok(())
}

fn create_invalid_utf8_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::create(path)?;
    
    // Write valid UTF-8 section header
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "System Status").unwrap();
    writeln!(file, "==============================================").unwrap();
    
    // Write some invalid UTF-8 sequences
    // These are byte sequences that don't form valid UTF-8
    let invalid_utf8_sequences = vec![
        vec![0xC0, 0x80], // Overlong encoding of null
        vec![0xE0, 0x80, 0x80], // Overlong encoding
        vec![0xF0, 0x80, 0x80, 0x80], // Overlong encoding
        vec![0xED, 0xA0, 0x80], // UTF-16 surrogate
        vec![0xFF, 0xFE, 0xFD, 0xFC], // Invalid start bytes
    ];
    
    writeln!(file, "CPU Status: OK").unwrap();
    
    for seq in invalid_utf8_sequences {
        file.write_all(&seq)?;
        writeln!(file, " <-- Invalid UTF-8 sequence").unwrap();
    }
    
    writeln!(file, "Memory Status: 8GB available").unwrap();
    
    // Add another section with more invalid sequences
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Process List").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "PID\tName\tStatus").unwrap();
    
    // Mix valid text with invalid UTF-8
    file.write_all(b"1234\t")?;
    file.write_all(&[0xC2])?; // Incomplete UTF-8 sequence
    file.write_all(b"\tRunning\n")?;
    
    file.write_all(b"5678\ttest_proc")?;
    file.write_all(&[0xF0, 0x90])?; // Incomplete 4-byte UTF-8 sequence
    file.write_all(b"\tStopped\n")?;
    
    Ok(())
}

fn create_normal_utf8_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::create(path)?;
    
    // Write completely normal UTF-8 content
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "General Info").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Host: normal-host").unwrap();
    writeln!(file, "OS: Linux 5.4.0").unwrap();
    writeln!(file, "Uptime: 30 days").unwrap();
    writeln!(file, "Load: 0.15, 0.23, 0.18").unwrap();
    writeln!(file, "").unwrap();
    
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Network Interfaces").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "eth0: 192.168.1.100/24").unwrap();
    writeln!(file, "lo: 127.0.0.1/8").unwrap();
    writeln!(file, "").unwrap();
    
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Security Policy").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Firewall: Enabled").unwrap();
    writeln!(file, "IPS: Active").unwrap();
    writeln!(file, "Anti-Virus: Updated").unwrap();
    
    Ok(())
}