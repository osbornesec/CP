#!/usr/bin/env rust-script

//! Quick performance profiling script for Check Point CPInfo Parser
//! 
//! This script provides rapid performance analysis including:
//! - CPU profiling with timing analysis
//! - Memory usage monitoring
//! - I/O throughput measurement
//! - Parsing rate calculation

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::{Duration, Instant};

const SAMPLE_FILE: &str = "samples/fw-02_vs0.tgz.info";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CPInfo Parser - Quick Performance Profile");
    println!("============================================");
    
    if !Path::new(SAMPLE_FILE).exists() {
        eprintln!("❌ Sample file not found: {}", SAMPLE_FILE);
        std::process::exit(1);
    }
    
    let file_size = std::fs::metadata(SAMPLE_FILE)?.len();
    println!("📁 File: {} ({:.2} MB)", SAMPLE_FILE, file_size as f64 / 1_000_000.0);
    
    // CPU and throughput profiling
    profile_file_reading(SAMPLE_FILE, file_size)?;
    
    // Memory usage estimation
    profile_memory_usage(SAMPLE_FILE)?;
    
    // Parsing performance simulation
    profile_parsing_simulation(SAMPLE_FILE)?;
    
    println!("\n✅ Profiling complete!");
    Ok(())
}

fn profile_file_reading(file_path: &str, file_size: u64) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 I/O Performance Analysis");
    println!("---------------------------");
    
    let start = Instant::now();
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    
    let mut lines_read = 0;
    let mut bytes_read = 0;
    
    for line_result in reader.lines() {
        let line = line_result?;
        lines_read += 1;
        bytes_read += line.len() + 1; // +1 for newline
        
        // Sample every 10,000 lines for performance
        if lines_read % 10_000 == 0 {
            let elapsed = start.elapsed();
            let throughput = bytes_read as f64 / elapsed.as_secs_f64() / 1_000_000.0;
            print!("\r⏱️  Lines: {:>8} | Throughput: {:.1} MB/s", lines_read, throughput);
        }
    }
    
    let total_time = start.elapsed();
    let final_throughput = file_size as f64 / total_time.as_secs_f64() / 1_000_000.0;
    
    println!("\r✅ Reading complete:");
    println!("   📏 Lines processed: {}", lines_read);
    println!("   ⏱️  Total time: {:.2}s", total_time.as_secs_f64());
    println!("   🚀 Throughput: {:.1} MB/s", final_throughput);
    
    Ok(())
}

fn profile_memory_usage(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧠 Memory Usage Analysis");
    println!("------------------------");
    
    // Simulate different buffer sizes
    let buffer_sizes = [1024, 4096, 8192, 16384, 65536];
    
    for &buffer_size in &buffer_sizes {
        let start = Instant::now();
        let file = File::open(file_path)?;
        let mut reader = BufReader::with_capacity(buffer_size, file);
        
        let mut line = String::new();
        let mut lines_count = 0;
        
        while reader.read_line(&mut line)? > 0 {
            lines_count += 1;
            line.clear();
            
            // Sample performance
            if lines_count % 50_000 == 0 {
                break;
            }
        }
        
        let elapsed = start.elapsed();
        println!("   📦 Buffer size: {:>6} bytes | Time for 50k lines: {:>6.2}ms", 
                buffer_size, elapsed.as_millis());
    }
    
    Ok(())
}

fn profile_parsing_simulation(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Parsing Performance Simulation");
    println!("----------------------------------");
    
    let start = Instant::now();
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    
    let mut section_count = 0;
    let mut command_count = 0;
    let mut file_content_count = 0;
    
    for line_result in reader.lines() {
        let line = line_result?;
        
        // Simulate delimiter detection (basic pattern matching)
        let dash_count = line.chars().take_while(|&c| c == '-').count();
        
        match dash_count {
            23 | 24 => {
                command_count += 1;
                section_count += 1;
            }
            66 => {
                file_content_count += 1;
                section_count += 1;
            }
            _ => {}
        }
    }
    
    let total_time = start.elapsed();
    let sections_per_sec = section_count as f64 / total_time.as_secs_f64();
    
    println!("   📊 Sections detected: {}", section_count);
    println!("   🔧 Commands found: {}", command_count);
    println!("   📁 File contents found: {}", file_content_count);
    println!("   ⚡ Sections/second: {:.1}", sections_per_sec);
    println!("   ⏱️  Total parsing time: {:.2}s", total_time.as_secs_f64());
    
    Ok(())
}