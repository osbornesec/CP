//! Content processing utilities for section extraction
//!
//! This module provides low-level content processing functions for extracting
//! and cleaning section content from cpinfo files.

/// Find the end of a section by looking for the next delimiter
///
/// Searches through the lines starting from `start_index` to find the next
/// section delimiter. If no delimiter is found, returns the total number of lines.
///
/// # Arguments
///
/// * `lines` - The lines of the file as string slices
/// * `start_index` - The index to start searching from
///
/// # Returns
///
/// The index of the next delimiter line, or `lines.len()` if no delimiter found
pub fn find_section_end(lines: &[&str], start_index: usize) -> usize {
    const DELIMITER: &str = "==============================================";

    for line_idx in start_index..lines.len() {
        if lines[line_idx].trim() == DELIMITER {
            return line_idx;
        }
    }

    lines.len()
}

/// Extract content between start and end indices
///
/// Extracts the content lines between the specified indices and performs
/// cleanup by removing trailing empty lines.
///
/// # Arguments
///
/// * `lines` - The lines of the file as string slices
/// * `start` - Starting index (inclusive)
/// * `end` - Ending index (exclusive)
///
/// # Returns
///
/// The extracted content as a single string with lines joined by newlines.
/// Returns empty string if the range is invalid or contains no meaningful content.
pub fn extract_section_content(lines: &[&str], start: usize, end: usize) -> String {
    if start >= end || start >= lines.len() {
        return String::new();
    }

    let content_lines: Vec<&str> = lines[start..end].to_vec();

    // Remove trailing empty lines
    let last_meaningful = find_last_meaningful_line(&content_lines);

    if last_meaningful == 0 {
        return String::new();
    }

    content_lines[..last_meaningful].join("\n")
}

/// Find the index after the last non-empty line
///
/// Scans through content lines to find the last line with meaningful content,
/// ignoring whitespace-only lines at the end.
///
/// # Arguments
///
/// * `content_lines` - Lines to scan for meaningful content
///
/// # Returns
///
/// Index after the last meaningful line (0 if no meaningful content found)
fn find_last_meaningful_line(content_lines: &[&str]) -> usize {
    let mut last_meaningful = 0;
    for (i, line) in content_lines.iter().enumerate() {
        if !line.trim().is_empty() {
            last_meaningful = i + 1;
        }
    }
    last_meaningful
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_section_end() {
        let lines = vec![
            "Content line 1",
            "Content line 2",
            "==============================================",
            "Next section",
        ];

        let end = find_section_end(&lines, 0);
        assert_eq!(end, 2);

        let end_no_delimiter = find_section_end(&lines, 3);
        assert_eq!(end_no_delimiter, 4); // Should return lines.len()
    }

    #[test]
    fn test_extract_section_content() {
        let lines = vec!["Line 1", "Line 2", "", "Line 4", "", ""];

        let content = extract_section_content(&lines, 0, 6);
        assert_eq!(content, "Line 1\nLine 2\n\nLine 4");

        let empty_content = extract_section_content(&lines, 4, 6);
        assert_eq!(empty_content, "");
    }

    #[test]
    fn test_extract_section_content_invalid_range() {
        let lines = vec!["Line 1", "Line 2"];

        // Start >= end
        let content = extract_section_content(&lines, 1, 1);
        assert_eq!(content, "");

        // Start >= lines.len()
        let content = extract_section_content(&lines, 5, 10);
        assert_eq!(content, "");
    }

    #[test]
    fn test_find_last_meaningful_line() {
        let lines = vec!["Line 1", "Line 2", "", "Line 4", "", ""];
        assert_eq!(find_last_meaningful_line(&lines), 4);

        let empty_lines = vec!["", "  ", "\t"];
        assert_eq!(find_last_meaningful_line(&empty_lines), 0);

        let no_trailing_empty = vec!["Line 1", "Line 2"];
        assert_eq!(find_last_meaningful_line(&no_trailing_empty), 2);
    }
}