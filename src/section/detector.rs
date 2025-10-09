//! Section delimiter detection functionality
//!
//! This module provides the `DelimiterDetector` struct and associated
//! functionality for detecting section delimiters in cpinfo files.

use crate::error::Result;
use crate::section::types::{SectionDelimiter, SectionValidation};
use crate::section::validation;
use std::path::Path;

/// Section delimiter detector
///
/// The `DelimiterDetector` provides functionality to detect and validate
/// section delimiters within cpinfo diagnostic files. It can identify
/// valid section boundaries and filter out formatting artifacts.
///
/// # Examples
///
/// ```ignore
/// use cpinfo_parser::section::DelimiterDetector;
///
/// let detector = DelimiterDetector::new();
/// let validation = detector.validate_section_name("System Information");
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DelimiterDetector;

impl DelimiterDetector {
    /// Detect delimiters with validation
    ///
    /// This method detects delimiters and validates associated section names.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to scan
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of valid `SectionDelimiter` instances
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or processed
    #[inline]
    pub fn detect_delimiters<P: AsRef<Path>>(path: P) -> Result<Vec<SectionDelimiter>> {
        let all_delimiters = match Self::identify_delimiters(path) {
            Ok(delimiters) => delimiters,
            Err(error) => return Err(error),
        };

        // Filter to only include delimiters with valid associated sections
        let valid_delimiters = all_delimiters
            .into_iter()
            .filter(|delimiter| {
                // Additional validation logic could go here
                return !delimiter.content.is_empty();
            })
            .collect();

        return Ok(valid_delimiters);
    }

    /// Find valid sections in a file
    ///
    /// This method scans a file and identifies all valid sections,
    /// returning their names and line numbers.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to scan
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of section names and line numbers
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or processed
    #[inline]
    pub fn find_valid_sections<P: AsRef<Path>>(path: P) -> Result<Vec<(String, usize)>> {
        let content = match std::fs::read_to_string(path) {
            Ok(file_content) => file_content,
            Err(io_error) => return Err(crate::error::CpinfoError::Io(io_error)),
        };
        let lines: Vec<&str> = content.lines().collect();
        let mut sections = Vec::new();

        for (line_index, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Look for potential delimiter lines
            if trimmed.len() > 10_usize
                && trimmed
                    .chars()
                    .all(|character| return "=-+*#_~^".contains(character))
            {
                // Check if the next line contains a valid section name
                if let Some(section_name) = Self::validate_strict_section_format(&lines, line_index)
                {
                    sections.push((section_name, line_index + 1_usize)); // +1 for 1-based line numbering
                }
            }
        }

        return Ok(sections);
    }

    /// Identify delimiters in a file
    ///
    /// This method identifies all potential delimiter lines in a file
    /// and returns their positions and content.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to scan
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `SectionDelimiter` instances
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read
    #[inline]
    pub fn identify_delimiters<P: AsRef<Path>>(path: P) -> Result<Vec<SectionDelimiter>> {
        let content = match std::fs::read_to_string(path) {
            Ok(file_content) => file_content,
            Err(io_error) => return Err(crate::error::CpinfoError::Io(io_error)),
        };
        let mut delimiters = Vec::new();

        for (line_number, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Identify lines that look like delimiters
            // Inline the potential delimiter check to avoid single-use function
            let is_delimiter = if trimmed.len() < 5_usize {
                false
            } else {
                // Check if line consists mainly of delimiter characters
                let delimiter_chars = "=-+*#_~^";
                let delimiter_count = trimmed
                    .chars()
                    .filter(|character| return delimiter_chars.contains(*character))
                    .count();
                let total_chars = trimmed.len();

                // Must be at least 80% delimiter characters
                // Use integer arithmetic to avoid floating point operations
                delimiter_count * 5_usize >= total_chars * 4_usize // 4/5 = 0.8
            };

            if is_delimiter {
                delimiters.push(SectionDelimiter::new(
                    line_number + 1_usize, // 1-based line numbering
                    trimmed.to_owned(),
                ));
            }
        }

        return Ok(delimiters);
    }

    /// Create a new delimiter detector
    ///
    /// # Returns
    ///
    /// A new `DelimiterDetector` instance
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        return Self;
    }

    /// Validate if a string is a legitimate section name
    ///
    /// This method provides a convenient interface to the validation system
    /// without debug output.
    ///
    /// # Arguments
    ///
    /// * `name` - The section name to validate
    ///
    /// # Returns
    ///
    /// A `SectionValidation` indicating if the name is valid
    #[must_use]
    #[inline]
    pub fn validate_section_name(name: &str) -> SectionValidation {
        return validation::validate_section_name_simple(name);
    }

    /// Validate section name with optional debug output
    ///
    /// This method provides access to the full validation system with
    /// optional debug information output.
    ///
    /// # Arguments
    ///
    /// * `name` - The section name to validate
    /// * `debug` - Whether to output debug information
    ///
    /// # Returns
    ///
    /// A `SectionValidation` indicating if the name is valid
    #[must_use]
    #[inline]
    pub fn validate_section_name_with_debug(name: &str, debug: bool) -> SectionValidation {
        return validation::validate_section_name(name, debug);
    }

    /// Validate strict section format for multi-line sections
    ///
    /// This method validates that a section follows the expected format
    /// with proper delimiter structure.
    ///
    /// # Arguments
    ///
    /// * `lines` - Array of lines to validate
    /// * `start_idx` - Starting index for validation
    ///
    /// # Returns
    ///
    /// `Some(String)` with section name if valid, `None` if invalid
    #[must_use]
    #[inline]
    pub fn validate_strict_section_format(lines: &[&str], start_idx: usize) -> Option<String> {
        if start_idx + 1_usize >= lines.len() {
            return None;
        }

        let potential_name = match lines.get(start_idx + 1_usize) {
            Some(line) => line.trim(),
            None => return None,
        };
        if Self::validate_section_name(potential_name).is_valid() {
            return Some(potential_name.to_owned());
        } else {
            return None;
        }
    }

    /// Legacy method for backward compatibility.
    ///
    /// This method maintains compatibility with existing code that expects
    /// the original `API` structure. It validates section format using strict
    /// parsing rules consistent with historical behavior.
    ///
    /// # Arguments
    ///
    /// * `lines` - Array of line strings to validate
    /// * `start_idx` - Starting index for validation within the lines array
    ///
    /// # Returns
    ///
    /// An `Option<String>` containing the validated section name if valid,
    /// or `None` if validation fails.
    #[must_use]
    #[inline]
    pub fn validate_strict_section_format_legacy(
        lines: &[&str],
        start_idx: usize,
    ) -> Option<String> {
        return Self::validate_strict_section_format(lines, start_idx);
    }
}

impl Default for DelimiterDetector {
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}
