//! Concurrency operations facade
//!
//! This module contains all concurrent processing functionality
//! for handling multiple files simultaneously.

use std::path::Path;

use crate::parser::stats::ConcurrentStats;

/// Concurrency operations facade
#[non_exhaustive]
pub struct ConcurrencyFacade;

impl ConcurrencyFacade {
    /// Process multiple files concurrently
    ///
    /// Delegates to the concurrency module for multi-file concurrent processing.
    /// This method handles resource management and load balancing across multiple files.
    ///
    /// # Arguments
    ///
    /// * `file_paths` - Vector of file paths to process concurrently
    ///
    /// # Returns
    ///
    /// `ConcurrentStats` containing processing statistics for all files
    ///
    /// # Errors
    ///
    /// Returns an error if concurrent processing fails or resource limits are exceeded.
    #[inline]
    pub async fn process_files_concurrent<P: AsRef<Path> + Send + 'static>(
        file_paths: Vec<P>,
    ) -> crate::error::Result<ConcurrentStats> {
        use crate::parser::concurrency::ConcurrentProcessor;
        use crate::parser::config::PerformanceConfig;

        let config = PerformanceConfig::default();
        let processor = ConcurrentProcessor::new(config);

        // Convert paths to PathBuf for thread safety
        let paths: Vec<std::path::PathBuf> = file_paths
            .into_iter()
            .map(|path| return path.as_ref().to_path_buf())
            .collect();

        // Convert async call to sync since the underlying implementation is sync
        let spawn_result =
            tokio::task::spawn_blocking(move || return processor.process(paths)).await;

        match spawn_result {
            Ok(result) => return result,
            Err(join_error) => {
                return Err(crate::error::CpinfoError::AsyncTaskError {
                    message: format!("Concurrent processing failed: {join_error}"),
                })
            }
        }
    }
}
