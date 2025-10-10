//! Pattern detection utilities for section name validation
//!
//! This module provides functions to detect various patterns in section names
//! that indicate invalid or decorative content.

use crate::section::SectionValidation;
use core::ops::Div as _;

/// Detects table formatting patterns
#[allow(
    clippy::single_call_fn,
    reason = "Pattern detection function used by validation system"
)]
#[inline]
pub fn detect_table_formatting(text: &str, debug: bool) -> Option<SectionValidation> {
    if is_table_formatting(text) {
        if debug {
            #[allow(clippy::print_stdout, reason = "Debug output for pattern detection")]
            {
                println!("\u{274c} NAME DEBUG: Table formatting detected");
            }
        }
        return Some(SectionValidation::Invalid(
            "Table formatting detected".to_owned(),
        ));
    }

    if debug {
        #[allow(clippy::print_stdout, reason = "Debug output for pattern detection")]
        {
            println!("\u{2705} NAME DEBUG: Table formatting check passed");
        }
    }
    return None;
}

/// Checks for repeated character patterns
#[allow(
    clippy::single_call_fn,
    reason = "Pattern detection function used by validation system"
)]
#[inline]
pub fn check_repeated_character_patterns(text: &str, debug: bool) -> Option<SectionValidation> {
    if is_repeated_character_line(text) {
        if debug {
            #[allow(clippy::print_stdout, reason = "Debug output for pattern detection")]
            {
                println!("\u{274c} NAME DEBUG: Repeated character pattern detected");
            }
        }
        return Some(SectionValidation::Invalid(
            "Repeated character pattern detected".to_owned(),
        ));
    }

    if debug {
        #[allow(clippy::print_stdout, reason = "Debug output for pattern detection")]
        {
            println!("\u{2705} NAME DEBUG: Repeated character pattern check passed");
        }
    }
    return None;
}

/// Checks for mixed decorator patterns
#[allow(
    clippy::single_call_fn,
    reason = "Pattern detection function used by validation system"
)]
#[inline]
pub fn check_mixed_decorator_patterns(text: &str, debug: bool) -> Option<SectionValidation> {
    if is_mixed_decorator_pattern(text) {
        if debug {
            #[allow(clippy::print_stdout, reason = "Debug output for pattern detection")]
            {
                println!("\u{274c} NAME DEBUG: Mixed decorator pattern detected");
            }
        }
        return Some(SectionValidation::Invalid(
            "Mixed decorator pattern detected".to_owned(),
        ));
    }

    if debug {
        #[allow(clippy::print_stdout, reason = "Debug output for pattern detection")]
        {
            println!("\u{2705} NAME DEBUG: Mixed decorator pattern check passed");
        }
    }
    return None;
}

/// Checks for partial delimiter patterns
#[allow(
    clippy::single_call_fn,
    reason = "Pattern detection function used by validation system"
)]
#[inline]
pub fn check_partial_delimiter_patterns(text: &str, debug: bool) -> Option<SectionValidation> {
    if contains_partial_delimiter(text) {
        if debug {
            #[allow(clippy::print_stdout, reason = "Debug output for pattern detection")]
            {
                println!("\u{274c} NAME DEBUG: Partial delimiter pattern detected");
            }
        }
        return Some(SectionValidation::Invalid(
            "Contains partial delimiter".to_owned(),
        ));
    }

    if debug {
        #[allow(clippy::print_stdout, reason = "Debug output for pattern detection")]
        {
            println!("\u{2705} NAME DEBUG: Partial delimiter pattern check passed");
        }
    }
    return None;
}

/// Checks if text contains table formatting patterns
#[allow(
    clippy::single_call_fn,
    reason = "Helper function for table formatting detection"
)]
#[inline]
fn is_table_formatting(text: &str) -> bool {
    // Check for pipe-separated columns
    if text.starts_with('|') && text.ends_with('|') && text.matches('|').count() >= 3 {
        return true;
    }

    // Check for box-drawing characters or table borders
    if text.contains("+-")
        || text.contains("-+")
        || text.contains("\u{2550}")
        || text.contains("\u{2551}")
    {
        return true;
    }

    // Check for consecutive pipe characters indicating column separators
    if text.matches('|').count() >= 2 {
        let mut part_count = 0;
        let all_parts_short = text.split('|').all(|section_part| {
            part_count += 1;
            return section_part.trim().len() < 20;
        });
        if part_count >= 3 && all_parts_short {
            return true;
        }
    }

    return false;
}

/// Checks if text is a repeated character line
#[allow(
    clippy::single_call_fn,
    reason = "Helper function for repeated character detection"
)]
#[inline]
fn is_repeated_character_line(text: &str) -> bool {
    if text.len() < 3 {
        return false;
    }

    let first_char = text.chars().next().unwrap_or('\0');

    // Check if at least 80% of characters are the same
    let same_char_count = text
        .chars()
        .filter(|&character| {
            return character == first_char;
        })
        .count();
    // Use integer math to avoid floating-point arithmetic
    // 80% threshold means 4/5, so multiply by 4 and compare with length * 4
    let threshold = text.len().div(5) * 4;

    return same_char_count >= threshold && "=-_*#~+".contains(first_char);
}

/// Checks if text is a mixed decorator pattern
#[allow(
    clippy::single_call_fn,
    reason = "Helper function for mixed decorator detection"
)]
#[inline]
fn is_mixed_decorator_pattern(text: &str) -> bool {
    let decorator_chars = "=-_*#~+|";
    let decorator_count = text
        .chars()
        .filter(|character| {
            return decorator_chars.contains(*character);
        })
        .count();
    let alphanumeric_count = text
        .chars()
        .filter(|character| {
            return character.is_alphanumeric();
        })
        .count();

    // Mixed decorator: mostly decorators with minimal alphanumeric content
    // Use integer division without the disallowed / operator
    let half_length = text.len().div(2);
    return decorator_count > alphanumeric_count && decorator_count >= half_length;
}

/// Detects a partial section delimiter sequence in the given text.
///
/// Returns `true` if the text contains the substring `"===="` but does not start with a full long delimiter `"===================="`, `false` otherwise.
///
/// # Examples
///
/// ```rust,ignore
/// assert!(contains_partial_delimiter("Title\n====\nContent"));
/// assert!(!contains_partial_delimiter("==================== full delimiter"));
/// ```
#[allow(
    clippy::single_call_fn,
    reason = "Helper function for partial delimiter detection"
)]
#[inline]
fn contains_partial_delimiter(text: &str) -> bool {
    // Check for incomplete section delimiters
    return text.contains("====") && !text.starts_with("====================");
}
