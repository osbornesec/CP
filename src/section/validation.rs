//! Section name validation module
//!
//! This module provides a comprehensive validation system for section names
//! found in cpinfo files. It uses a modular approach to validate different
//! aspects of section names and identify invalid patterns.

mod artifact_detection;
mod content_analysis;
mod pattern_detection;

use crate::section::SectionValidation;
use artifact_detection::check_formatting_artifacts;
use content_analysis::{analyze_punctuation_ratio, validate_meaningful_content};
use pattern_detection::{
    check_mixed_decorator_patterns, check_partial_delimiter_patterns,
    check_repeated_character_patterns, detect_table_formatting,
};

/// Validates a section name using all available validation checks
///
/// This function orchestrates all validation steps in a logical order,
/// from basic constraints to complex pattern detection. It follows the
/// fail-fast principle, returning as soon as any validation fails.
///
/// # Arguments
///
/// * `name` - The section name to validate (will be trimmed)
/// * `debug` - Whether to output debug information during validation
///
/// # Returns
///
/// A `SectionValidation` indicating whether the name is valid or invalid
/// with a descriptive error message.
///
/// # Examples
///
/// ```
/// use cpinfo_parser::section::validation::validate_section_name;
/// use cpinfo_parser::section::SectionValidation;
///
/// let result = validate_section_name("System Information", false);
/// assert!(matches!(result, SectionValidation::Valid));
///
/// let result = validate_section_name("========", false);
/// assert!(matches!(result, SectionValidation::Invalid(_)));
/// ```
#[must_use]
#[inline]
pub fn validate_section_name(name: &str, debug: bool) -> SectionValidation {
    let trimmed = name.trim();

    // Step 1: Basic constraint validation
    // Inlined validate_basic_constraints function content
    let trimmed_name = trimmed;

    if debug {
        // Debug output disabled due to restriction lints
    }

    // Empty or whitespace-only names are invalid
    if trimmed_name.is_empty() {
        if debug {
            // Debug output disabled due to restriction lints
        }
        return SectionValidation::Invalid("Empty section name".to_owned());
    }

    // Names that are too short (less than 3 characters) are likely artifacts
    if trimmed_name.len() < 3 {
        if debug {
            // Debug output disabled due to restriction lints
        }
        return SectionValidation::Invalid("Section name too short".to_owned());
    }

    // Names that are too long (over 255 chars) are likely malformed
    if trimmed_name.len() > 255 {
        if debug {
            // Debug output disabled due to restriction lints
        }
        return SectionValidation::Invalid("Section name too long".to_owned());
    }

    if debug {
        // Debug output disabled due to restriction lints
    }

    // Step 2: Table formatting detection (high priority filter)
    if let Some(result) = detect_table_formatting(trimmed, debug) {
        return result;
    }

    // Step 3: Punctuation ratio analysis
    if let Some(result) = analyze_punctuation_ratio(trimmed, debug) {
        return result;
    }

    // Step 4: Pattern-based validations
    if let Some(result) = check_repeated_character_patterns(trimmed, debug) {
        return result;
    }

    if let Some(result) = check_mixed_decorator_patterns(trimmed, debug) {
        return result;
    }

    // Step 5: Content validation
    if let Some(result) = validate_meaningful_content(trimmed, debug) {
        return result;
    }

    // Step 6: Delimiter and artifact detection
    if let Some(result) = check_partial_delimiter_patterns(trimmed, debug) {
        return result;
    }

    if let Some(result) = check_formatting_artifacts(trimmed, debug) {
        return result;
    }

    // All validations passed
    if debug {
        // Debug output disabled due to restriction lints
    }

    return SectionValidation::Valid;
}

/// Validates a section name without debug output
///
/// Convenience function for the most common validation use case.
///
/// # Arguments
///
/// * `name` - The section name to validate
///
/// # Returns
///
/// A `SectionValidation` indicating whether the name is valid or invalid.
#[must_use]
#[inline]
pub fn validate_section_name_simple(name: &str) -> SectionValidation {
    return validate_section_name(name, false);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_section_names() {
        let valid_names = [
            "System Information",
            "Network Configuration",
            "Security Settings",
            "Performance Metrics",
            "Log Analysis: Details",
            "Section 1: Overview",
        ];

        for name in &valid_names {
            let result = validate_section_name_simple(name);
            assert!(
                matches!(result, SectionValidation::Valid),
                "Failed for: {}",
                name
            );
        }
    }

    #[test]
    fn test_invalid_section_names() {
        let invalid_names = [
            "",                // Empty
            "ab",              // Too short
            "========",        // Repeated characters
            "| Col1 | Col2 |", // Table formatting
            "===---+++",       // Mixed decorator
            "!@#$%^&*()",      // No meaningful content
        ];

        for name in &invalid_names {
            let result = validate_section_name_simple(name);
            assert!(
                matches!(result, SectionValidation::Invalid(_)),
                "Should fail for: {}",
                name
            );
        }
    }

    #[test]
    fn test_validation_with_debug() {
        // This test ensures debug output doesn't change validation logic
        let test_name = "Valid Section Name";
        let result_no_debug = validate_section_name(test_name, false);
        let result_with_debug = validate_section_name(test_name, true);

        match (result_no_debug, result_with_debug) {
            (SectionValidation::Valid, SectionValidation::Valid) => (),
            (SectionValidation::Invalid(msg1), SectionValidation::Invalid(msg2)) => {
                assert_eq!(msg1, msg2);
            }
            _ => panic!("Debug flag should not change validation result"),
        }
    }
}
