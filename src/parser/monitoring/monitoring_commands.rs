//! Command pattern implementations for monitoring operations

#![allow(
    clippy::std_instead_of_alloc,
    reason = "Project uses std, alloc not available in std context"
)]
#![allow(
    clippy::std_instead_of_core,
    reason = "Project uses std, Instant not available in core"
)]

use crate::parser::config::PerformanceConfig;
use crate::parser::monitoring::monitoring_config::{
    LINE_BUFFER_CAPACITY, MEMORY_CHECK_INTERVAL, SECTION_DELIMITER,
};
use crate::parser::stats::{CacheStats, MemoryStats, ResourceStats, SpeedStats};
use crate::Result;
use std::collections::HashMap;
use std::io::{BufRead as _, BufReader};
use std::path::Path;
use std::time::{Duration, Instant};

/// Trait for monitoring commands
pub trait MonitorCommand<T> {
    /// Execute the monitoring command
    ///
    /// # Errors
    /// Returns an error if the monitoring operation fails.
    fn execute(self) -> crate::Result<T>;
}

/// Cache monitoring command
pub struct CacheMonitorCommand<'path_lifetime> {
    config: &'path_lifetime PerformanceConfig,
    path: &'path_lifetime Path,
}

impl<'path_lifetime> CacheMonitorCommand<'path_lifetime> {
    /// Create new cache monitoring command
    #[must_use]
    #[inline]
    pub const fn new(
        path: &'path_lifetime Path,
        config: &'path_lifetime PerformanceConfig,
    ) -> Self {
        return Self { config, path };
    }
}

/// Local cache bookkeeping to avoid shared mutable state across commands
#[derive(Default)]
struct CacheState {
    entries: HashMap<String, String>,
    hits: usize,
    misses: usize,
}

impl MonitorCommand<CacheStats> for CacheMonitorCommand<'_> {
    #[inline]
    fn execute(self) -> Result<CacheStats> {
        return self.execute_cache_monitoring();
    }
}

/// Cache monitoring implementation methods
impl CacheMonitorCommand<'_> {
    /// Calculate final cache statistics
    #[inline]
    fn calculate_cache_stats(
        self,
        start_time: Instant,
        bytes_processed: u64,
        sections_extracted: usize,
        cache_state: &CacheState,
    ) -> Result<CacheStats> {
        let processing_duration_ms = match u64::try_from(start_time.elapsed().as_millis()) {
            Ok(duration) => duration,
            Err(conversion_error) => {
                return Err(crate::error::CpinfoError::resource_exhaustion(
                    "duration_conversion",
                    &format!("Duration conversion error: {conversion_error}"),
                ));
            }
        };

        let cache_hits = cache_state.hits;
        let cache_misses = cache_state.misses;
        let total_requests = cache_hits + cache_misses;

        let cache_hit_rate = if total_requests > 0 {
            #[allow(
                clippy::cast_precision_loss,
                reason = "Statistical calculations require f64 precision"
            )]
            let hits_f64 = cache_hits as f64;
            #[allow(
                clippy::cast_precision_loss,
                reason = "Statistical calculations require f64 precision"
            )]
            let total_f64 = total_requests as f64;
            #[allow(
                clippy::float_arithmetic,
                reason = "Statistical calculations require floating point arithmetic"
            )]
            let hit_rate = hits_f64 / total_f64;
            hit_rate
        } else {
            0.0
        };

        #[allow(
            clippy::cast_precision_loss,
            reason = "Memory calculations require f64 precision"
        )]
        let base_memory_mb = self.config.max_memory_mb as f64;
        #[allow(
            clippy::float_arithmetic,
            reason = "Memory calculations require floating point arithmetic"
        )]
        let base_memory = base_memory_mb / 2.0; // 50% of base
        #[allow(
            clippy::cast_precision_loss,
            reason = "Memory calculations require f64 precision"
        )]
        let cache_memory = self.config.cache_size_mb as f64;
        #[allow(
            clippy::float_arithmetic,
            reason = "Memory calculations require floating point arithmetic"
        )]
        let peak_memory_mb = cache_memory.mul_add(0.3, base_memory);

        return Ok(CacheStats {
            bytes_processed,
            cache_hit_rate,
            cache_hits,
            cache_misses,
            cache_size_mb: cache_memory,
            peak_memory_mb,
            processing_duration_ms,
            sections_extracted,
        });
    }

    /// Execute cache monitoring with separated logic
    #[inline]
    fn execute_cache_monitoring(self) -> Result<CacheStats> {
        let mut cache_state = CacheState::default();

        let start_time = Instant::now();
        let file = match std::fs::File::open(self.path) {
            Ok(file) => file,
            Err(io_error) => return Err(io_error.into()),
        };
        let mut reader = BufReader::with_capacity(self.config.buffer_size, file);
        let mut line_buffer = String::with_capacity(LINE_BUFFER_CAPACITY);
        let mut bytes_processed = 0_u64;
        let mut sections_extracted = 0;
        let mut current_section = String::new();
        let mut in_section = false;
        let mut current_cache_index: Option<usize> = None;

        let common_patterns = [
            "General Information",
            "Network Configuration",
            "Security Policy",
            "System Status",
            "Performance Metrics",
            "Hardware Information",
        ];

        return self.process_cache_lines(
            &mut reader,
            &mut line_buffer,
            &mut bytes_processed,
            &mut sections_extracted,
            &mut current_section,
            &mut in_section,
            &mut current_cache_index,
            &common_patterns,
            &mut cache_state,
            start_time,
        );
    }

    /// Process lines for cache monitoring
    #[inline]
    #[allow(
        clippy::too_many_arguments,
        reason = "Complex monitoring requires many parameters"
    )]
    fn process_cache_lines(
        self,
        reader: &mut BufReader<std::fs::File>,
        line_buffer: &mut String,
        bytes_processed: &mut u64,
        sections_extracted: &mut usize,
        current_section: &mut String,
        in_section: &mut bool,
        current_cache_index: &mut Option<usize>,
        common_patterns: &[&str; 6],
        cache_state: &mut CacheState,
        start_time: Instant,
    ) -> Result<CacheStats> {
        let mut should_store_section = false;
        loop {
            line_buffer.clear();
            let bytes_read = match reader.read_line(line_buffer) {
                Ok(bytes) => bytes,
                Err(io_error) => return Err(io_error.into()),
            };
            if bytes_read == 0 {
                break;
            }

            *bytes_processed += bytes_read as u64;
            let line = line_buffer.trim();

            let mut matched_pattern: Option<(usize, &str)> = None;
            for (index, pattern) in common_patterns.iter().copied().enumerate() {
                if line.contains(pattern) {
                    matched_pattern = Some((index, pattern));
                    break;
                }
            }

            if let Some((cache_index, cache_key)) = matched_pattern {
                line.clone_into(current_section);
                *in_section = true;
                *current_cache_index = Some(cache_index);
                should_store_section = true;

                if cache_state.entries.contains_key(cache_key) {
                    cache_state.hits += 1;
                    should_store_section = false;
                    std::thread::sleep(Duration::from_micros(10));
                    continue;
                }
                cache_state.misses += 1;
                std::thread::sleep(Duration::from_micros(100));
            } else if line == SECTION_DELIMITER && *in_section {
                let mut cache_key = "unknown";
                if let Some(index_ref) = current_cache_index.as_ref() {
                    if let Some(pattern) = common_patterns.get(*index_ref) {
                        cache_key = *pattern;
                    }
                }
                if should_store_section {
                    cache_state
                        .entries
                        .insert(cache_key.to_owned(), current_section.clone());
                }
                *in_section = false;
                should_store_section = false;
                *current_cache_index = None;
                *sections_extracted += 1;
            } else {
                // Handle other lines
            }

            if *sections_extracted > 50 {
                break;
            }
        }

        return self.calculate_cache_stats(
            start_time,
            *bytes_processed,
            *sections_extracted,
            cache_state,
        );
    }
}

// Memory monitoring functionality
/// Memory monitoring command
pub struct MemoryMonitorCommand<'path_lifetime> {
    config: &'path_lifetime PerformanceConfig,
    path: &'path_lifetime Path,
}

impl<'path_lifetime> MemoryMonitorCommand<'path_lifetime> {
    /// Create new memory monitoring command
    #[must_use]
    #[inline]
    pub const fn new(
        path: &'path_lifetime Path,
        config: &'path_lifetime PerformanceConfig,
    ) -> Self {
        return Self { config, path };
    }
}

/// Memory monitoring helper methods
impl MemoryMonitorCommand<'_> {
    /// Check if current memory usage is within limits
    #[inline]
    fn check_memory_usage(&self, current_memory_mb: f64) -> Result<()> {
        #[allow(
            clippy::cast_precision_loss,
            reason = "Memory calculations require f64 precision"
        )]
        let max_memory_f64 = self.config.max_memory_mb as f64;
        if current_memory_mb > max_memory_f64 {
            return Err(crate::error::CpinfoError::resource_exhaustion(
                "memory_limit_exceeded",
                &format!(
                    "Memory usage {current_memory_mb:.2} MB exceeds limit {} MB",
                    self.config.max_memory_mb
                ),
            ));
        }
        return Ok(());
    }
}

impl MonitorCommand<MemoryStats> for MemoryMonitorCommand<'_> {
    #[inline]
    fn execute(self) -> Result<MemoryStats> {
        let start_time = Instant::now();
        let file = match std::fs::File::open(self.path) {
            Ok(file_handle) => file_handle,
            Err(io_error) => return Err(io_error.into()),
        };
        let mut reader = BufReader::with_capacity(self.config.buffer_size, file);
        let mut line_buffer = String::with_capacity(LINE_BUFFER_CAPACITY);

        let mut bytes_processed = 0_u64;
        let mut sections_extracted = 0_usize;
        let mut in_section = false;

        // Simulate initial memory allocation
        #[allow(
            clippy::cast_precision_loss,
            reason = "Memory calculations require f64 precision"
        )]
        let memory_mb_f64 = self.config.max_memory_mb as f64;
        #[allow(
            clippy::float_arithmetic,
            reason = "Memory calculations require floating point arithmetic"
        )]
        let base_memory = memory_mb_f64 / 2.5; // 40% of max memory
        let mut current_memory = base_memory;
        let mut peak_memory_mb = current_memory;

        loop {
            line_buffer.clear();
            let bytes_read = match reader.read_line(&mut line_buffer) {
                Ok(bytes) => bytes,
                Err(io_error) => return Err(io_error.into()),
            };
            if bytes_read == 0 {
                break;
            }

            bytes_processed += bytes_read as u64;
            let line = line_buffer.trim();

            // Memory usage increases with processing
            // Memory usage increases with processing
            #[allow(
                clippy::float_arithmetic,
                reason = "Memory simulation requires floating point arithmetic"
            )]
            {
                let memory_increment = 0.001;
                current_memory += memory_increment; // Small increase per line
            };
            if current_memory > peak_memory_mb {
                peak_memory_mb = current_memory;
            }

            match self.check_memory_usage(current_memory) {
                Ok(()) => {}
                Err(memory_error) => return Err(memory_error),
            }

            if line == SECTION_DELIMITER {
                if in_section {
                    sections_extracted += 1;
                    // Simulate section processing memory spike
                    // Simulate section processing memory spike
                    #[allow(
                        clippy::float_arithmetic,
                        reason = "Memory simulation requires floating point arithmetic"
                    )]
                    {
                        let memory_spike = 0.5;
                        current_memory += memory_spike; // Section processing spike
                    };
                    if current_memory > peak_memory_mb {
                        peak_memory_mb = current_memory;
                    }
                    match self.check_memory_usage(current_memory) {
                        Ok(()) => {}
                        Err(memory_error) => return Err(memory_error),
                    }
                }
                in_section = !in_section;

                // Memory cleanup after section
                if !in_section {
                    #[allow(
                        clippy::float_arithmetic,
                        reason = "Memory simulation requires floating point arithmetic"
                    )]
                    {
                        current_memory *= 0.95; // 5% reduction
                    }
                }
            }

            if sections_extracted > 50 {
                break;
            }
        }

        let processing_duration_ms = match u64::try_from(start_time.elapsed().as_millis()) {
            Ok(duration) => duration,
            Err(conversion_error) => {
                return Err(crate::error::CpinfoError::resource_exhaustion(
                    "duration_conversion",
                    &format!("Duration conversion error: {conversion_error}"),
                ));
            }
        };

        return Ok(MemoryStats {
            bytes_processed,
            peak_memory_mb,
            processing_duration_ms,
            sections_extracted,
        });
    }
}

// Resource monitoring functionality
/// Resource monitoring command
pub struct ResourceMonitorCommand<'path_lifetime> {
    config: &'path_lifetime PerformanceConfig,
    path: &'path_lifetime Path,
}

impl<'path_lifetime> ResourceMonitorCommand<'path_lifetime> {
    /// Create new resource monitoring command
    #[must_use]
    #[inline]
    pub const fn new(
        path: &'path_lifetime Path,
        config: &'path_lifetime PerformanceConfig,
    ) -> Self {
        return Self { config, path };
    }
}

impl MonitorCommand<ResourceStats> for ResourceMonitorCommand<'_> {
    #[inline]
    fn execute(self) -> Result<ResourceStats> {
        let start_time = Instant::now();
        let file = match std::fs::File::open(self.path) {
            Ok(file) => file,
            Err(io_error) => return Err(io_error.into()),
        };
        let mut reader = BufReader::with_capacity(self.config.buffer_size, file);
        let mut line_buffer = String::with_capacity(LINE_BUFFER_CAPACITY);

        let mut bytes_processed = 0_u64;
        let mut sections_extracted = 0_usize;
        let mut in_section = false;
        let mut cpu_usage_samples = Vec::new();
        let mut memory_samples = Vec::new();
        let mut io_samples = Vec::new();

        // Simulate CPU usage tracking
        let mut current_cpu_usage = 15.0_f64; // Start at 15% CPU

        // Process file content and collect samples
        match process_resource_monitoring_lines(
            &mut reader,
            &mut line_buffer,
            &mut bytes_processed,
            &mut sections_extracted,
            &mut in_section,
            &mut current_cpu_usage,
            &mut cpu_usage_samples,
            &mut memory_samples,
            &mut io_samples,
        ) {
            Ok(value) => value,
            Err(error) => return Err(error),
        }

        // Calculate final statistics
        return calculate_resource_statistics(
            start_time,
            bytes_processed,
            sections_extracted,
            cpu_usage_samples,
            memory_samples,
            io_samples,
        );
    }
}

// Speed monitoring functionality

/// Speed monitoring command
pub struct SpeedMonitorCommand<'path_lifetime> {
    config: &'path_lifetime PerformanceConfig,
    path: &'path_lifetime Path,
}

impl<'path_lifetime> SpeedMonitorCommand<'path_lifetime> {
    /// Create new speed monitoring command
    #[must_use]
    #[inline]
    pub const fn new(
        path: &'path_lifetime Path,
        config: &'path_lifetime PerformanceConfig,
    ) -> Self {
        return Self { config, path };
    }
}

impl MonitorCommand<SpeedStats> for SpeedMonitorCommand<'_> {
    #[inline]
    fn execute(self) -> Result<SpeedStats> {
        let start_time = Instant::now();
        let file = match std::fs::File::open(self.path) {
            Ok(file) => file,
            Err(io_error) => return Err(io_error.into()),
        };
        let mut reader = BufReader::with_capacity(self.config.buffer_size, file);
        let mut line_buffer = String::with_capacity(LINE_BUFFER_CAPACITY);

        let mut bytes_processed = 0_u64;
        let mut sections_extracted = 0_usize;
        let mut in_section = false;
        let mut speed_samples = Vec::new();
        let mut last_speed_check = start_time;

        loop {
            line_buffer.clear();
            let bytes_read = match reader.read_line(&mut line_buffer) {
                Ok(bytes) => bytes,
                Err(io_error) => return Err(io_error.into()),
            };
            if bytes_read == 0 {
                break;
            }

            bytes_processed += bytes_read as u64;
            let line = line_buffer.trim();

            // Calculate speed every MEMORY_CHECK_INTERVAL bytes
            if bytes_processed.wrapping_rem(MEMORY_CHECK_INTERVAL) == 0 {
                let elapsed = last_speed_check.elapsed();
                if elapsed.as_millis() > 0 {
                    #[allow(
                        clippy::cast_precision_loss,
                        reason = "Speed calculations require f64 precision"
                    )]
                    #[allow(
                        clippy::float_arithmetic,
                        reason = "Speed calculations require floating point arithmetic"
                    )]
                    let speed_mbps =
                        (MEMORY_CHECK_INTERVAL as f64 / 1_000_000.0) / elapsed.as_secs_f64();
                    speed_samples.push(speed_mbps);
                    last_speed_check = Instant::now();
                }
            }

            if line == SECTION_DELIMITER {
                if in_section {
                    sections_extracted += 1;
                }
                in_section = !in_section;
            }

            if sections_extracted > 50 {
                break;
            }
        }

        let total_duration = start_time.elapsed();
        let processing_duration_ms = match u64::try_from(total_duration.as_millis()) {
            Ok(duration) => duration,
            Err(conversion_error) => {
                return Err(crate::error::CpinfoError::resource_exhaustion(
                    "duration_conversion",
                    &format!("Duration conversion error: {conversion_error}"),
                ));
            }
        };

        // Calculate final speeds using actual struct fields
        let sections_per_second = if total_duration.as_secs_f64() > 0.0 {
            #[allow(
                clippy::cast_precision_loss,
                reason = "Speed calculations require f64 precision"
            )]
            #[allow(
                clippy::float_arithmetic,
                reason = "Speed calculations require floating point arithmetic"
            )]
            let rate = sections_extracted as f64 / total_duration.as_secs_f64();
            rate
        } else {
            0.0
        };

        let bytes_per_second = if total_duration.as_secs_f64() > 0.0 {
            #[allow(
                clippy::cast_precision_loss,
                reason = "Speed calculations require f64 precision"
            )]
            #[allow(
                clippy::float_arithmetic,
                reason = "Speed calculations require floating point arithmetic"
            )]
            let rate = bytes_processed as f64 / total_duration.as_secs_f64();
            rate
        } else {
            0.0
        };

        return Ok(SpeedStats {
            bytes_per_second,
            processing_duration_ms,
            sections_per_second,
            total_sections: sections_extracted,
        });
    }
}

/// Process monitoring lines and collect resource usage samples
///
/// # Arguments
///
/// * `reader` - Buffered reader for input file
/// * `line_buffer` - Reusable string buffer for line reading
/// * `bytes_processed` - Mutable reference to total bytes processed counter
/// * `sections_extracted` - Mutable reference to sections extracted counter
/// * `in_section` - Mutable reference to section processing state
/// * `current_cpu_usage` - Mutable reference to current CPU usage simulation
/// * `cpu_usage_samples` - Vector to collect CPU usage samples
/// * `memory_samples` - Vector to collect memory usage samples
/// * `io_samples` - Vector to collect IO rate samples
///
/// # Returns
///
/// Ok(()) on successful processing
///
/// # Errors
///
/// Returns error if file reading fails or other IO errors occur
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Helper function for splitting long execute function in ResourceMonitorCommand"
)]
#[allow(
    clippy::too_many_arguments,
    reason = "Required arguments for resource monitoring data collection"
)]
fn process_resource_monitoring_lines(
    reader: &mut BufReader<std::fs::File>,
    line_buffer: &mut String,
    bytes_processed: &mut u64,
    sections_extracted: &mut usize,
    in_section: &mut bool,
    current_cpu_usage: &mut f64,
    cpu_usage_samples: &mut Vec<f64>,
    memory_samples: &mut Vec<f64>,
    io_samples: &mut Vec<f64>,
) -> Result<()> {
    loop {
        line_buffer.clear();
        let bytes_read = match reader.read_line(line_buffer) {
            Ok(bytes) => bytes,
            Err(io_error) => return Err(io_error.into()),
        };
        if bytes_read == 0 {
            break;
        }

        *bytes_processed += bytes_read as u64;
        let line = line_buffer.trim();

        // Simulate CPU usage fluctuation (deterministic based on bytes processed)
        #[allow(
            clippy::float_arithmetic,
            reason = "CPU usage simulation requires floating point arithmetic"
        )]
        let random_factor = {
            // Use a safe alternative to modulo for generating variation
            let variation_base = bytes_processed.wrapping_rem(500);
            #[allow(
                clippy::cast_precision_loss,
                reason = "Statistical calculations require f64 precision"
            )]
            let variation_f64 = variation_base as f64 / 100.0;
            variation_f64 - 2.5 // ±2.5% variation
        };
        #[allow(
            clippy::float_arithmetic,
            reason = "CPU usage simulation requires floating point arithmetic"
        )]
        {
            let cpu_variation = random_factor;
            *current_cpu_usage += cpu_variation; // CPU variation
        };
        *current_cpu_usage = current_cpu_usage.clamp(5.0, 95.0);
        cpu_usage_samples.push(*current_cpu_usage);

        // Add memory and IO samples
        #[allow(
            clippy::float_arithmetic,
            reason = "Memory and IO calculations require floating point arithmetic"
        )]
        memory_samples.push(*current_cpu_usage * 2.0); // Simple relationship
        #[allow(
            clippy::cast_precision_loss,
            reason = "IO calculations require f64 precision"
        )]
        #[allow(
            clippy::float_arithmetic,
            reason = "IO rate calculations require floating point arithmetic"
        )]
        io_samples.push(bytes_read as f64 / 1024.0); // KB

        if line == SECTION_DELIMITER {
            if *in_section {
                *sections_extracted += 1;
                // Section processing increases CPU usage temporarily
                #[allow(
                    clippy::float_arithmetic,
                    reason = "CPU usage simulation requires floating point arithmetic"
                )]
                {
                    let cpu_increase = 10.0;
                    *current_cpu_usage += cpu_increase; // Section processing increase
                };
                *current_cpu_usage = current_cpu_usage.min(95.0);
            }
            *in_section = !*in_section;
        }

        if *sections_extracted > 50 {
            break;
        }
    }
    return Ok(());
}

/// Calculate final resource monitoring statistics
///
/// # Arguments
///
/// * `start_time` - Start time of monitoring operation
/// * `bytes_processed` - Total bytes processed during monitoring
/// * `sections_extracted` - Total sections extracted during monitoring
/// * `cpu_usage_samples` - Vector of collected CPU usage samples
/// * `memory_samples` - Vector of collected memory usage samples
/// * `io_samples` - Vector of collected IO rate samples
///
/// # Returns
///
/// `ResourceStats` struct with calculated statistics on success
///
/// # Errors
///
/// Returns error if duration conversion fails or calculations are invalid
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Helper function for splitting long execute function in ResourceMonitorCommand"
)]
fn calculate_resource_statistics(
    start_time: Instant,
    bytes_processed: u64,
    sections_extracted: usize,
    cpu_usage_samples: Vec<f64>,
    memory_samples: Vec<f64>,
    io_samples: Vec<f64>,
) -> Result<ResourceStats> {
    let processing_duration_ms = match u64::try_from(start_time.elapsed().as_millis()) {
        Ok(duration) => duration,
        Err(conversion_error) => {
            return Err(crate::error::CpinfoError::resource_exhaustion(
                "duration_conversion",
                &format!("Duration conversion error: {conversion_error}"),
            ));
        }
    };

    let peak_cpu_percent = cpu_usage_samples
        .iter()
        .fold(0.0_f64, |accumulator, &sample_value| {
            return accumulator.max(sample_value);
        });
    let peak_memory_mb = memory_samples
        .iter()
        .fold(0.0_f64, |accumulator, &sample_value| {
            return accumulator.max(sample_value);
        });
    let total_io: f64 = io_samples.iter().sum();
    let average_io_rate_mb_per_sec = if processing_duration_ms > 0 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "IO rate calculations require f64 precision"
        )]
        #[allow(
            clippy::float_arithmetic,
            reason = "IO rate calculations require floating point arithmetic"
        )]
        let rate = (total_io / 1024.0) / (processing_duration_ms as f64 / 1000.0);
        rate
    } else {
        0.0
    };

    return Ok(ResourceStats {
        average_io_rate_mb_per_sec,
        cpu_samples: cpu_usage_samples,
        io_samples,
        memory_samples,
        peak_cpu_percent,
        peak_memory_mb,
        total_bytes_read: bytes_processed,
        total_bytes_written: sections_extracted as u64 * 1024, // Assume 1KB per section
        total_monitoring_duration_ms: processing_duration_ms,
    });
}
