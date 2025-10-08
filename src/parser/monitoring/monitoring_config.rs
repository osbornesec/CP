//! Configuration constants and types for monitoring operations

/// Line buffer capacity for reading operations
pub const LINE_BUFFER_CAPACITY: usize = 1024;

/// Interval for memory checks during processing
pub const MEMORY_CHECK_INTERVAL: u64 = 1000;

/// Section delimiter used in cpinfo files
pub const SECTION_DELIMITER: &str = "==============================";

/// Buffer size ratio for speed monitoring
pub const SPEED_BUFFER_RATIO: usize = 8;

/// Maximum number of sections to process for speed monitoring
pub const MAX_SPEED_SECTIONS: usize = 10000;

/// Default test memory base for testing environment
#[cfg(test)]
pub const DEFAULT_TEST_MEMORY_BASE: f64 = 50.0;

/// Common patterns found in cpinfo files
pub const COMMON_PATTERNS: &[&str] = &[
    "General Information",
    "Network Configuration",
    "Security Policy",
    "System Status",
    "Performance Metrics",
    "Hardware Information",
];
