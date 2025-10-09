//! Core types for section handling
//!
//! This module defines the fundamental data structures used throughout
//! the section parsing and validation system.

/// Represents a section delimiter found in the cpinfo file
///
/// This structure represents a delimiter that separates sections within a cpinfo
/// diagnostic file. Delimiters are typically lines of equal signs or dashes that
/// mark the boundaries between different diagnostic sections.
///
/// # Delimiter Format
///
/// Common delimiter patterns in cpinfo files:
/// - `==============================================`
/// - `----------------------------------------------`
/// - Mixed patterns with section identifiers
///
/// # Examples
///
/// ```rust
/// use cpinfo_parser::section::SectionDelimiter;
///
/// let delimiter = SectionDelimiter::new(
///     42,
///     "==============================================".to_string()
/// );
///
/// assert_eq!(delimiter.line_number, 42);
/// assert!(delimiter.content.contains("="));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SectionDelimiter {
    /// The actual delimiter content (e.g., "==============================================")
    pub content: String,
    /// Line number where the delimiter was found (1-based indexing)
    pub line_number: usize,
}

impl SectionDelimiter {
    /// Create a new section delimiter
    ///
    /// # Arguments
    ///
    /// * `line_number` - The line number where the delimiter was found
    /// * `content` - The actual delimiter text content
    ///
    /// # Returns
    ///
    /// A new `SectionDelimiter` instance
    #[must_use]
    #[inline]
    pub const fn new(line_number: usize, content: String) -> Self {
        return Self {
            content,
            line_number,
        };
    }
}

/// Section validation result
///
/// Represents the outcome of validating a potential section name. This enum
/// provides clear feedback about whether a string is suitable as a section
/// name and, if not, explains why it was rejected.
///
/// # Variants
///
/// * `Valid` - The section name passed all validation checks
/// * `Invalid(String)` - The section name failed validation with a reason
///
/// # Examples
///
/// ```rust
/// use cpinfo_parser::section::SectionValidation;
///
/// let validation = SectionValidation::Valid;
/// assert!(matches!(validation, SectionValidation::Valid));
///
/// let validation = SectionValidation::Invalid("Too short".to_string());
/// if let SectionValidation::Invalid(reason) = validation {
///     println!("Validation failed: {}", reason);
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SectionValidation {
    /// The section name is invalid with the given reason
    Invalid(String),
    /// The section name is valid
    Valid,
}

impl SectionValidation {
    /// Get the error message if the validation failed
    ///
    /// # Returns
    ///
    /// `Some(String)` with the error message if invalid, `None` if valid
    #[must_use]
    #[inline]
    #[allow(
        clippy::pattern_type_mismatch,
        reason = "Pattern matching on enum variants in const context is safe here"
    )]
    pub const fn error_message(&self) -> Option<&String> {
        if let Self::Invalid(message) = self {
            return Some(message);
        }
        return None;
    }

    /// Check if the validation result indicates an invalid section name
    ///
    /// # Returns
    ///
    /// `true` if the validation result is `Invalid`, `false` otherwise
    #[must_use]
    #[inline]
    #[allow(
        clippy::pattern_type_mismatch,
        reason = "Pattern matching on enum variants in const context is safe here"
    )]
    pub const fn is_invalid(&self) -> bool {
        return matches!(self, Self::Invalid(_));
    }

    /// Check if the validation result indicates a valid section name
    ///
    /// # Returns
    ///
    /// `true` if the validation result is `Valid`, `false` otherwise
    #[must_use]
    #[inline]
    #[allow(
        clippy::pattern_type_mismatch,
        reason = "Pattern matching on enum variants in const context is safe here"
    )]
    pub const fn is_valid(&self) -> bool {
        return matches!(self, Self::Valid);
    }
}
