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

/// Extract content between start and end indices
///
/// # Arguments
///
/// * `lines` - All lines from the input file
/// * `start` - Starting index for content extraction
/// * `end` - Ending index for content extraction
///
/// # Returns
///
/// Extracted content as a string, with trailing empty lines removed
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_cpinfo_file() -> (tempfile::TempDir, std::path::PathBuf) {
        let temp_dir = match tempdir() {
            Ok(directory) => directory,
            Err(error) => panic!("Failed to create temp directory for test: {error}"),
        };
        let file_path = temp_dir.path().join("test.cpinfo");

        let content = "Check Point Support Information\n\n==============================================\nSystem Information\n==============================================\nSystem: Check Point Security Gateway\nVersion: R80.40\nBuild: 12345\n\n==============================================\nNetwork Configuration\n==============================================\nInterfaces: eth0, eth1\nRoutes: Default gateway configured\nDNS: 8.8.8.8, 8.8.4.4\n\n==============================================\nEmpty Section\n==============================================\n\n==============================================\n";

        match fs::write(&file_path, content) {
            Ok(()) => {},
            Err(error) => panic!("Failed to write test file: {error}"),
        }
        (temp_dir, file_path)
    }

    #[test]
    fn test_extract_sections() {
        let (_temp_input_dir, input_file) = create_test_cpinfo_file();
        let temp_output_dir = match tempdir() {
            Ok(directory) => directory,
            Err(error) => panic!("Failed to create temp output directory for test: {error}"),
        };

        let result = match extract_sections(&input_file, temp_output_dir.path()) {
            Ok(result_data) => result_data,
            Err(error) => panic!("Section extraction failed: {error}"),
        };

        assert_eq!(result.sections_extracted, 2); // Should extract 2 non-empty sections
        assert_eq!(result.section_files.len(), 2);
        assert_eq!(result.output_directory, temp_output_dir.path());

        // Check that files were created
        let system_file = temp_output_dir.path().join("System_Information.txt");
        let network_file = temp_output_dir.path().join("Network_Configuration.txt");

        assert!(system_file.exists());
        assert!(network_file.exists());

        // Verify content
        let system_content = match fs::read_to_string(&system_file) {
            Ok(content) => content,
            Err(error) => panic!("Failed to read system file content: {error}"),
        };
        assert!(system_content.contains("Check Point Security Gateway"));
        assert!(system_content.contains("Version: R80.40"));

        let network_content = match fs::read_to_string(&network_file) {
            Ok(content) => content,
            Err(error) => panic!("Failed to read network file content: {error}"),
        };
        assert!(network_content.contains("Interfaces: eth0, eth1"));
        assert!(network_content.contains("DNS: 8.8.8.8, 8.8.4.4"));
    }

    #[test]
    fn test_read_file_content() {
        let temp_dir = match tempdir() {
            Ok(directory) => directory,
            Err(error) => panic!("Failed to create temp directory for test: {error}"),
        };
        let file_path = temp_dir.path().join("test.txt");

        match fs::write(&file_path, "Test content\nLine 2") {
            Ok(()) => {},
            Err(error) => panic!("Failed to write test file: {error}"),
        }

        let content = match read_file_content(&file_path) {
            Ok(file_content) => file_content,
            Err(error) => panic!("Failed to read file content: {error}"),
        };
        assert_eq!(content, "Test content\nLine 2");
    }

    #[test]
    fn test_find_section_end() {
        let lines = vec![
            "Content line 1",
            "Content line 2",
            "==============================================",
            "Next section",
        ];

        let end = find_section_end(&lines, 0);
        assert_eq!(end, 2);

        let end_no_delimiter = find_section_end(&lines, 3);
        assert_eq!(end_no_delimiter, 4); // Should return lines.len()
    }

    #[test]
    fn test_extract_section_content() {
        let lines = vec!["Line 1", "Line 2", "", "Line 4", "", ""];

        let content = extract_section_content(&lines, 0, 6);
        assert_eq!(content, "Line 1\nLine 2\n\nLine 4");

        let empty_content = extract_section_content(&lines, 4, 6);
        assert_eq!(empty_content, "");
    }

    #[test]
    fn test_extract_sections_empty_file() {
        let temp_input_dir = match tempdir() {
            Ok(directory) => directory,
            Err(error) => panic!("Failed to create temp input directory for test: {error}"),
        };
        let temp_output_dir = match tempdir() {
            Ok(directory) => directory,
            Err(error) => panic!("Failed to create temp output directory for test: {error}"),
        };
        let input_file = temp_input_dir.path().join("empty.cpinfo");

        match fs::write(&input_file, "No valid sections here") {
            Ok(()) => {},
            Err(error) => panic!("Failed to write test file: {error}"),
        }

        let result = match extract_sections(&input_file, temp_output_dir.path()) {
            Ok(result_data) => result_data,
            Err(error) => panic!("Section extraction failed: {error}"),
        };

        assert_eq!(result.sections_extracted, 0);
        assert_eq!(result.section_files.len(), 0);
    }
}
