use core::time::Duration;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Types of section delimiters found in section files
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SectionDelimiterType {
    /// Command section delimiter with exactly 23 dashes: `-----------------------`
    Command23Dash,
    /// Command section delimiter with exactly 24 dashes: `------------------------`
    Command24Dash,
    /// File section delimiter with 66 or more dashes: `------------------------------------------------------------------`
    File66Dash,
}

/// Represents a parsed command section from a section file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CommandSection {
    /// Command output content after closing delimiter
    pub content: String,
    /// Type of delimiter used (23-dash or 24-dash)
    pub delimiter_type: SectionDelimiterType,
    /// Command name extracted from between delimiters
    pub name: String,
}

/// Represents a parsed file section from a section file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FileSection {
    /// File content after closing delimiter
    pub content: String,
    /// File path extracted from between delimiters
    pub path: String,
}

/// Result of parsing a section file containing both command and file sections
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SectionParseResult {
    /// Total bytes processed
    pub bytes_processed: u64,
    /// Parsed command sections
    pub command_sections: Vec<CommandSection>,
    /// Directories created during extraction
    pub directories_created: Vec<PathBuf>,
    /// Duration of the parsing operation
    pub duration: Duration,
    /// Parsed file sections
    pub file_sections: Vec<FileSection>,
    /// Output directory where sections were written
    pub output_directory: PathBuf,
    /// Phase 1 result for organized workflow
    pub phase_1_result: Option<Box<SectionParseResult>>,
    /// Total number of sections processed
    pub section_count: usize,
    /// Number of sections extracted
    pub sections_extracted: usize,
    /// Number of virtual systems detected
    pub virtual_systems_count: usize,
    /// Whether VSX virtual systems were detected
    pub vsx_detected: bool,
}

impl SectionParseResult {
    /// Creates a new parse result with the given sections
    #[inline]
    #[must_use]
    pub fn new(command_sections: Vec<CommandSection>, file_sections: Vec<FileSection>) -> Self {
        let command_sections_length = command_sections.len();
        let file_sections_length = file_sections.len();
        let total_sections = command_sections_length + file_sections_length;

        return Self {
            bytes_processed: 0,
            command_sections,
            directories_created: Vec::new(),
            duration: Duration::new(0, 0),
            file_sections,
            output_directory: PathBuf::new(),
            phase_1_result: None,
            section_count: total_sections,
            sections_extracted: total_sections,
            virtual_systems_count: 0,
            vsx_detected: false,
        };
    }
}
