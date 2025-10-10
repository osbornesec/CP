use super::delimiter::SectionDelimiterDetector;
use super::types::{CommandSection, FileSection, SectionDelimiterType};
use crate::error::{CpinfoError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct SectionFileProcessingStats {
    pub total_bytes_processed: u64,
    pub total_commands_found: u32,
    pub total_files_found: u32,
    pub total_files_processed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SectionFileProcessResult {
    pub command_sections: Vec<CommandSection>,
    pub file_path: String,
    pub file_sections: Vec<FileSection>,
    pub stats: SectionFileProcessingStats,
}

/// Parser for section files containing multiple commands/files
#[non_exhaustive]
pub struct SectionFileParser {
    detector: SectionDelimiterDetector,
}

impl SectionFileParser {
    #[inline]
    fn extract_command_section_content(
        &self,
        lines: &[&str],
        start_index: usize,
    ) -> Option<String> {
        if !self.is_command_header(lines, start_index) {
            return None;
        }

        let content_start = start_index + 3_usize;
        let content_end = self.find_next_section_start(lines, content_start);

        let slice_end = content_end.min(lines.len());
        return Some(lines[start_index..slice_end].join("\n"));
    }

    /// Extract file section content
    #[inline]
    fn extract_file_section_content(&self, lines: &[&str], start_index: usize) -> Option<String> {
        if !self.is_file_header(lines, start_index) {
            return None;
        }

        let content_start = start_index + 3_usize;
        let content_end = self.find_next_section_start(lines, content_start);

        let slice_end = content_end.min(lines.len());
        return Some(lines[start_index..slice_end].join("\n"));
    }
    #[must_use]
    #[inline]
    fn find_next_section_start(&self, lines: &[&str], start_index: usize) -> usize {
        for line_index in start_index..lines.len() {
            if self.is_command_header(lines, line_index) || self.is_file_header(lines, line_index) {
                return line_index;
            }
        }
        return lines.len();
    }

    /// Determines whether the line at `index` begins a well-formed command section header.
    ///
    /// A well-formed command header consists of:
    /// - an opening command delimiter (either 23- or 24-dash variant) on the line at `index`,
    /// - a non-empty command name on the following line,
    /// - a closing delimiter on the third line that matches the opening delimiter.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let parser = SectionFileParser::new();
    /// let lines = [
    ///     "-----------------------", // opening command delimiter (23 or 24 dashes)
    ///     "my-command",
    ///     "-----------------------", // matching closing delimiter
    /// ];
    /// assert!(parser.is_command_header(&lines, 0));
    /// ```
    fn is_command_header(&self, lines: &[&str], index: usize) -> bool {
        if index + 2_usize >= lines.len() {
            return false;
        }

        let opening_delimiter = match self.detector.detect_section_delimiter(lines[index]) {
            Some(
                delimiter @ (SectionDelimiterType::Command23Dash
                | SectionDelimiterType::Command24Dash),
            ) => delimiter,
            _ => return false,
        };

        let command_name = lines[index + 1_usize].trim();
        if command_name.is_empty() {
            return false;
        }

        match self
            .detector
            .detect_section_delimiter(lines[index + 2_usize])
        {
            Some(delimiter) if delimiter == opening_delimiter => true,
            _ => false,
        }
    }

    /// Determines whether a file section header begins at `index` in `lines`.
    ///
    /// The check requires: an opening file delimiter of at least 66 dashes on the first line,
    /// a non-empty path on the second line, and a matching file delimiter on the third line.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let parser = SectionFileParser::new();
    /// let delim = "-".repeat(66);
    /// let lines: Vec<&str> = vec![&delim, "/some/path.txt", &delim];
    /// assert!(parser.is_file_header(&lines, 0));
    /// ```
    #[inline]
    fn is_file_header(&self, lines: &[&str], index: usize) -> bool {
        if index + 2_usize >= lines.len() {
            return false;
        }

        if self.detector.detect_section_delimiter(lines[index])
            != Some(SectionDelimiterType::File66Dash)
        {
            return false;
        }

        let path = lines[index + 1_usize].trim();
        if path.is_empty() {
            return false;
        }

        matches!(
            self.detector
                .detect_section_delimiter(lines[index + 2_usize]),
            Some(SectionDelimiterType::File66Dash)
        )
    }

    /// Create a new section file parser
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        return Self {
            detector: SectionDelimiterDetector::new(),
        };
    }

    /// Parse command section content
    ///
    /// # Errors
    /// Returns an error if command section format is invalid
    #[inline]
    pub fn parse_command_section(&self, section_content: &str) -> Result<CommandSection> {
        let lines: Vec<&str> = section_content.lines().collect();

        if lines.len() < 3_usize {
            return Err(CpinfoError::ParseError {
                message: "Insufficient lines for command section".to_owned(),
                line: 0_usize,
            });
        }

        let first_line = match lines.first() {
            Some(line_content) => line_content,
            None => {
                return Err(CpinfoError::ParseError {
                    message: "Missing first line".to_owned(),
                    line: 0_usize,
                });
            }
        };

        let opening_delimiter = match self
            .detector
            .detect_section_delimiter(first_line)
            .ok_or_else(|| {
                return CpinfoError::ParseError {
                    message: "Invalid opening delimiter".to_owned(),
                    line: 0_usize,
                };
            }) {
            Ok(value) => value,
            Err(error) => return Err(error),
        };

        match opening_delimiter {
            SectionDelimiterType::Command24Dash | SectionDelimiterType::Command23Dash => {} // These are valid command delimiters
            SectionDelimiterType::File66Dash => {
                return Err(CpinfoError::ParseError {
                    message: "Expected command delimiter".to_owned(),
                    line: 0_usize,
                });
            }
        }

        let second_line = match lines.get(1_usize) {
            Some(line_content) => line_content,
            None => {
                return Err(CpinfoError::ParseError {
                    message: "Missing command name line".to_owned(),
                    line: 1_usize,
                });
            }
        };

        let command_name = second_line.trim().to_owned();

        if command_name.is_empty() {
            return Err(CpinfoError::ParseError {
                message: "Empty command name".to_owned(),
                line: 1_usize,
            });
        }

        let third_line = match lines.get(2_usize) {
            Some(line_content) => line_content,
            None => {
                return Err(CpinfoError::ParseError {
                    message: "Missing closing delimiter line".to_owned(),
                    line: 2_usize,
                });
            }
        };

        let closing_delimiter = match self
            .detector
            .detect_section_delimiter(third_line)
            .ok_or_else(|| {
                return CpinfoError::ParseError {
                    message: "Invalid closing delimiter".to_owned(),
                    line: 2_usize,
                };
            }) {
            Ok(value) => value,
            Err(error) => return Err(error),
        };

        if opening_delimiter != closing_delimiter {
            return Err(CpinfoError::ParseError {
                message: "Mismatched delimiters".to_owned(),
                line: 2_usize,
            });
        }

        let content_lines = match lines.get(3_usize..) {
            Some(remaining_lines) => remaining_lines,
            None => &[],
        };
        let final_content = content_lines.join("\n");

        return Ok(CommandSection {
            name: command_name,
            content: final_content,
            delimiter_type: opening_delimiter,
        });
    }

    /// Parse a file section string into a `FileSection`.
    ///
    /// Validates that the section begins with a file delimiter (>= 66 dashes), contains a non-empty
    /// file path on the second line, and has a matching closing delimiter on the third line.
    /// On success returns a `FileSection` with the parsed path and the remaining lines joined as the
    /// file content.
    ///
    /// # Errors
    ///
    /// Returns a `CpinfoError::ParseError` when the section is malformed (for example: too few lines,
    /// missing or empty path, invalid opening/closing delimiter, or mismatched delimiters).
    ///
    /// # Examples
    ///
    /// ```
    /// let parser = SectionFileParser::new();
    /// let d = "-".repeat(66);
    /// let section = format!("{}\n/path/to/file.txt\n{}\nline1\nline2", d, d);
    /// let file = parser.parse_file_section(&section).unwrap();
    /// assert_eq!(file.path, "/path/to/file.txt");
    /// assert_eq!(file.content, "line1\nline2");
    /// ```
    pub fn parse_file_section(&self, section_content: &str) -> Result<FileSection> {
        let lines: Vec<&str> = section_content.lines().collect();

        if lines.len() < 3_usize {
            return Err(CpinfoError::ParseError {
                message: "Insufficient lines for file section".to_owned(),
                line: 0_usize,
            });
        }

        let first_line = match lines.first() {
            Some(line_content) => line_content,
            None => {
                return Err(CpinfoError::ParseError {
                    message: "Missing first line".to_owned(),
                    line: 0_usize,
                });
            }
        };

        let opening_delimiter = match self
            .detector
            .detect_section_delimiter(first_line)
            .ok_or_else(|| {
                return CpinfoError::ParseError {
                    message: "Invalid opening delimiter".to_owned(),
                    line: 0_usize,
                };
            }) {
            Ok(value) => value,
            Err(error) => return Err(error),
        };

        if opening_delimiter != SectionDelimiterType::File66Dash {
            return Err(CpinfoError::ParseError {
                message: "Expected file delimiter (>= 66 dashes)".to_owned(),
                line: 0_usize,
            });
        }

        let second_line = match lines.get(1_usize) {
            Some(line_content) => line_content,
            None => {
                return Err(CpinfoError::ParseError {
                    message: "Missing file path line".to_owned(),
                    line: 1_usize,
                });
            }
        };

        let file_path = second_line.trim().to_owned();

        if file_path.is_empty() {
            return Err(CpinfoError::ParseError {
                message: "Empty file path".to_owned(),
                line: 1_usize,
            });
        }

        let third_line = match lines.get(2_usize) {
            Some(line_content) => line_content,
            None => {
                return Err(CpinfoError::ParseError {
                    message: "Missing closing delimiter line".to_owned(),
                    line: 2_usize,
                });
            }
        };

        let closing_delimiter = match self
            .detector
            .detect_section_delimiter(third_line)
            .ok_or_else(|| {
                return CpinfoError::ParseError {
                    message: "Invalid closing delimiter".to_owned(),
                    line: 2_usize,
                };
            }) {
            Ok(value) => value,
            Err(error) => return Err(error),
        };

        if opening_delimiter != closing_delimiter {
            return Err(CpinfoError::ParseError {
                message: "Mismatched delimiters".to_owned(),
                line: 2_usize,
            });
        }

        let content_lines = match lines.get(3_usize..) {
            Some(remaining_lines) => remaining_lines,
            None => &[],
        };
        let file_content = content_lines.join("\n");

        return Ok(FileSection {
            path: file_path,
            content: file_content,
        });
    }

    /// Parse an entire section file into command and file sections.
    ///
    /// Processes the given file content, extracting zero or more command sections and file
    /// sections and returning them as two separate vectors.
    ///
    /// # Errors
    ///
    /// Returns an error if any detected section fails to parse.
    ///
    /// # Examples
    ///
    /// ```
    /// let parser = SectionFileParser::new();
    /// let (commands, files) = parser.parse_section_file("").unwrap();
    /// assert!(commands.is_empty() && files.is_empty());
    /// ```
    #[inline]
    pub fn parse_section_file(
        &self,
        file_content: &str,
    ) -> Result<(Vec<CommandSection>, Vec<FileSection>)> {
        let mut command_sections = Vec::new();
        let mut file_sections = Vec::new();
        let lines: Vec<&str> = file_content.lines().collect();

        let mut line_index = 0_usize;
        while line_index < lines.len() {
            if self.is_command_header(&lines, line_index) {
                if let Some(command_section_content) =
                    self.extract_command_section_content(&lines, line_index)
                {
                    if let Ok(cmd_section) = self.parse_command_section(&command_section_content) {
                        command_sections.push(cmd_section);
                    }
                }
                line_index = self.find_next_section_start(&lines, line_index + 3_usize);
                continue;
            }

            if self.is_file_header(&lines, line_index) {
                if let Some(file_section_content) =
                    self.extract_file_section_content(&lines, line_index)
                {
                    if let Ok(file_section) = self.parse_file_section(&file_section_content) {
                        file_sections.push(file_section);
                    }
                }
                line_index = self.find_next_section_start(&lines, line_index + 3_usize);
                continue;
            }

            line_index += 1_usize;
        }

        return Ok((command_sections, file_sections));
    }

    /// Process a single section file asynchronously.
    ///
    /// # Errors
    /// Returns an error if file reading or parsing fails
    #[inline]
    pub async fn process_section_file(&self, file_path: &Path) -> Result<SectionFileProcessResult> {
        let content = match fs::read_to_string(file_path).await.map_err(CpinfoError::Io) {
            Ok(file_content) => file_content,
            Err(error) => return Err(error),
        };
        let (command_sections, file_sections) = match self.parse_section_file(&content) {
            Ok(result) => result,
            Err(error) => return Err(error),
        };

        let stats = SectionFileProcessingStats {
            total_files_processed: 1_u32,
            total_commands_found: u32::try_from(command_sections.len()).unwrap_or_else(
                |_conversion_error| {
                    warn!(
                        "Too many command sections: {}, using u32::MAX",
                        command_sections.len()
                    );
                    return u32::MAX;
                },
            ),
            total_files_found: u32::try_from(file_sections.len()).unwrap_or_else(
                |_conversion_error| {
                    warn!(
                        "Too many file sections: {}, using u32::MAX",
                        file_sections.len()
                    );
                    return u32::MAX;
                },
            ),
            total_bytes_processed: u64::try_from(content.len()).unwrap_or_else(
                |_conversion_error| {
                    warn!("Content too large: {} bytes, using u64::MAX", content.len());
                    return u64::MAX;
                },
            ),
        };

        return Ok(SectionFileProcessResult {
            command_sections,
            file_path: file_path.to_string_lossy().into_owned(),
            file_sections,
            stats,
        });
    }
}

impl Default for SectionFileParser {
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}
