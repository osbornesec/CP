//! Configuration and utilities for section writers
//!
//! This module provides configuration structures and utility functions
//! that support the section writing process.

/// Configuration for section writing operations
pub struct WriterConfig {
    /// Buffer size for file writes
    pub buffer_size: usize,
    /// Whether to show progress bars for large sections
    pub show_progress: bool,
    /// Minimum lines to trigger progress reporting
    pub progress_threshold: usize,
}

impl Default for WriterConfig {
    fn default() -> Self {
        Self {
            buffer_size: 64 * 1024,
            show_progress: true,
            progress_threshold: 10_000,
        }
    }
}

impl WriterConfig {
    /// Create a new writer configuration
    pub fn new(buffer_size: usize, show_progress: bool, progress_threshold: usize) -> Self {
        Self {
            buffer_size,
            show_progress,
            progress_threshold,
        }
    }

    /// Create configuration optimized for performance
    pub fn for_performance() -> Self {
        Self {
            buffer_size: 128 * 1024, // Larger buffer for performance
            show_progress: false,     // No progress overhead
            progress_threshold: usize::MAX,
        }
    }

    /// Create configuration optimized for user experience
    pub fn for_user_experience() -> Self {
        Self {
            buffer_size: 64 * 1024,
            show_progress: true,
            progress_threshold: 1_000, // Lower threshold for better feedback
        }
    }
}

/// Sanitize a section name to be a valid filename
///
/// Replaces problematic characters with underscores to ensure
/// the resulting filename is valid on all major filesystems.
///
/// # Arguments
///
/// * `name` - The section name to sanitize
///
/// # Returns
///
/// A sanitized filename string suitable for all major filesystems
///
/// # Examples
///
/// ```
/// use cpinfo::extraction::writer::config::sanitize_filename;
///
/// assert_eq!(sanitize_filename("Normal Name"), "Normal_Name");
/// assert_eq!(sanitize_filename("Special<>Chars"), "Special__Chars");
/// ```
pub fn sanitize_filename(name: &str) -> String {
    name.replace(' ', "_")
        .replace('/', "_")
        .replace(':', "_")
        .replace(['<', '>', '"', '|', '?', '*'], "_")
}

