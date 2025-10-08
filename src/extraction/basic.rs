//! Section extraction facade
//!
//! This module provides a unified interface for various section extraction methods,
//! delegating to specialized extraction modules while maintaining API compatibility.

use std::path::Path;

use crate::error::Result;
use crate::extraction::types::{ExtractionResult, OrganizedExtractionResult};
use crate::extraction::{basic_extraction, organized_extraction};

/// Section extractor facade
///
/// Provides a unified interface for section extraction operations,
/// delegating to specialized modules for different extraction strategies.
#[non_exhaustive]
pub struct SectionExtractor;

impl SectionExtractor {
    /// Extract sections from a cpinfo file to an output directory
    ///
    /// Provides basic section extraction with minimal processing.
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
    pub fn extract_sections<FirstPath: AsRef<Path>, SecondPath: AsRef<Path>>(
        input_path: FirstPath,
        output_path: SecondPath,
    ) -> Result<ExtractionResult> {
        return basic_extraction::extract_sections(input_path, output_path);
    }

    /// Extract sections with organized directory structure
    ///
    /// Provides advanced section extraction with categorized directory organization,
    /// progress reporting for large sections, and detailed extraction statistics.
    ///
    /// # Arguments
    ///
    /// * `input_path` - Path to the input cpinfo file
    /// * `output_path` - Base directory for organized extraction
    ///
    /// # Returns
    ///
    /// `OrganizedExtractionResult` with detailed extraction information
    ///
    /// # Errors
    ///
    /// Returns an error if file reading fails, validation fails, or sections cannot be written
    #[inline]
    pub fn extract_sections_organized<FirstPath: AsRef<Path>, SecondPath: AsRef<Path>>(
        input_path: FirstPath,
        output_path: SecondPath,
    ) -> Result<OrganizedExtractionResult> {
        return organized_extraction::extract_sections_organized(input_path, output_path);
    }

    /// Extract sections with VSX detection
    ///
    /// Provides specialized extraction with VSX (Virtual System Extension) detection.
    /// This delegates to the VSX extraction module for advanced Virtual System processing.
    ///
    /// # Arguments
    ///
    /// * `input_path` - Path to the input cpinfo file
    /// * `output_path` - Directory for VSX-aware extraction
    ///
    /// # Returns
    ///
    /// `OrganizedExtractionResult` with VSX detection results
    ///
    /// # Errors
    ///
    /// Returns an error if file reading fails, validation fails, or VSX detection encounters issues
    #[inline]
    pub fn extract_sections_with_vsx_detection<FirstPath: AsRef<Path>, SecondPath: AsRef<Path>>(
        input_path: FirstPath,
        output_path: SecondPath,
    ) -> Result<OrganizedExtractionResult> {
        // TODO: Implement VSX detection module
        // For now, delegate to organized extraction
        return organized_extraction::extract_sections_organized(input_path, output_path);
    }

    /// Create a new section extractor
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        return Self;
    }
}

impl Default for SectionExtractor {
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
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

        let content = "Check Point Support Information

==============================================
System Information
==============================================
System: Check Point Security Gateway
Version: R80.40
Build: 12345

==============================================
Network Configuration
==============================================
Interfaces: eth0, eth1
Routes: Default gateway configured
DNS: 8.8.8.8, 8.8.4.4

==============================================
";

        match fs::write(&file_path, content) {
            Ok(()) => {},
            Err(error) => panic!("Failed to write test file: {error}"),
        }
        (temp_dir, file_path)
    }

    #[test]
    fn test_section_extractor_creation() {
        let _extractor = SectionExtractor::new();
        let _default_extractor = SectionExtractor::default();

        // Both constructors should work without panicking
        // No meaningful comparison needed for unit structs
    }

    #[test]
    fn test_extract_sections() {
        let (_temp_input_dir, input_file) = create_test_cpinfo_file();
        let temp_output_dir = match tempdir() {
            Ok(directory) => directory,
            Err(error) => panic!("Failed to create temp output directory for test: {error}"),
        };

        let result = SectionExtractor::extract_sections(&input_file, temp_output_dir.path());
        assert!(result.is_ok());

        let extraction_result = match result {
            Ok(result_data) => result_data,
            Err(error) => panic!("Section extraction failed: {error}"),
        };
        assert_eq!(extraction_result.sections_extracted, 2);
        assert_eq!(extraction_result.section_files.len(), 2);
        assert_eq!(extraction_result.output_directory, temp_output_dir.path());
    }

    #[test]
    fn test_extract_sections_organized() {
        let (_temp_input_dir, input_file) = create_test_cpinfo_file();
        let temp_output_dir = match tempdir() {
            Ok(directory) => directory,
            Err(error) => panic!("Failed to create temp output directory for test: {error}"),
        };

        let result =
            SectionExtractor::extract_sections_organized(&input_file, temp_output_dir.path());
        assert!(result.is_ok());

        let organized_result = match result {
            Ok(result_data) => result_data,
            Err(error) => panic!("Organized extraction failed: {error}"),
        };
        assert_eq!(organized_result.sections_extracted, 2);
        assert_eq!(organized_result.section_files.len(), 2);
        assert_eq!(organized_result.output_directory, temp_output_dir.path());
        assert!(!organized_result.directories_created.is_empty());
    }

    #[test]
    fn test_extract_sections_with_vsx_detection() {
        let (_temp_input_dir, input_file) = create_test_cpinfo_file();
        let temp_output_dir = match tempdir() {
            Ok(directory) => directory,
            Err(error) => panic!("Failed to create temp output directory for test: {error}"),
        };

        let result = SectionExtractor::extract_sections_with_vsx_detection(
            &input_file,
            temp_output_dir.path(),
        );
        assert!(result.is_ok());

        let vsx_result = match result {
            Ok(result_data) => result_data,
            Err(error) => panic!("VSX extraction failed: {error}"),
        };
        assert_eq!(vsx_result.sections_extracted, 2);
        assert_eq!(vsx_result.section_files.len(), 2);
    }
}
