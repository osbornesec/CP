//! Artifact detection utilities for section name validation
//!
//! This module provides functions to detect formatting artifacts,
//! encoding issues, and other text anomalies in section names.

use crate::section::SectionValidation;

/// Checks for formatting artifacts in the provided text string.
///
/// This is the main entry point for artifact detection in section validation.
///
/// # Arguments
///
/// * `text` - The text content to check for formatting artifacts
/// * `debug` - Whether to output debug information during validation
///
/// # Returns
///
/// Returns `Some(SectionValidation::Invalid)` if formatting artifacts are detected,
/// `None` if no artifacts are found.
#[inline]
#[must_use]
#[allow(
    clippy::single_match_else,
    clippy::single_call_fn,
    reason = "Main entry point function providing artifact detection API"
)]
pub fn check_formatting_artifacts(text: &str, debug: bool) -> Option<SectionValidation> {
    if contains_formatting_artifacts(text) {
        if debug {
            // Debug output suppressed to comply with restriction lints
            #[allow(
                clippy::let_unit_value,
                reason = "Debug flag must be acknowledged even when output is suppressed"
            )]
            let (): () = ();
        }
        return Some(SectionValidation::Invalid(
            "Contains formatting artifacts".to_owned(),
        ));
    }

    if debug {
        // Debug output suppressed to comply with restriction lints
        #[allow(
            clippy::let_unit_value,
            reason = "Debug flag must be acknowledged even when output is suppressed"
        )]
        let (): () = ();
    }
    return None;
}

/// Checks if text contains formatting artifacts.
///
/// This function is used internally by `check_formatting_artifacts` to detect
/// various types of text formatting issues.
///
/// # Arguments
///
/// * `text` - The text content to analyze for formatting artifacts
///
/// # Returns
///
/// Returns `true` if any formatting artifacts are detected, `false` otherwise.
#[inline]
#[allow(
    clippy::single_match_else,
    clippy::single_call_fn,
    reason = "Helper function provides logical separation of concerns"
)]
fn contains_formatting_artifacts(text: &str) -> bool {
    return has_control_characters(text)
        || has_excessive_whitespace(text)
        || has_html_like_artifacts(text)
        || has_encoding_artifacts(text);
}

/// Checks for control characters or unusual Unicode in the text.
///
/// This function is used by `contains_formatting_artifacts` to detect control characters.
///
/// # Arguments
///
/// * `text` - The text content to check for control characters
///
/// # Returns
///
/// Returns `true` if control characters (except tab) are found, `false` otherwise.
#[inline]
#[allow(
    clippy::single_match_else,
    clippy::single_call_fn,
    reason = "Specialized helper provides focused control character detection logic"
)]
fn has_control_characters(text: &str) -> bool {
    return text.chars().any(|character| {
        return character.is_control() && character != '\t';
    });
}

/// Checks for excessive whitespace patterns in the text.
///
/// This function is used by `contains_formatting_artifacts` to detect excessive whitespace.
///
/// # Arguments
///
/// * `text` - The text content to check for excessive whitespace
///
/// # Returns
///
/// Returns `true` if excessive whitespace patterns are detected, `false` otherwise.
#[inline]
#[allow(
    clippy::single_match_else,
    clippy::single_call_fn,
    reason = "Specialized helper provides focused whitespace pattern detection logic"
)]
fn has_excessive_whitespace(text: &str) -> bool {
    let trimmed_length = text.trim().len();
    let total_length = text.len();
    let has_triple_spaces = text.contains("   ");

    // Check if text has triple spaces and trimmed length is less than half of total
    return has_triple_spaces && (trimmed_length * 2) < total_length;
}

/// Checks for HTML-like artifacts in the text.
///
/// This function is used by `contains_formatting_artifacts` to detect HTML-like tags.
///
/// # Arguments
///
/// * `text` - The text content to check for HTML-like artifacts
///
/// # Returns
///
/// Returns `true` if HTML-like tags are detected, `false` otherwise.
#[inline]
#[allow(
    clippy::single_match_else,
    clippy::single_call_fn,
    reason = "Specialized helper provides focused HTML artifact detection logic"
)]
fn has_html_like_artifacts(text: &str) -> bool {
    return text.contains('<') && text.contains('>');
}

/// Detects common text encoding artifacts in a string.
///
/// This checks for visible signs of encoding problems, including the Unicode replacement
/// character U+FFFD and common mis-decoded byte sequences produced by UTF-8 ↔ Latin-1 errors
/// (for example the sequences "\u{e2}\u{20ac}\u{2122}" and "\u{c3}\u{a2}").
///
/// # Examples
///
/// ```rust,ignore
/// assert!(has_encoding_artifacts("\u{FFFD}"));
/// assert!(has_encoding_artifacts("\u{e2}\u{20ac}\u{2122}"));
/// assert!(!has_encoding_artifacts("Normal text"));
/// ```
#[inline]
#[allow(
    clippy::single_match_else,
    clippy::single_call_fn,
    reason = "Specialized helper provides focused encoding artifact detection logic"
)]
fn has_encoding_artifacts(text: &str) -> bool {
    // Check for common encoding artifacts
    let has_replacement_char = text.contains('\u{FFFD}'); // Unicode replacement character
    let has_utf8_latin1_error = text.contains("\u{e2}\u{20ac}\u{2122}"); // Common UTF-8 to Latin-1 encoding error
    let has_another_artifact = text.contains("\u{c3}\u{a2}"); // Another common encoding artifact

    return has_replacement_char || has_utf8_latin1_error || has_another_artifact;
}
