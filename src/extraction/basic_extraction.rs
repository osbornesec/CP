//! Basic section extraction functionality
//!
//! This module provides the core functionality for extracting sections from cpinfo files
//! with simple directory structure and basic content processing.

use std::fs;
use std::path::Path;

use crate::error::Result;
use crate::extraction::types::ExtractionResult;
use crate::extraction::writer::{sanitize_filename, write_section_simple};
use crate::section::DelimiterDetector;

/// Extract sections from a cpinfo file to an output directory
///
/// This function provides basic section extraction with minimal processing.
/// Each section is saved as a separate file in the output directory.
///
/// # Arguments
///
/// * `input_path` - Path to the input cpinfo file
/// * `output_path` - Directory where extracted sections will be saved
///
/// # Returns
///
/// `ExtractionResult` containing extraction statistics and file paths
///
/// # Errors
///
/// Returns an error if file reading fails or sections cannot be written
#[inline]
pub fn extract_sections<P1: AsRef<Path>, P2: AsRef<Path>>(
    input_path: P1,
    output_path: P2,
) -> Result<ExtractionResult> {
    let input_file_path = input_path.as_ref();
    let output_directory_path = output_path.as_ref();

    match fs::create_dir_all(output_directory_path) {
        Ok(()) => {}
        Err(error) => return Err(error.into()),
    }

    let valid_sections = match DelimiterDetector::find_valid_sections(input_file_path) {
        Ok(sections) => sections,
        Err(error) => return Err(error),
    };

    if valid_sections.is_empty() {
        return Ok(ExtractionResult::new(
            0,
            output_directory_path.to_path_buf(),
            Vec::new(),
        ));
    }

    let file_content = match read_file_content(input_file_path) {
        Ok(content) => content,
        Err(error) => return Err(error),
    };
    let lines: Vec<&str> = file_content.lines().collect();

    let section_files = match extract_valid_sections(&valid_sections, &lines, output_directory_path)
    {
        Ok(files) => files,
        Err(error) => return Err(error),
    };

    return Ok(ExtractionResult::new(
        section_files.len(),
        output_directory_path.to_path_buf(),
        section_files,
    ));
}

/// Read file content with fallback for non-UTF8 files
///
/// # Arguments
///
/// * `path` - Path to the file to read
///
/// # Returns
///
/// String content of the file, with lossy UTF-8 conversion if needed
///
/// # Errors
///
/// Returns an error if the file cannot be read
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Helper function for modular code organization"
)]
fn read_file_content(path: &Path) -> Result<String> {
    if let Ok(content) = fs::read_to_string(path) {
        return Ok(content);
    }

    // Fallback for files that might contain binary data
    let file_bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => return Err(error.into()),
    };
    return Ok(String::from_utf8_lossy(&file_bytes).into_owned());
}

/// Extract all valid sections to separate files
///
/// # Arguments
///
/// * `valid_sections` - List of section names and their starting line numbers
/// * `lines` - All lines from the input file
/// * `output_path` - Directory where sections will be saved
///
/// # Returns
///
/// Vector of paths to the created section files
///
/// # Errors
///
/// Returns an error if any section cannot be written
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Helper function for modular code organization"
)]
fn extract_valid_sections(
    valid_sections: &[(String, usize)],
    lines: &[&str],
    output_path: &Path,
) -> Result<Vec<std::path::PathBuf>> {
    let mut section_files = Vec::new();

    for section_tuple in valid_sections {
        let section_name = &section_tuple.0;
        let start_line = section_tuple.1;
        let section_file =
            match extract_single_section(section_name, start_line, lines, output_path) {
                Ok(file) => file,
                Err(error) => return Err(error),
            };
        if let Some(file) = section_file {
            section_files.push(file);
        }
    }

    return Ok(section_files);
}

/// Extract a single section to a file
///
/// # Arguments
///
/// * `section_name` - Name of the section to extract
/// * `start_line` - Line number where the section starts
/// * `lines` - All lines from the input file
/// * `output_path` - Directory where the section will be saved
///
/// # Returns
///
/// Optional path to the created file, None if section is empty
///
/// # Errors
///
/// Returns an error if the section file cannot be written
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Helper function for modular code organization"
)]
fn extract_single_section(
    section_name: &str,
    start_line: usize,
    lines: &[&str],
    output_path: &Path,
) -> Result<Option<std::path::PathBuf>> {
    let content_start = start_line + 2; // Skip delimiter and section name

    // Find the end of this section
    let content_end = find_section_end(lines, content_start);

    if content_start >= content_end || content_start >= lines.len() {
        return Ok(None);
    }

    // Extract section content
    let section_content = extract_section_content(lines, content_start, content_end);

    if section_content.trim().is_empty() {
        return Ok(None);
    }

    // Create output file
    let safe_name = sanitize_filename(section_name);
    let section_file = output_path.join(format!("{safe_name}.txt"));

    return write_section_simple(&section_content, &section_file);
}

/// Find the end of a section by looking for the next delimiter
///
/// # Arguments
///
/// * `lines` - All lines from the input file
/// * `start_index` - Index to start searching from
///
/// # Returns
///
/// Index of the next delimiter line, or end of file if no delimiter found
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Helper function for modular code organization"
)]
fn find_section_end(lines: &[&str], start_index: usize) -> usize {
    const DELIMITER: &str = "==============================================";

    for line_idx in start_index..lines.len() {
        let line = match lines.get(line_idx) {
            Some(line_content) => line_content,
            None => break,
        };
        if line.trim() == DELIMITER {
            return line_idx;
        }
    }

    return lines.len();
}

/// Extracts and returns the lines between `start` (inclusive) and `end` (exclusive) joined by `\n`, with trailing empty lines removed.
///
/// If `start >= end` or `start` is out of bounds for `lines`, an empty `String` is returned.
///
/// # Returns
///
/// `String` containing the extracted lines joined with `\n`; trailing empty lines are removed.
///
/// # Examples
///
/// ```rust,ignore
/// let lines = ["section line 1", "section line 2", "", ""];
/// let s = extract_section_content(&lines, 0, 4);
/// assert_eq!(s, "section line 1\nsection line 2");
/// ```
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Helper function for modular code organization"
)]
fn extract_section_content(lines: &[&str], start: usize, end: usize) -> String {
    if start >= end || start >= lines.len() {
        return String::new();
    }

    let content_slice = match lines.get(start..end) {
        Some(slice) => slice,
        None => return String::new(),
    };
    let content_lines: Vec<&str> = content_slice.to_vec();

    // Remove trailing empty lines
    let mut last_meaningful = 0;
    for (line_index, line) in content_lines.iter().enumerate() {
        if !line.trim().is_empty() {
            last_meaningful = line_index + 1;
        }
    }

    if last_meaningful == 0 {
        return String::new();
    }

    let final_slice = match content_lines.get(..last_meaningful) {
        Some(slice) => slice,
        None => return String::new(),
    };
    return final_slice.join("\n");
}
