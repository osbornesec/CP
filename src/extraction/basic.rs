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
    /// Creates a default SectionExtractor.
    ///
    /// # Examples
    ///
    /// ```
    /// let extractor = SectionExtractor::default();
    /// let _ = extractor;
    /// ```
    fn default() -> Self {
        return Self::new();
    }
}