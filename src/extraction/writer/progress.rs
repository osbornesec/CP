//! Progress reporting utilities for section extraction
//!
//! This module handles progress bar creation and management for large section
//! extraction operations, providing visual feedback to users during processing.

use indicatif::{ProgressBar, ProgressStyle};
use tracing::info;

use crate::error::Result;

/// Configuration for progress reporting
pub struct ProgressConfig {
    /// Whether to show progress bars
    pub show_progress: bool,
    /// Minimum lines to trigger progress reporting
    pub progress_threshold: usize,
}

impl ProgressConfig {
    /// Create a new progress configuration
    pub fn new(show_progress: bool, progress_threshold: usize) -> Self {
        Self {
            show_progress,
            progress_threshold,
        }
    }
}

/// Create progress bar if section is large enough
///
/// Sets up a progress bar with appropriate styling and messaging for
/// sections that exceed the configured threshold.
///
/// # Arguments
///
/// * `estimated_lines` - Number of lines expected to be processed
/// * `section_name` - Name of the section being processed
/// * `section_index` - Current section number (1-based)
/// * `total_sections` - Total number of sections
/// * `config` - Progress configuration
///
/// # Returns
///
/// `Some(ProgressBar)` if progress reporting should be enabled, `None` otherwise
///
/// # Errors
///
/// Returns error if progress bar template is invalid
pub fn create_progress_bar_if_needed(
    estimated_lines: usize,
    section_name: &str,
    section_index: usize,
    total_sections: usize,
    config: &ProgressConfig,
) -> Result<Option<ProgressBar>> {
    if !config.show_progress || estimated_lines < config.progress_threshold {
        return Ok(None);
    }

    log_large_section_detected(estimated_lines);
    let pb = create_styled_progress_bar(estimated_lines)?;
    set_progress_message(&pb, section_name, section_index, total_sections);

    Ok(Some(pb))
}

/// Log that a large section has been detected
fn log_large_section_detected(estimated_lines: usize) {
    info!(
        "   \u{1f6a8} LARGE SECTION DETECTED: {} lines - enabling progress reporting",
        estimated_lines
    );
}

/// Create a styled progress bar
fn create_styled_progress_bar(estimated_lines: usize) -> Result<ProgressBar> {
    let pb = ProgressBar::new(estimated_lines.try_into().unwrap_or(u64::MAX));
    let template = "   {msg} [{bar:40.cyan/blue}] {pos}/{len} lines ({percent}%) ETA: {eta}";
    
    let style = match ProgressStyle::default_bar()
        .template(template)
        .map_err(|e| {
            crate::error::CpinfoError::validation_error(format!(
                "Invalid progress bar template: {e}"
            ))
        }) {
        Ok(s) => s,
        Err(e) => return Err(e),
    };
    
    pb.set_style(style);
    Ok(pb)
}

/// Set progress bar message with section information
fn set_progress_message(
    pb: &ProgressBar,
    section_name: &str,
    section_index: usize,
    total_sections: usize,
) {
    let message = format!(
        "Section {}/{}: {}",
        section_index, total_sections, section_name
    );
    pb.set_message(message);
}

/// Finish progress bar with success message
pub fn finish_progress_success(
    pb: &ProgressBar,
    section_index: usize,
    total_sections: usize,
    lines_written: usize,
) {
    let message = format!(
        "\u{2705} Section {}/{} complete: {} lines",
        section_index, total_sections, lines_written
    );
    pb.finish_with_message(message);
}

/// Abandon progress bar with error message
pub fn abandon_progress_error(pb: &ProgressBar, error: &str) {
    let message = format!("\u{274c} Error writing section: {error}");
    pb.abandon_with_message(message);
}

/// Update progress bar position
pub fn update_progress_position(
    pb: &ProgressBar,
    current_line: usize,
    start_line: usize,
    lines_written: usize,
) {
    // Update every 1000 lines or at the end
    if lines_written % 1000 == 0 {
        let position = current_line.saturating_sub(start_line);
        pb.set_position(position.try_into().unwrap_or(u64::MAX));
    }
}