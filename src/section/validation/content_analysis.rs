//! Content analysis utilities for section name validation
//!
//! This module provides functions to analyze the content of section names,
//! including punctuation ratio analysis and meaningful content validation.

#![allow(
    clippy::single_call_fn,
    reason = "Public API functions may appear single-use during development"
)]

use crate::section::SectionValidation;

/// Analyzes the punctuation ratio in a section name
///
/// # Arguments
///
/// * `name` - The trimmed section name to analyze
/// * `debug` - Whether to output debug information
///
/// # Returns
///
/// * `Some(SectionValidation::Invalid)` if validation fails
/// * `None` if validation passes and should continue to next checks
#[must_use]
#[inline]
pub fn analyze_punctuation_ratio(name: &str, debug: bool) -> Option<SectionValidation> {
    let total_chars = name.len();
    let punctuation_count = name.chars().filter(char::is_ascii_punctuation).count();

    // Avoid floating point arithmetic by using integer comparison
    // Check if punctuation_count * 10 > total_chars * 7 (equivalent to ratio > 0.7)
    let high_punctuation_ratio = punctuation_count * 10 > total_chars * 7;

    if debug {
        // Debug information would be logged here in a real implementation
        // Removed eprintln! due to clippy restrictions
    }

    // If more than 70% of the string is punctuation, it's likely a decorator
    if high_punctuation_ratio {
        if debug {
            // Debug information would be logged here in a real implementation
            // Removed eprintln! due to clippy restrictions
        }
        return Some(SectionValidation::Invalid(
            "Too much punctuation (likely decorator)".to_owned(),
        ));
    }

    if debug {
        // Debug information would be logged here in a real implementation
        // Removed eprintln! due to clippy restrictions
    }
    return None;
}

/// Determines whether a trimmed section name contains sufficient alphanumeric content to be considered meaningful.
///
/// Returns `Some(SectionValidation::Invalid(_))` when the name contains no alphanumeric characters or when fewer than 30% of characters are alphanumeric; returns `None` when the name passes this check.
///
/// # Examples
///
/// ```
/// use crate::section::SectionValidation;
///
/// // no alphanumeric characters -> invalid
/// assert_eq!(
///     super::validate_meaningful_content("---!!!", false),
///     Some(SectionValidation::Invalid("No meaningful content".to_owned()))
/// );
///
/// // sufficient alphanumeric proportion -> valid (passes this check)
/// assert_eq!(
///     super::validate_meaningful_content("Title 123", false),
///     None
/// );
/// ```
#[must_use]
#[inline]
pub fn validate_meaningful_content(name: &str, debug: bool) -> Option<SectionValidation> {
    let alphanumeric_count = name
        .chars()
        .filter(|character| return character.is_alphanumeric())
        .count();
    let total_chars = name.len();

    if debug {
        // Debug information would be logged here in a real implementation
        // Removed eprintln! due to clippy restrictions
    }

    // If no alphanumeric characters, likely not meaningful
    if alphanumeric_count == 0 {
        if debug {
            // Debug information would be logged here in a real implementation
            // Removed eprintln! due to clippy restrictions
        }
        return Some(SectionValidation::Invalid(
            "No meaningful content".to_owned(),
        ));
    }

    // If less than 30% of characters are alphanumeric, likely not meaningful
    // Avoid floating point arithmetic by using integer comparison
    // Check if alphanumeric_count * 10 < total_chars * 3 (equivalent to ratio < 0.3)
    let low_meaningful_ratio = alphanumeric_count * 10 < total_chars * 3;
    if low_meaningful_ratio {
        if debug {
            // Debug information would be logged here in a real implementation
            // Removed eprintln! due to clippy restrictions
        }
        return Some(SectionValidation::Invalid(
            "Insufficient meaningful content".to_owned(),
        ));
    }

    if debug {
        // Debug information would be logged here in a real implementation
        // Removed eprintln! due to clippy restrictions
    }
    return None;
}