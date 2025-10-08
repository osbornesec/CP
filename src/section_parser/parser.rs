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
    /// Extract command section content
    #[inline]
    fn extract_command_section_content(
        &self,
        lines: &[&str],
        start_index: usize,
    ) -> Option<String> {
        if start_index + 2_usize >= lines.len() {
            return None;
        }

        let opening_line = match lines.get(start_index) {
            Some(line_content) => line_content,
            None => return None,
        };

        let opening_delimiter = match self.detector.detect_section_delimiter(opening_line) {
            Some(delimiter_type) => delimiter_type,
            None => return None,
        };

        let closing_line = match lines.get(start_index + 2_usize) {
            Some(line_content) => line_content,
            None => return None,
        };

        if let Some(closing_delimiter) = self.detector.detect_section_delimiter(closing_line) {
            if opening_delimiter == closing_delimiter {
                let content_start = start_index + 3_usize;
                let content_end = self.find_next_section_start(lines, content_start);

                let first_line = match lines.get(start_index) {
                    Some(line_content) => line_content,
                    None => return None,
                };
                let second_line = match lines.get(start_index + 1_usize) {
                    Some(line_content) => line_content,
                    None => return None,
                };
                let third_line = match lines.get(start_index + 2_usize) {
                    Some(line_content) => line_content,
                    None => return None,
                };

                let mut section_lines = vec![*first_line, *second_line, *third_line];

                for content_line in lines
                    .iter()
                    .take(content_end.min(lines.len()))
                    .skip(content_start)
                {
                    section_lines.push(*content_line);
                }

                return Some(section_lines.join("\n"));
            }
        }

        return None;
    }

    /// Extract file section content
    #[inline]
    fn extract_file_section_content(&self, lines: &[&str], start_index: usize) -> Option<String> {
        if start_index + 2_usize >= lines.len() {
            return None;
        }

        let opening_line = match lines.get(start_index) {
            Some(line_content) => line_content,
            None => return None,
        };

        if self.detector.detect_section_delimiter(opening_line)
            != Some(SectionDelimiterType::File66Dash)
        {
            return None;
        }

        let closing_line = match lines.get(start_index + 2_usize) {
            Some(line_content) => line_content,
            None => return None,
        };

        if self.detector.detect_section_delimiter(closing_line)
            == Some(SectionDelimiterType::File66Dash)
        {
            let content_start = start_index + 3_usize;
            let content_end = self.find_next_section_start(lines, content_start);

            let first_line = match lines.get(start_index) {
                Some(line_content) => line_content,
                None => return None,
            };
            let second_line = match lines.get(start_index + 1_usize) {
                Some(line_content) => line_content,
                None => return None,
            };
            let third_line = match lines.get(start_index + 2_usize) {
                Some(line_content) => line_content,
                None => return None,
            };

            let mut section_lines = vec![*first_line, *second_line, *third_line];

            for content_line in lines
                .iter()
                .take(content_end.min(lines.len()))
                .skip(content_start)
            {
                section_lines.push(*content_line);
            }

            return Some(section_lines.join("\n"));
        }

        return None;
    }

    /// Find the next section start index
    #[must_use]
    #[inline]
    fn find_next_section_start(&self, lines: &[&str], start_index: usize) -> usize {
        for (line_index, line_content) in lines.iter().enumerate().skip(start_index) {
            if self
                .detector
                .detect_section_delimiter(line_content)
                .is_some()
            {
                return line_index;
            }
        }
        return lines.len();
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

    /// Parse file section content
    ///
    /// # Errors
    /// Returns an error if file section format is invalid
    #[inline]
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
                message: "Expected file delimiter (66 dashes)".to_owned(),
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

    /// Parse entire section file containing multiple commands and files
    ///
    /// # Errors
    /// Returns an error if section parsing fails
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
            let current_line = match lines.get(line_index) {
                Some(line_content) => line_content,
                None => break,
            };
            if let Some(delimiter_type) = self.detector.detect_section_delimiter(current_line) {
                match delimiter_type {
                    SectionDelimiterType::Command24Dash | SectionDelimiterType::Command23Dash => {
                        let command_section_result =
                            self.extract_command_section_content(&lines, line_index);
                        if let Some(command_section_content) = command_section_result {
                            if let Ok(cmd_section) =
                                self.parse_command_section(&command_section_content)
                            {
                                command_sections.push(cmd_section);
                            }
                            line_index = self.find_next_section_start(&lines, line_index + 1_usize);
                        } else {
                            line_index += 1_usize;
                        }
                    }
                    SectionDelimiterType::File66Dash => {
                        let file_section_result =
                            self.extract_file_section_content(&lines, line_index);
                        if let Some(file_section_content) = file_section_result {
                            if let Ok(file_section) = self.parse_file_section(&file_section_content)
                            {
                                file_sections.push(file_section);
                            }
                            line_index = self.find_next_section_start(&lines, line_index + 1_usize);
                        } else {
                            line_index += 1_usize;
                        }
                    }
                }
            } else {
                line_index += 1_usize;
            }
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
