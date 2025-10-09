//! File writing utilities for section extraction
//!
//! This module provides common utilities for writing extracted sections to files,
//! including progress reporting, content filtering, and file naming.

use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use indicatif::{ProgressBar, ProgressStyle};
use tracing::{debug, info};

use crate::error::Result;

/// State tracker for content writing operations
struct ContentWriteState {
    has_meaningful_content: bool,
    lines_written: usize,
    skip_leading_empty: bool,
    trailing_empty_lines: usize,
}

impl ContentWriteState {
    #[expect(
        clippy::single_call_fn,
        reason = "used only once but provides clear abstraction"
    )]
    #[inline]
    const fn new() -> Self {
        return Self {
            has_meaningful_content: false,
            lines_written: 0,
            skip_leading_empty: true,
            trailing_empty_lines: 0,
        };
    }
}

/// Parameters for section extraction to reduce function argument count
#[non_exhaustive]
pub struct SectionWriteParams<'content> {
    /// Ending line index for content  
    pub content_end: usize,
    /// Starting line index for content
    pub content_start: usize,
    /// All lines from the file
    pub lines: &'content [&'content str],
    /// Output file path
    pub output_file: &'content Path,
    /// Section index for progress reporting
    pub section_index: usize,
    /// Name of the section
    pub section_name: &'content str,
    /// Total sections for progress reporting
    pub total_sections: usize,
}

/// Configuration for section writing operations
#[non_exhaustive]
pub struct WriterConfig {
    /// Buffer size for file writes
    pub buffer_size: usize,
    /// Minimum lines to trigger progress reporting
    pub progress_threshold: usize,
    /// Whether to show progress bars for large sections
    pub show_progress: bool,
}

impl Default for WriterConfig {
    #[inline]
    fn default() -> Self {
        return Self {
            buffer_size: 64 * 1024,
            progress_threshold: 10_000,
            show_progress: true,
        };
    }
}

/// Create progress bar if section is large enough
#[expect(
    clippy::single_call_fn,
    reason = "used only once but provides clear abstraction"
)]
#[inline]
fn create_progress_bar_if_needed(
    estimated_lines: usize,
    section_name: &str,
    section_index: usize,
    total_sections: usize,
    config: &WriterConfig,
) -> Result<Option<ProgressBar>> {
    if !config.show_progress || estimated_lines < config.progress_threshold {
        return Ok(None);
    }

    info!(
        "   \\u{{1f6a8}} LARGE SECTION DETECTED: {} lines - enabling progress reporting",
        estimated_lines
    );

    let pb = ProgressBar::new(estimated_lines as u64);
    pb.set_style(
        match ProgressStyle::default_bar()
            .template("   {msg} [{bar:40.cyan/blue}] {pos}/{len} lines ({percent}%) ETA: {eta}")
        {
            Ok(value) => value,
            Err(error) => {
                return Err(crate::error::CpinfoError::validation_error(format!(
                    "Invalid progress bar template: {error}"
                )));
            }
        }
        .progress_chars("#>-"),
    );
    pb.set_message(format!(
        "Section {section_index}/{total_sections}: {section_name}"
    ));

    return Ok(Some(pb));
}

/// Handle the result of writing section content
#[expect(
    clippy::single_call_fn,
    reason = "used only once but provides clear abstraction"
)]
#[inline]
fn handle_write_result(
    result: Result<(usize, bool)>,
    mut writer: BufWriter<File>,
    output_file: &Path,
    progress_bar: Option<ProgressBar>,
    section_index: usize,
    total_sections: usize,
) -> Result<Option<PathBuf>> {
    match result {
        Ok((lines_written, has_content)) => {
            if let Some(pb) = progress_bar {
                pb.finish_with_message(format!(
                    "\\u{{2705}} Section {section_index}/{total_sections} complete: {lines_written} lines"
                ));
            }

            match writer.flush() {
                Ok(()) => {}
                Err(error) => return Err(error.into()),
            }
            drop(writer);

            if !has_content || lines_written == 0 {
                // Remove empty file - ignore errors since file might not exist
                #[allow(
                    clippy::let_underscore_must_use,
                    reason = "intentionally ignoring result for cleanup"
                )]
                let _: core::result::Result<(), std::io::Error> = std::fs::remove_file(output_file);
                info!("   \\u{{23ed}} SKIPPED: No meaningful content found (empty after trimming)");
                return Ok(None);
            } else {
                info!(
                    "   \\u{{2705}} EXTRACTION SUCCESSFUL: {} lines written to {:?}",
                    lines_written, output_file
                );
                return Ok(Some(output_file.to_path_buf()));
            }
        }
        Err(error) => {
            if let Some(pb) = progress_bar {
                pb.abandon_with_message(format!("\\u{{274c}} Error writing section: {error}"));
            }
            return Err(error);
        }
    }
}

/// Prepare buffered file writer
#[expect(
    clippy::single_call_fn,
    reason = "used only once but provides clear abstraction"
)]
#[inline]
fn prepare_file_writer(output_file: &Path, config: &WriterConfig) -> Result<BufWriter<File>> {
    debug!("   \\u{{1f4c4}} Creating file: {:?}", output_file);

    // Ensure parent directory exists
    if let Some(parent) = output_file.parent() {
        match create_dir_all(parent) {
            Ok(()) => {}
            Err(error) => return Err(error.into()),
        }
    }

    let file = match File::create(output_file) {
        Ok(file_handle) => file_handle,
        Err(error) => return Err(error.into()),
    };
    return Ok(BufWriter::with_capacity(config.buffer_size, file));
}

/// Process a single line of content
#[expect(
    clippy::single_call_fn,
    reason = "used only once but provides clear abstraction"
)]
#[inline]
fn process_line_content<W: Write>(
    writer: &mut W,
    state: &mut ContentWriteState,
    line_content: &str,
) -> Result<()> {
    if line_content.trim().is_empty() {
        state.trailing_empty_lines += 1;
    } else {
        match write_accumulated_empty_lines(writer, state) {
            Ok(()) => {}
            Err(error) => return Err(error),
        }
        match writeln!(writer, "{line_content}") {
            Ok(()) => {}
            Err(error) => return Err(error.into()),
        }
        state.lines_written += 1;
    }
    return Ok(());
}

/// Check if we should skip a leading empty line
#[expect(
    clippy::single_call_fn,
    reason = "used only once but provides clear abstraction"
)]
#[inline]
fn should_skip_leading_empty_line(state: &mut ContentWriteState, line_content: &str) -> bool {
    if state.skip_leading_empty && line_content.trim().is_empty() {
        return true;
    }
    state.skip_leading_empty = false;
    state.has_meaningful_content = true;
    return false;
}

/// Update progress bar if conditions are met
#[expect(
    clippy::single_call_fn,
    reason = "used only once but provides clear abstraction"
)]
#[inline]
fn update_progress_if_needed(
    progress_bar: Option<&ProgressBar>,
    state: &ContentWriteState,
    line_idx: usize,
    content_start: usize,
    content_end: usize,
) {
    if let Some(pb) = progress_bar {
        // Progress update every 1000 lines or at end
        if state.lines_written.wrapping_rem(1000) == 0 || line_idx >= content_end.saturating_sub(1)
        {
            pb.set_position((line_idx - content_start) as u64);
        }
    }
}

/// Write any accumulated trailing empty lines
#[expect(
    clippy::single_call_fn,
    reason = "used only once but provides clear abstraction"
)]
#[inline]
fn write_accumulated_empty_lines<W: Write>(
    writer: &mut W,
    state: &mut ContentWriteState,
) -> Result<()> {
    for _ in 0..state.trailing_empty_lines {
        match writeln!(writer) {
            Ok(()) => {}
            Err(error) => return Err(error.into()),
        }
    }
    state.trailing_empty_lines = 0;
    return Ok(());
}

/// Write section content to a buffered writer
///
/// Handles content filtering, empty line management, and progress updates.
///
/// # Returns
///
/// `(lines_written, has_meaningful_content)`
#[expect(
    clippy::single_call_fn,
    reason = "used only once but provides clear abstraction"
)]
#[inline]
fn write_section_content<W: Write>(
    writer: &mut W,
    lines: &[&str],
    content_start: usize,
    content_end: usize,
    progress_bar: Option<&ProgressBar>,
) -> Result<(usize, bool)> {
    let mut state = ContentWriteState::new();

    for line_idx in content_start..content_end {
        if line_idx >= lines.len() {
            break;
        }

        let line_content = match lines.get(line_idx) {
            Some(content) => content,
            None => break,
        };

        if should_skip_leading_empty_line(&mut state, line_content) {
            continue;
        }

        match process_line_content(writer, &mut state, line_content) {
            Ok(()) => {}
            Err(error) => return Err(error),
        }
        update_progress_if_needed(progress_bar, &state, line_idx, content_start, content_end);
    }

    return Ok((state.lines_written, state.has_meaningful_content));
}

/// Sanitize a section name to be a valid filename
///
/// Replaces problematic characters with underscores to ensure
/// the resulting filename is valid on all major filesystems.
#[inline]
#[must_use]
pub fn sanitize_filename(name: &str) -> String {
    return name
        .replace([' ', '/', '\\', ':'], "_")
        .replace(['<', '>', '"', '|', '?', '*'], "_");
}

/// Simple section writer without progress reporting
///
/// For smaller sections or when progress reporting is not needed.
///
/// # Errors
///
/// Returns error if file creation, writing, or flushing fails.
#[inline]
pub fn write_section_simple(content: &str, output_file: &Path) -> Result<Option<PathBuf>> {
    if content.trim().is_empty() {
        return Ok(None);
    }

    // Ensure parent directory exists
    if let Some(parent) = output_file.parent() {
        match create_dir_all(parent) {
            Ok(()) => {}
            Err(error) => return Err(error.into()),
        }
    }

    let mut file = match File::create(output_file) {
        Ok(file_handle) => file_handle,
        Err(error) => return Err(error.into()),
    };
    match write!(file, "{content}") {
        Ok(()) => {}
        Err(error) => return Err(error.into()),
    }
    match file.flush() {
        Ok(()) => {}
        Err(error) => return Err(error.into()),
    }

    return Ok(Some(output_file.to_path_buf()));
}

/// Writes a section to the given output file, optionally showing a progress bar for large sections.
///
/// The function writes lines in the range [content_start, content_end) from `params.lines` into
/// `params.output_file`. It may create and update a progress bar when the section is large,
/// ensures parent directories exist, and removes the output file if no meaningful content was written.
///
/// # Returns
///
/// `Ok(Some(PathBuf))` when the section was written and contains meaningful content, `Ok(None)` when
/// the section was skipped because it contained no meaningful content, or `Err(...)` if an error
/// occurred while creating the progress bar, preparing the file, writing content, or finalizing the result.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
///
/// // Construct a minimal SectionWriteParams; fields shown for illustration.
/// let lines: Vec<String> = vec!["line1".into(), "".into(), "line2".into()];
/// let params = SectionWriteParams {
///     content_start: 0,
///     content_end: lines.len(),
///     lines: &lines,
///     output_file: PathBuf::from("output.txt"),
///     section_index: 1,
///     total_sections: 1,
///     section_name: "example",
/// };
/// let config = WriterConfig::default();
///
/// // Call the writer (returns Result<Option<PathBuf>, _>)
/// let _ = write_section_with_progress(&params, &config);
/// ```
pub fn write_section_with_progress(
    params: &SectionWriteParams,
    config: &WriterConfig,
) -> Result<Option<PathBuf>> {
    let estimated_lines = params.content_end.saturating_sub(params.content_start);

    // Setup progress reporting for large sections
    let progress_bar = match create_progress_bar_if_needed(
        estimated_lines,
        params.section_name,
        params.section_index,
        params.total_sections,
        config,
    ) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };

    // Prepare file writer
    let mut writer = match prepare_file_writer(params.output_file, config) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };

    // Write content and handle result
    let result = write_section_content(
        &mut writer,
        params.lines,
        params.content_start,
        params.content_end,
        progress_bar.as_ref(),
    );

    return handle_write_result(
        result,
        writer,
        params.output_file,
        progress_bar,
        params.section_index,
        params.total_sections,
    );
}