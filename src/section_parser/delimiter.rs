use super::types::SectionDelimiterType;

/// Detector for section file delimiters
///
/// This detector analyzes line content to identify section delimiters
/// used in section files. It supports detection of command sections
/// (23-dash and 24-dash patterns) and file sections (66+ dash pattern).
///
/// # Performance
///
/// Detection is performed in O(1) time by checking line length first,
/// then validating character content only for matching lengths.
#[non_exhaustive]
pub struct SectionDelimiterDetector;

impl SectionDelimiterDetector {
    /// Determine the section delimiter type represented by a line.
    ///
    /// Recognizes three trimmed-line patterns: exactly 23 dashes (`Command23Dash`),
    /// exactly 24 dashes (`Command24Dash`), and 66 or more dashes (`File66Dash`).
    ///
    /// # Parameters
    ///
    /// * `input_line` - Line to analyze; leading and trailing whitespace are ignored.
    ///
    /// # Returns
    ///
    /// `Some(SectionDelimiterType::Command23Dash)`, `Some(SectionDelimiterType::Command24Dash)`, or
    /// `Some(SectionDelimiterType::File66Dash)` when a matching delimiter is found, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use cpinfo_parser::section_parser::delimiter::SectionDelimiterDetector;
    /// use cpinfo_parser::section_parser::types::SectionDelimiterType;
    ///
    /// let detector = SectionDelimiterDetector::new();
    /// assert_eq!(detector.detect_section_delimiter(&"-".repeat(23)), Some(SectionDelimiterType::Command23Dash));
    /// assert_eq!(detector.detect_section_delimiter(&"-".repeat(24)), Some(SectionDelimiterType::Command24Dash));
    /// assert_eq!(detector.detect_section_delimiter(&"-".repeat(66)), Some(SectionDelimiterType::File66Dash));
    /// assert_eq!(detector.detect_section_delimiter("not a delimiter"), None);
    /// ```
    #[inline]
    #[must_use]
    pub fn detect_section_delimiter(&self, input_line: &str) -> Option<SectionDelimiterType> {
        let trimmed_content = input_line.trim();

        let length = trimmed_content.len();

        match length {
            23 => {
                if Self::is_all_dashes(trimmed_content) {
                    Some(SectionDelimiterType::Command23Dash)
                } else {
                    None
                }
            }
            24 => {
                if Self::is_all_dashes(trimmed_content) {
                    Some(SectionDelimiterType::Command24Dash)
                } else {
                    None
                }
            }
            n if n >= 66 => {
                if Self::is_all_dashes(trimmed_content) {
                    Some(SectionDelimiterType::File66Dash)
                } else {
                    None
                }
            }
            _ => None,
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
