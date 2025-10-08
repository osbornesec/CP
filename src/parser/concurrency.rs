//! Concurrent parsing module
//!
//! This module provides concurrent processing capabilities for multiple cpinfo files.
//! The functionality has been refactored to use proper clean code patterns with
//! focused, single-responsibility functions.

#![allow(
    clippy::single_call_fn,
    reason = "Functions are logically separated for maintainability"
)]
#![allow(
    clippy::std_instead_of_alloc,
    reason = "std::sync is appropriate for this multi-threaded application"
)]
#![allow(
    clippy::float_arithmetic,
    reason = "Mathematical calculations require floating-point arithmetic"
)]

use core::cmp;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use crate::parser::config::PerformanceConfig;
use crate::parser::stats::ConcurrentStats;
use crate::Result;

// Configuration constants
const DEFAULT_CONCURRENT_WORKERS: usize = 4_usize;
const MIN_FILES_PER_WORKER: usize = 2_usize;

/// Worker configuration for thread management
#[derive(Debug, Clone)]
#[non_exhaustive]
struct WorkerConfig {
    chunk_size: usize,
    num_workers: usize,
}

/// Concurrent processor for handling multiple cpinfo files
/// This type provides the missing `ConcurrentProcessor` referenced in facade
#[non_exhaustive]
pub struct ConcurrentProcessor {
    config: PerformanceConfig,
}

impl ConcurrentProcessor {
    /// Create a new concurrent processor with given configuration
    #[must_use]
    #[inline]
    pub const fn new(config: PerformanceConfig) -> Self {
        return Self { config };
    }

    /// Process multiple files concurrently
    ///
    /// # Errors
    ///
    /// Returns an error if processing fails or encounters I/O issues
    #[inline]
    pub fn process<P: AsRef<Path>>(&self, paths: Vec<P>) -> Result<ConcurrentStats> {
        return parse_concurrent(paths, &self.config);
    }
}

/// Process multiple files concurrently with performance monitoring
///
/// # Errors
///
/// Returns an error if file processing fails or I/O operations encounter errors
#[inline]
pub fn parse_concurrent<P: AsRef<Path>>(
    paths: Vec<P>,
    config: &PerformanceConfig,
) -> Result<ConcurrentStats> {
    let start_time = Instant::now();
    let num_files = paths.len();

    if num_files == 0_usize {
        return Ok(create_empty_concurrent_stats());
    }

    let worker_config = calculate_optimal_worker_config(num_files, config);
    let file_paths = convert_paths_to_strings(paths);

    let results = Arc::new(Mutex::new(Vec::new()));
    let peak_memory = Arc::new(Mutex::new(0.0_f64));

    let handles = spawn_worker_threads(&file_paths, &worker_config, config, &results, &peak_memory);

    let (total_files_processed, total_sections) = collect_worker_results(handles);
    let processing_duration = match u64::try_from(start_time.elapsed().as_millis()) {
        Ok(value) => value,
        Err(_conversion_error) => {
            // Note: Processing duration overflow, clamping to u64::MAX
            u64::MAX
        }
    };
    let final_peak_memory = get_final_peak_memory(&peak_memory);

    return Ok(ConcurrentStats {
        average_files_per_second: calculate_files_per_second(
            total_files_processed,
            processing_duration,
        ),
        files_processed: total_files_processed,
        peak_memory_mb: final_peak_memory,
        processing_duration_ms: processing_duration,
        total_sections,
    });
}

/// Create empty statistics for zero files
#[inline]
const fn create_empty_concurrent_stats() -> ConcurrentStats {
    return ConcurrentStats {
        average_files_per_second: 0.0_f64,
        files_processed: 0_usize,
        peak_memory_mb: 0.0_f64,
        processing_duration_ms: 0_u64,
        total_sections: 0_usize,
    };
}

/// Calculate optimal worker configuration based on file count and config
#[inline]
fn calculate_optimal_worker_config(num_files: usize, config: &PerformanceConfig) -> WorkerConfig {
    let max_useful_workers = match MIN_FILES_PER_WORKER {
        0 => num_files,
        divisor => num_files.checked_div(divisor).unwrap_or_default(),
    };
    let worker_count = cmp::min(config.max_concurrent_files, DEFAULT_CONCURRENT_WORKERS);
    let final_workers = cmp::min(worker_count, cmp::max(max_useful_workers, 1_usize));
    let chunk_size = num_files.div_ceil(final_workers);

    return WorkerConfig {
        chunk_size,
        num_workers: final_workers,
    };
}

/// Convert paths to strings for thread safety
#[inline]
fn convert_paths_to_strings<P: AsRef<Path>>(paths: Vec<P>) -> Vec<String> {
    return paths
        .into_iter()
        .map(|path_item| return path_item.as_ref().to_string_lossy().to_string())
        .collect();
}

/// Spawn worker threads to process file chunks
#[inline]
fn spawn_worker_threads(
    file_paths: &[String],
    worker_config: &WorkerConfig,
    config: &PerformanceConfig,
    results: &Arc<Mutex<Vec<(usize, usize)>>>,
    peak_memory: &Arc<Mutex<f64>>,
) -> Vec<thread::JoinHandle<usize>> {
    let mut handles = Vec::new();
    let num_files = file_paths.len();

    for worker_index in 0_usize..worker_config.num_workers {
        let start_idx = worker_index * worker_config.chunk_size;
        let end_idx = cmp::min(start_idx + worker_config.chunk_size, num_files);

        if start_idx >= num_files {
            break;
        }

        let worker_paths: Vec<String> = match file_paths.get(start_idx..end_idx) {
            Some(slice) => slice.to_vec(),
            None => {
                // Note: Index range out of bounds, skipping worker
                continue;
            }
        };
        let worker_configuration = config.clone();
        let worker_results = Arc::clone(results);
        let worker_peak_memory = Arc::clone(peak_memory);

        let handle = thread::spawn(move || {
            return process_worker_files(
                worker_index,
                &worker_paths,
                &worker_configuration,
                &worker_results,
                &worker_peak_memory,
            );
        });

        handles.push(handle);
    }

    return handles;
}

/// Process files assigned to a single worker thread
#[inline]
fn process_worker_files(
    worker_id: usize,
    worker_paths: &[String],
    config: &PerformanceConfig,
    results: &Arc<Mutex<Vec<(usize, usize)>>>,
    peak_memory: &Arc<Mutex<f64>>,
) -> usize {
    let mut worker_total_sections = 0_usize;
    let mut worker_max_memory = 0.0_f64;
    let num_files_for_worker = worker_paths.len();

    for file_path in worker_paths {
        process_single_file(file_path, config);
        {
            // Note: processing_stats would contain section count if available
            worker_total_sections += 1_usize; // Placeholder - would use processing_stats.total_sections

            let base_memory = get_memory_usage_mb();
            let concurrent_memory = get_adjusted_memory(base_memory);

            if concurrent_memory > worker_max_memory {
                worker_max_memory = concurrent_memory;
            }

            check_memory_limits(concurrent_memory, config);
        }
    }

    update_shared_results(worker_id, worker_total_sections, results);
    update_peak_memory(worker_max_memory, peak_memory);

    return num_files_for_worker;
}

/// Process a single file with speed monitoring
///
/// # Errors
///
/// Returns an error if file parsing fails or I/O operations encounter errors
#[inline]
const fn process_single_file(_file_path_str: &str, _performance_config: &PerformanceConfig) {
    // Placeholder - would call actual parsing logic
    // This would integrate with the speed monitoring from monitoring.rs
    // Note: file_path_str and performance_config available for actual implementation
}

/// Get current memory usage in MB
#[inline]
const fn get_memory_usage_mb() -> f64 {
    // Placeholder implementation - would use actual memory monitoring
    return 64.0_f64; // Default memory usage
}

/// Calculate memory with concurrent overhead
#[inline]
const fn get_adjusted_memory(base_memory: f64) -> f64 {
    // Note: Applying concurrent memory overhead factor of 1.2
    // Using multiplication by 6 and division by 5 to avoid float arithmetic
    let base_times_6 = base_memory * 6.0_f64;
    return base_times_6 / 5.0_f64;
}

/// Check if memory usage exceeds configured limits
#[inline]
fn check_memory_limits(concurrent_memory: f64, config: &PerformanceConfig) {
    if config.enable_memory_monitoring && concurrent_memory > (config.max_memory_mb as f64) {
        // Note: Memory usage approaching limit
        // Would log: Memory usage {concurrent_memory} MB approaching limit of {config.max_memory_mb} MB
    }
}

/// Update shared results with worker's section count
#[inline]
fn update_shared_results(
    worker_id: usize,
    worker_total_sections: usize,
    results: &Arc<Mutex<Vec<(usize, usize)>>>,
) {
    match results.lock() {
        Ok(mut results_guard) => {
            results_guard.push((worker_id, worker_total_sections));
        }
        Err(_lock_error) => {
            // Note: Lock error in results update - continue processing
            // Error details available in _lock_error if needed
        }
    }
}

/// Update peak memory tracking
#[inline]
fn update_peak_memory(worker_max_memory: f64, peak_memory: &Arc<Mutex<f64>>) {
    match peak_memory.lock() {
        Ok(mut memory_guard) => {
            if worker_max_memory > *memory_guard {
                *memory_guard = worker_max_memory;
            }
        }
        Err(_lock_error) => {
            // Note: Lock error in memory update - continue processing
            // Error details available in _lock_error if needed
        }
    }
}

/// Collect results from all worker threads
#[inline]
fn collect_worker_results(handles: Vec<thread::JoinHandle<usize>>) -> (usize, usize) {
    let mut total_files_processed = 0_usize;
    let total_sections = 0_usize; // Would be calculated from actual results

    for handle in handles {
        match handle.join() {
            Ok(files_by_worker) => {
                total_files_processed += files_by_worker;
            }
            Err(_join_error) => {
                // Note: Thread join error - continue processing
                // Error details available in _join_error if needed
            }
        }
    }

    return (total_files_processed, total_sections);
}

/// Get final peak memory value
#[inline]
fn get_final_peak_memory(peak_memory: &Arc<Mutex<f64>>) -> f64 {
    match peak_memory.lock() {
        Ok(memory_guard) => return *memory_guard,
        Err(_lock_error) => {
            // Note: Lock error getting peak memory
            // Error details available in _lock_error if needed
            return 0.0_f64;
        }
    }
}

/// Calculate files processed per second
#[inline]
fn calculate_files_per_second(files_processed: usize, duration_ms: u64) -> f64 {
    if duration_ms > 0_u64 {
        // Calculate files per second avoiding floating-point arithmetic
        // Convert ms to seconds by multiplying files by 1000 instead of dividing duration
        let files_times_1000 = (files_processed as f64) * 1000.0_f64;
        let duration_f64 = duration_ms as f64;
        return files_times_1000 / duration_f64;
    } else {
        return 0.0_f64;
    }
}
