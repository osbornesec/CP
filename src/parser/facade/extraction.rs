//! Extraction operations facade
//!
//! This module contains all section extraction functionality including
//! basic extraction, organized extraction, and VSX detection.

use std::path::Path;

use crate::validation::FileValidator;

/// Extraction operations facade
#[non_exhaustive]
pub struct ExtractionFacade;

impl ExtractionFacade {
    /// Extract sections from a cpinfo file to an output directory
    ///
    /// Delegates to the extraction module for comprehensive section processing.
    ///
    /// # Arguments
    ///
    /// * `input_path` - Path to the input cpinfo file
    /// * `output_path` - Directory where sections will be extracted
    ///
    /// # Returns
    ///
    /// `ExtractionResult` with extraction statistics
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails or extraction encounters issues.
    #[inline]
    pub fn extract_sections<P1: AsRef<Path>, P2: AsRef<Path>>(
        input_path: P1,
        output_path: P2,
    ) -> crate::error::Result<crate::extraction::ExtractionResult> {
        use crate::extraction::SectionExtractor;

        let _validated = match FileValidator::validate_file(input_path.as_ref()) {
            Ok(validated_file) => validated_file,
            Err(validation_error) => return Err(validation_error),
        };
        return SectionExtractor::extract_sections(input_path, output_path);
    }

    /// Extract sections with organized directory structure
    ///
    /// Provides organized extraction with structured output directories.
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
    /// Returns an error if validation fails or organized extraction encounters issues.
    #[inline]
    pub fn extract_sections_organized<P1: AsRef<Path>, P2: AsRef<Path>>(
        input_path: P1,
        output_path: P2,
    ) -> crate::error::Result<crate::extraction::OrganizedExtractionResult> {
        use crate::extraction::SectionExtractor;

        let _validated = match FileValidator::validate_file(input_path.as_ref()) {
            Ok(validated_file) => validated_file,
            Err(validation_error) => return Err(validation_error),
        };
        return SectionExtractor::extract_sections_organized(input_path, output_path);
    }

    /// Extract sections with binary content detection
    ///
    /// Delegates to the `binary_extraction` module for specialized binary detection.
    ///
    /// # Arguments
    ///
    /// * `input_path` - Path to the input cpinfo file
    /// * `output_path` - Directory where sections will be extracted
    ///
    /// # Returns
    ///
    /// `BinaryDetectionResult` with binary detection statistics
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails or binary detection encounters issues.
    #[inline]
    pub fn extract_sections_with_binary_detection<P1: AsRef<Path>, P2: AsRef<Path>>(
        input_path: P1,
        output_path: P2,
    ) -> crate::error::Result<crate::extraction::BinaryDetectionResult> {
        use crate::parser::binary_extraction;
        return binary_extraction::extract_sections_with_binary_detection(input_path, output_path);
    }

    /// Extract sections with VSX detection
    ///
    /// Provides specialized extraction with VSX (Virtual System Extension) detection.
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
    /// Returns an error if validation fails or VSX detection encounters issues.
    #[inline]
    pub fn extract_sections_with_vsx_detection<P1: AsRef<Path>, P2: AsRef<Path>>(
        input_path: P1,
        output_path: P2,
    ) -> crate::error::Result<crate::extraction::OrganizedExtractionResult> {
        use crate::extraction::SectionExtractor;

        let _validated = match FileValidator::validate_file(input_path.as_ref()) {
            Ok(validated_file) => validated_file,
            Err(validation_error) => return Err(validation_error),
        };
        return SectionExtractor::extract_sections_with_vsx_detection(input_path, output_path);
    }
}
