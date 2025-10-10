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
    /// Retrieve the full text of a command section starting at the given line index.
    ///
    /// The returned string contains the header and all lines up to (but not including)
    /// the next section header, joined with newline characters. If the line at
    /// `start_index` is not a valid command header, this returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let parser = SectionFileParser::new();
    /// let lines = ["---", "my-cmd", "---", "echo hello", "echo world"];
    /// let content = parser
    ///     .extract_command_section_content(&lines, 0)
    ///     .unwrap();
    /// assert_eq!(content, "---\nmy-cmd\n---\necho hello\necho world");
    /// ```
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

        let mut section_lines = Vec::new();
        for line in lines
            .iter()
            .take(content_end.min(lines.len()))
            .skip(start_index)
        {
            section_lines.push(*line);
        }

        return Some(section_lines.join("\n"));
    }

    /// Retrieve the full text of a file section (header and its content) starting at the given line index.
    ///
    /// The returned string contains the contiguous lines from the header at `start_index` up to (but not
    /// including) the next recognized section header. If `start_index` does not point to a valid file
    /// header, `None` is returned.
    ///
    /// # Parameters
    ///
    /// - `lines`: slice of line string slices representing the file contents split by lines.
    /// - `start_index`: index into `lines` where a file header is expected to begin.
    ///
    /// # Returns
    ///
    /// `Some(String)` containing the joined lines of the file section (header + content) when a valid
    /// file header is found at `start_index`, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let parser = SectionFileParser::new();
    /// let lines = vec![
    ///     "------------------------------------------------------------------", // file header delimiter
    ///     "path/to/file.txt",
    ///     "------------------------------------------------------------------",
    ///     "file contents line 1",
    ///     "file contents line 2",
    /// ];
    /// let slice: Vec<&str> = lines.iter().map(|s| *s).collect();
    /// let section = parser.extract_file_section_content(&slice, 0);
    /// assert!(section.is_some());
    /// ```
    #[inline]
    fn extract_file_section_content(&self, lines: &[&str], start_index: usize) -> Option<String> {
        if !self.is_file_header(lines, start_index) {
            return None;
        }

        let content_start = start_index + 3_usize;
        let content_end = self.find_next_section_start(lines, content_start);

        let mut section_lines = Vec::new();
        for line in lines
            .iter()
            .take(content_end.min(lines.len()))
            .skip(start_index)
        {
            section_lines.push(*line);
        }

        return Some(section_lines.join("\n"));
    }

    /// Locate the index of the next section header (command or file) at or after `start_index`.
    ///
    /// # Returns
    ///
    /// The line index of the next command or file header, or `lines.len()` if none is found.
    ///
    /// # Examples
    ///
    /// ```
    /// let parser = SectionFileParser::new();
    /// let lines = [
    ///     "-------------------------", // command header opening (detector-dependent)
    ///     "do-something",
    ///     "-------------------------",
    ///     "some other line",
    /// ];
    /// let idx = parser.find_next_section_start(&lines, 0);
    /// assert_eq!(idx, 0);
    /// ```
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

    /// Determines whether the slice of lines beginning at `index` forms a valid command header.
    ///
    /// This checks that there are at least three lines starting at `index`, the first line is a command opening delimiter
    /// of type `Command23Dash` or `Command24Dash`, the second line contains a non-empty command name, and the third line
    /// is a matching closing delimiter equal to the opening delimiter.
    ///
    /// # Examples
    ///
    /// ```
    /// let parser = SectionFileParser::new();
    /// let lines = ["---", "do_something", "---", "rest of content"];
    /// assert!(parser.is_command_header(&lines, 0));
    /// ```
    #[inline]
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

    /// Detects whether the lines starting at `index` form a valid file section header.
    ///
    /// A valid file header is a three-line block where:
    /// - The first line is a file opening delimiter of type `File66Dash`,
    /// - The second line is a non-empty file path,
    /// - The third line is a matching `File66Dash` closing delimiter.
    ///
    /// This check requires at least three lines from `index` onward.
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

    /// Parse a file section block and produce a FileSection.
    ///
    /// Validates opening and closing file delimiters, extracts and trims the file path,
    /// and returns the remaining lines as the file content.
    ///
    /// # Errors
    ///
    /// Returns a `ParseError` when the section has fewer than three lines, when the opening
    /// or closing delimiter is missing or invalid, when the opening and closing delimiters
    /// do not match, or when the file path is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let parser = SectionFileParser::new();
    /// let section = "------------------------------------------------------------------\npath/to/file.txt\n------------------------------------------------------------------\nfile contents\n";
    /// let fs = parser.parse_file_section(section).unwrap();
    /// assert_eq!(fs.path, "path/to/file.txt");
    /// assert_eq!(fs.content, "file contents");
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

    /// Parse a section file into its command and file sections.
    ///
    /// Malformed or unrecognized section blocks are ignored; the method returns the collected
    /// command and file sections found in `file_content`.
    ///
    /// # Returns
    ///
    /// A tuple `(Vec<CommandSection>, Vec<FileSection>)` containing the parsed command sections
    /// and file sections respectively.
    ///
    /// # Examples
    ///
    /// ```
    /// let parser = SectionFileParser::new();
    /// let (cmds, files) = parser.parse_section_file("").unwrap();
    /// assert!(cmds.is_empty() && files.is_empty());
    /// ```
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