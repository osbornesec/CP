//! Binary detection and extraction functionality
//!
//! This module handles the extraction of sections with binary content detection,
//! providing detailed reporting of binary sections found during parsing.

use std::path::Path;

use crate::extraction::BinaryDetectionResult;
use crate::parser::utils;
use crate::validation::FileValidator;

/// Internal state for tracking section extraction with binary detection
struct ExtractionState {
    binary_sections_detected: usize,
    errors: Vec<String>,
    lines: Vec<String>,
    section_files: Vec<std::path::PathBuf>,
    sections_extracted: usize,
    warnings: Vec<String>,
}

impl ExtractionState {
    /// Create a new extraction state with the given lines
    #[inline]
    fn into_result(self, output_dir: &Path) -> BinaryDetectionResult {
        return BinaryDetectionResult {
            sections_extracted: self.sections_extracted,
            binary_sections_detected: self.binary_sections_detected,
            output_directory: output_dir.to_path_buf(),
            section_files: self.section_files,
            warnings: self.warnings,
            errors: self.errors,
        };
    }

    /// Initialize a new `ExtractionState` instance
    #[inline]
    #[allow(
        clippy::single_call_fn,
        reason = "Constructor function for state initialization"
    )]
    const fn new(lines: Vec<String>) -> Self {
        return Self {
            binary_sections_detected: 0,
            errors: Vec::new(),
            lines,
            section_files: Vec::new(),
            sections_extracted: 0,
            warnings: Vec::new(),
        };
    }
}

/// Extract sections with binary content detection
///
/// This function parses a cpinfo file, detects sections containing binary data,
/// and extracts all sections to individual files while reporting binary content.
///
/// # Arguments
///
/// * `input_path` - Path to the input cpinfo file
/// * `output_path` - Directory where extracted sections will be saved
///
/// # Returns
///
/// A `BinaryDetectionResult` containing extraction statistics and binary detection info
///
/// # Errors
///
/// Returns an error if file validation fails, file cannot be read, or sections cannot be saved
#[inline]
pub fn extract_sections_with_binary_detection<P1: AsRef<Path>, P2: AsRef<Path>>(
    input_path: P1,
    output_path: P2,
) -> crate::error::Result<BinaryDetectionResult> {
    let _validated = match FileValidator::validate_file(input_path.as_ref()) {
        Ok(validated) => validated,
        Err(error) => return Err(error),
    };
    let output_dir = output_path.as_ref();
    match std::fs::create_dir_all(output_dir) {
        Ok(()) => {}
        Err(error) => return Err(error.into()),
    }

    let extraction_state = match parse_file_with_binary_detection(&input_path) {
        Ok(state) => state,
        Err(error) => return Err(error),
    };
    let sections = match process_sections_with_binary_detection(extraction_state, output_dir) {
        Ok(sections) => sections,
        Err(error) => return Err(error),
    };

    return Ok(sections);
}

/// Parse file and prepare for binary detection processing
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Function is specialized for binary detection parsing"
)]
fn parse_file_with_binary_detection<P: AsRef<Path>>(
    input_path: P,
) -> crate::error::Result<ExtractionState> {
    let file_bytes = match std::fs::read(input_path.as_ref()) {
        Ok(bytes) => bytes,
        Err(error) => return Err(error.into()),
    };
    let file_content = String::from_utf8_lossy(&file_bytes);
    let lines: Vec<String> = file_content.lines().map(String::from).collect();

    return Ok(ExtractionState::new(lines));
}

/// Process sections with binary detection and save to files
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Function handles specific binary detection processing"
)]
#[allow(
    clippy::unnecessary_wraps,
    reason = "Result type needed for future error handling expansion"
)]
fn process_sections_with_binary_detection(
    mut state: ExtractionState,
    output_dir: &Path,
) -> crate::error::Result<BinaryDetectionResult> {
    const DELIMITER: &str = "==============================================";

    let mut current_section_name = String::new();
    let mut current_section_content = Vec::new();
    let mut in_section = false;
    let mut current_section_has_binary = false;

    // Skip header section
    let start_index = find_header_end(&state.lines);

    // Process sections
    for index in start_index..state.lines.len() {
        let line = match state.lines.get(index) {
            Some(line) => line,
            None => continue,
        };
        let line_trimmed = line.trim();

        if line_trimmed == DELIMITER {
            if in_section && !current_section_name.is_empty() {
                process_section_end(
                    &mut state,
                    &current_section_name,
                    &current_section_content,
                    current_section_has_binary,
                    output_dir,
                );

                current_section_content.clear();
                current_section_has_binary = false;
                in_section = false;
            } else if !current_section_name.is_empty() {
                in_section = true;
                current_section_content.clear();
                current_section_has_binary = false;
            }
        } else if !in_section && !line_trimmed.is_empty() {
            current_section_name = line_trimmed.to_owned();
        } else if in_section {
            if utils::contains_binary_data(line) {
                current_section_has_binary = true;
            }
            current_section_content.push(line.clone());
        }
    }

    // Process final section if exists
    if in_section && !current_section_name.is_empty() {
        process_section_end(
            &mut state,
            &current_section_name,
            &current_section_content,
            current_section_has_binary,
            output_dir,
        );
    }

    // Add summary warning if binary sections were detected
    if state.binary_sections_detected > 0 {
        state.warnings.push(format!(
            "Detected binary content in {} section(s)",
            state.binary_sections_detected
        ));
    }

    return Ok(state.into_result(output_dir));
}

/// Find the end of the header section
#[inline]
#[allow(clippy::single_call_fn, reason = "Specialized header parsing function")]
fn find_header_end(lines: &[String]) -> usize {
    let mut index = 0_usize;
    let mut found_header = false;

    while index < lines.len() {
        let current_line = match lines.get(index) {
            Some(line) => line,
            None => break,
        };

        if current_line.contains("Check Point Support Information") {
            found_header = true;
            index += 1_usize;
            continue;
        }

        let line_to_check = match lines.get(index) {
            Some(line) => line,
            None => break,
        };

        if found_header && line_to_check.trim() == "=============================================="
        {
            index += 1_usize;
            break;
        }
        index += 1_usize;
    }

    return index;
}

/// Process the end of a section - save it and update statistics
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Specialized section processing function"
)]
fn process_section_end(
    state: &mut ExtractionState,
    section_name: &str,
    section_content: &[String],
    has_binary: bool,
    output_dir: &Path,
) {
    if has_binary {
        state.binary_sections_detected += 1;
        state.warnings.push(format!(
            "Binary content detected in section '{section_name}'"
        ));
    }

    let content_str = section_content.join("\n");
    match utils::save_section(section_name, &content_str, output_dir) {
        Ok(()) => {
            state.sections_extracted += 1;
            let section_file = output_dir.join(format!("{}.txt", section_name.replace(' ', "_")));
            state.section_files.push(section_file);
        }
        Err(error) => {
            state
                .errors
                .push(format!("Failed to save section '{section_name}': {error}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_extract_sections_with_binary_detection() {
        let temp_input_dir = tempdir().unwrap();
        let temp_output_dir = tempdir().unwrap();

        let input_file = temp_input_dir.path().join("test.cpinfo");
        let test_content = "Check Point Support Information\n\n==============================================\nTest Section 1\n==============================================\nThis is normal content\nSome more normal text\n\n==============================================\nBinary Section\n==============================================\nThis has binary content\nSome normal text\n==============================================\n";

        fs::write(&input_file, test_content).unwrap();

        let result =
            extract_sections_with_binary_detection(&input_file, temp_output_dir.path()).unwrap();

        assert_eq!(result.sections_extracted, 2);
        assert_eq!(result.section_files.len(), 2);
        assert_eq!(result.output_directory, temp_output_dir.path());
    }

    #[test]
    fn test_parse_file_with_binary_detection() {
        let temp_dir = tempdir().unwrap();
        let input_file = temp_dir.path().join("test.cpinfo");

        fs::write(&input_file, "line1\nline2\nline3").unwrap();

        let state = parse_file_with_binary_detection(&input_file).unwrap();
        assert_eq!(state.lines.len(), 3);
        assert_eq!(state.lines[0], "line1");
    }

    #[test]
    fn test_find_header_end() {
        let lines = vec![
            "Some header".to_string(),
            "Check Point Support Information".to_string(),
            "More header".to_string(),
            "==============================================".to_string(),
            "First section".to_string(),
        ];

        let end_index = find_header_end(&lines);
        assert_eq!(end_index, 4);
    }
}
