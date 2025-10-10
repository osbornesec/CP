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
    /// Detects whether a trimmed line matches a recognized section delimiter.
    ///
    /// The input line is trimmed of surrounding whitespace before testing. Returns
    /// `Some(SectionDelimiterType::Command23Dash)` for exactly 23 dashes,
    /// `Some(SectionDelimiterType::Command24Dash)` for exactly 24 dashes,
    /// `Some(SectionDelimiterType::File66Dash)` for 66 or more dashes, and `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let d23 = "-".repeat(23);
    /// assert_eq!(
    ///     SectionDelimiterDetector::new().detect_section_delimiter(&d23),
    ///     Some(SectionDelimiterType::Command23Dash)
    /// );
    ///
    /// let d24 = "-".repeat(24);
    /// assert_eq!(
    ///     SectionDelimiterDetector::new().detect_section_delimiter(&d24),
    ///     Some(SectionDelimiterType::Command24Dash)
    /// );
    ///
    /// let d66 = "-".repeat(66);
    /// assert_eq!(
    ///     SectionDelimiterDetector::new().detect_section_delimiter(&d66),
    ///     Some(SectionDelimiterType::File66Dash)
    /// );
    ///
    /// let none = "  not-a-delimiter  ";
    /// assert_eq!(
    ///     SectionDelimiterDetector::new().detect_section_delimiter(none),
    ///     None
    /// );
    /// ```
    #[inline]
    #[must_use]
    pub fn detect_section_delimiter(&self, input_line: &str) -> Option<SectionDelimiterType> {
        let trimmed_content = input_line.trim();

        let length = trimmed_content.len();

        if length == 23 && Self::is_all_dashes(trimmed_content) {
            return Some(SectionDelimiterType::Command23Dash);
        }

        if length == 24 && Self::is_all_dashes(trimmed_content) {
            return Some(SectionDelimiterType::Command24Dash);
        }

        if length >= 66 && Self::is_all_dashes(trimmed_content) {
            return Some(SectionDelimiterType::File66Dash);
        }

        return None;
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