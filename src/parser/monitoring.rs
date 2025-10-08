//! Parser monitoring functionality
//!
//! This module provides memory and performance monitoring capabilities
//! for cpinfo file parsing operations. It uses the Command pattern to
//! separate monitoring concerns into focused, testable components.

pub mod monitoring_commands;
pub mod monitoring_config;
pub mod monitoring_memory;
pub mod monitoring_metrics;

use crate::parser::config::PerformanceConfig;
use crate::parser::stats::{CacheStats, MemoryStats, ResourceStats, SpeedStats};
use crate::Result;
use std::path::Path;

use monitoring_commands::{
    CacheMonitorCommand, MemoryMonitorCommand, MonitorCommand as _, ResourceMonitorCommand,
    SpeedMonitorCommand,
};

/// Parse with memory monitoring (TDD Test 37)
///
/// # Errors
///
/// Returns an error if the file cannot be read or if memory usage exceeds the configured limit.
#[inline]
pub fn parse_with_memory_monitoring<P: AsRef<Path>>(
    path: P,
    config: &PerformanceConfig,
) -> Result<MemoryStats> {
    let command = MemoryMonitorCommand::new(path.as_ref(), config);
    return command.execute();
}

/// Parses a file with speed monitoring.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
#[inline]
pub fn parse_with_speed_monitoring<P: AsRef<Path>>(
    path: P,
    config: &PerformanceConfig,
) -> Result<SpeedStats> {
    let command = SpeedMonitorCommand::new(path.as_ref(), config);
    return command.execute();
}

/// Parses a file with resource monitoring.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
#[inline]
pub fn parse_with_resource_monitoring<P: AsRef<Path>>(
    path: P,
    config: &PerformanceConfig,
) -> Result<ResourceStats> {
    let command = ResourceMonitorCommand::new(path.as_ref(), config);
    return command.execute();
}

/// Parses a file with caching enabled.
///
/// # Errors
///
/// Returns an error if the file cannot be read or if mutex is poisoned.
#[inline]
pub fn parse_with_caching<P: AsRef<Path>>(
    path: P,
    config: &PerformanceConfig,
) -> Result<CacheStats> {
    let command = CacheMonitorCommand::new(path.as_ref(), config);
    return command.execute();
}

/// Parse sections with basic monitoring
///
/// # Errors
/// Returns an error if the file cannot be read.
#[inline]
pub fn parse_sections_basic<P: AsRef<Path>>(path: P) -> crate::Result<usize> {
    use std::fs::File;
    use std::io::{BufRead as _, BufReader};

    const SECTION_DELIMITER: &str = "==============================";

    let file_handle = match File::open(path) {
        Ok(file) => file,
        Err(file_error) => return Err(file_error.into()),
    };
    let reader = BufReader::new(file_handle);
    let mut section_count = 0_usize;

    for line_result in reader.lines() {
        let current_line = match line_result {
            Ok(line_content) => line_content,
            Err(io_error) => return Err(io_error.into()),
        };
        if current_line.trim() == SECTION_DELIMITER {
            section_count += 1_usize;
        }
    }

    // Use explicit division with checked operation to avoid integer division warnings
    let section_pairs = section_count.checked_div(2_usize).unwrap_or_default();

    return Ok(section_pairs); // Sections are delimited by pairs
}
