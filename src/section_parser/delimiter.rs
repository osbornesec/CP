use super::types::SectionDelimiterType;

/// Detector for section file delimiters
///
/// This detector analyzes line content to identify section delimiters
/// used in section files. It supports detection of command sections
/// (23-dash and 24-dash patterns) and file sections (66-dash pattern).
///
/// # Performance
///
/// Detection is performed in O(1) time by checking line length first,
/// then validating character content only for matching lengths.
#[non_exhaustive]
pub struct SectionDelimiterDetector;

impl SectionDelimiterDetector {
    /// Detect section delimiter type from a line
    ///
    /// Analyzes the provided line to determine if it contains a recognized
    /// section delimiter pattern. Detection is based on line length and
    /// character content validation.
    ///
    /// # Arguments
    ///
    /// * `input_line` - The line content to analyze for delimiter patterns
    ///
    /// # Returns
    ///
    /// `Some(SectionDelimiterType)` if a valid delimiter is detected,
    /// `None` if no recognized pattern is found.
    #[inline]
    #[must_use]
    pub fn detect_section_delimiter(&self, input_line: &str) -> Option<SectionDelimiterType> {
        let trimmed_content = input_line.trim();

        match trimmed_content.len() {
            23 if Self::is_all_dashes(trimmed_content) => {
                return Some(SectionDelimiterType::Command23Dash);
            }
            24 if Self::is_all_dashes(trimmed_content) => {
                return Some(SectionDelimiterType::Command24Dash);
            }
            66 if Self::is_all_dashes(trimmed_content) => {
                return Some(SectionDelimiterType::File66Dash);
            }
            _ => {
                return None;
            }
        }
    }

    /// Helper: Check if string contains only dash characters
    ///
    /// Validates that the provided string consists entirely of dash characters
    /// and is not empty. Used for delimiter pattern validation.
    ///
    /// # Arguments
    ///
    /// * `input_string` - The string content to validate
    ///
    /// # Returns
    ///
    /// `true` if the string is non-empty and contains only dashes,
    /// `false` otherwise.
    fn is_all_dashes(input_string: &str) -> bool {
        let is_not_empty = !input_string.is_empty();
        let all_characters_are_dashes = input_string.chars().all(|character| {
            return character == '-';
        });
        return is_not_empty && all_characters_are_dashes;
    }

    /// Create a new section delimiter detector
    ///
    /// # Returns
    ///
    /// A new detector instance ready for delimiter detection operations.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        return Self;
    }
}

impl Default for SectionDelimiterDetector {
    /// Creates a default section delimiter detector
    ///
    /// # Returns
    ///
    /// A new detector instance using the standard configuration.
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}
