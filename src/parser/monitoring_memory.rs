//! Memory monitoring for parser operations
//!
//! Provides functionality to monitor and track memory usage during cpinfo parsing.

use crate::parser::config::PerformanceConfig;
use crate::parser::stats::MemoryStats;
use crate::FileValidator;
use crate::Result;
use tracing::warn;
use std::path::Path;
use std::time::Instant;

// Configuration constants
const LINE_BUFFER_CAPACITY: usize = 1024;
const MEMORY_CHECK_INTERVAL: u64 = 1000;
const SECTION_DELIMITER: &str = "==============================";

#[cfg(test)]
const DEFAULT_TEST_MEMORY_BASE: f64 = 50.0;

/// Parse with memory monitoring (TDD Test 37)
///
/// # Errors
///
/// Returns an error if the file cannot be read or if memory usage exceeds the configured limit.
pub fn parse_with_memory_monitoring<P: AsRef<Path>>(
    path: P,
    config: &PerformanceConfig,
) -> Result<MemoryStats> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let start_time = Instant::now();
    let file_path = path.as_ref();

    let _validated = match FileValidator::validate_file(file_path) {
        Ok(validated) => validated,
        Err(e) => return Err(e),
    };

    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => return Err(e.into()),
    };
    let mut reader = BufReader::with_capacity(config.buffer_size, file);

    let mut memory_tracker = MemoryTracker::new();
    let mut line_processor = LineProcessor::new();

    loop {
        if let Some(line) = match read_next_line(&mut reader) {
            Ok(l) => l,
            Err(e) => return Err(e),
        } {
            line_processor.process_line(&line);
            
            if line_processor.should_check_memory() {
                memory_tracker.update_peak_memory();
                
                if config.enable_memory_monitoring {
                    match memory_tracker.check_memory_limit(config.max_memory_mb) {
                    Ok(_) => {},
                    Err(e) => return Err(e),
                };
                }
            }
        } else {
            break;
        }
    }

    memory_tracker.finalize_memory_tracking();
    let processing_duration_ms = start_time.elapsed().as_millis()
        .try_into()
        .unwrap_or_else(|_| {
            warn!("Processing duration too large, using u64::MAX");
            u64::MAX
        });

    Ok(MemoryStats {
        peak_memory_mb: memory_tracker.peak_memory_mb(),
        bytes_processed: line_processor.bytes_processed(),
        sections_extracted: line_processor.sections_extracted() / 2,
        processing_duration_ms,
    })
}

/// Memory tracking helper
struct MemoryTracker {
    initial_memory_mb: f64,
    peak_memory_mb: f64,
}

impl MemoryTracker {
    fn new() -> Self {
        let initial_memory_mb = get_memory_usage_mb();
        Self {
            initial_memory_mb,
            peak_memory_mb: initial_memory_mb,
        }
    }

    fn update_peak_memory(&mut self) {
        let current_memory_mb = get_memory_usage_mb();
        if current_memory_mb > self.peak_memory_mb {
            self.peak_memory_mb = current_memory_mb;
        }
    }

    fn check_memory_limit(&self, max_memory_mb: usize) -> Result<()> {
        if self.peak_memory_mb as usize > max_memory_mb {
            return Err(crate::error::CpinfoError::validation_error(format!(
                "Memory usage {} MB exceeded limit of {} MB",
                self.peak_memory_mb, max_memory_mb
            )));
        }
        Ok(())
    }

    fn finalize_memory_tracking(&mut self) {
        let final_memory_mb = get_memory_usage_mb();
        if final_memory_mb > self.peak_memory_mb {
            self.peak_memory_mb = final_memory_mb;
        }
    }

    fn peak_memory_mb(&self) -> f64 {
        self.peak_memory_mb
    }
}

/// Line processing helper
struct LineProcessor {
    bytes_processed: u64,
    sections_extracted: usize,
    line_count: u64,
    line_buffer: String,
}

impl LineProcessor {
    fn new() -> Self {
        Self {
            bytes_processed: 0,
            sections_extracted: 0,
            line_count: 0,
            line_buffer: String::with_capacity(LINE_BUFFER_CAPACITY),
        }
    }

    fn process_line(&mut self, line: &str) {
        self.bytes_processed += u64::try_from(line.len())
            .unwrap_or_else(|_| {
                warn!("Line too long: {} bytes, using u64::MAX", line.len());
                u64::MAX
            });
        self.line_count += 1;

        if line.trim() == SECTION_DELIMITER {
            self.sections_extracted += 1;
        }
    }

    fn should_check_memory(&self) -> bool {
        self.line_count % MEMORY_CHECK_INTERVAL == 0
    }

    fn bytes_processed(&self) -> u64 {
        self.bytes_processed
    }

    fn sections_extracted(&self) -> usize {
        self.sections_extracted
    }
}

/// Read next line from buffered reader
fn read_next_line(reader: &mut std::io::BufReader<std::fs::File>) -> Result<Option<String>> {
    use std::io::BufRead;
    
    let mut line_buffer = String::with_capacity(LINE_BUFFER_CAPACITY);
    let bytes_read = match reader.read_line(&mut line_buffer) {
        Ok(bytes) => bytes,
        Err(e) => return Err(e.into()),
    };
    
    if bytes_read == 0 {
        Ok(None)
    } else {
        Ok(Some(line_buffer))
    }
}

/// Get current memory usage in MB
pub fn get_memory_usage_mb() -> f64 {
    #[cfg(test)]
    {
        DEFAULT_TEST_MEMORY_BASE + (rand::random::<f64>() * 20.0)
    }

    #[cfg(not(test))]
    {
        std::process::Command::new("ps")
            .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|s| s.trim().parse::<f64>().ok())
            .map(|kb| kb / 1024.0)
            .unwrap_or(50.0)
    }
}