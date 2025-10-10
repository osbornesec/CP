//! Organized section extraction with categorized directory structure
//!
//! This module provides advanced section extraction that organizes sections
//! into categorized directories with progress reporting and detailed statistics.

use core::cmp;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use tracing::{debug, info, warn};

use crate::error::Result;
use crate::extraction::types::OrganizedExtractionResult;
use crate::extraction::writer::{
    sanitize_filename, write_section_with_progress, SectionWriteParams, WriterConfig,
};

/// Parameters for organized section extraction to reduce function argument count
struct OrganizedExtractionParams<'content> {
    /// Directory where sections are stored
    sections_dir: &'content Path,
    /// Ending line index for content
    content_end: usize,
    /// Starting line index for content
    content_start: usize,
    /// All lines from the file
    lines: &'content [&'content str],
    /// Section index for progress reporting
    section_index: usize,
    /// Name of the section
    section_name: &'content str,
    /// Total sections for progress reporting
    total_sections: usize,
}

/// State for tracking organized extraction progress
struct OrganizedExtractionState {
    directories_created: Vec<PathBuf>,
    section_files: Vec<PathBuf>,
    writer_config: WriterConfig,
    sections_dir: PathBuf,
}

impl OrganizedExtractionState {
    /// Appends a section file path to the accumulated list of generated section files.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::{Path, PathBuf};
    ///
    /// let base = Path::new("/tmp");
    /// let mut state = OrganizedExtractionState::new(base).unwrap();
    /// state.add_section_file(PathBuf::from("sections/sec1.txt"));
    /// assert!(state.section_files.iter().any(|p| p.ends_with("sec1.txt")));
    /// ```
    fn add_section_file(&mut self, file: PathBuf) {
        self.section_files.push(file);
    }

    /// Creates a new OrganizedExtractionState rooted at `base_path` by ensuring a
    /// `sections` directory exists and recording any directories created during
    /// initialization.
    ///
    /// The returned state contains:
    /// - `sections_dir`: the path `base_path/sections`
    /// - `directories_created`: a list containing `sections_dir` if it was created
    ///   by this call (empty otherwise)
    /// - default `writer_config` and an empty `section_files` list.
    ///
    /// # Parameters
    ///
    /// - `base_path`: base directory under which the `sections` directory is created.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// // Initialize state under the given base path (creates `base_path/sections` if needed)
    /// let state = OrganizedExtractionState::new(Path::new("output")).unwrap();
    /// assert!(state.sections_dir.ends_with("sections"));
    /// ```
    #[allow(
    clippy::single_call_fn,
    reason = "Semantic clarity and code organization"
    )]
    #[inline]
    fn new(base_path: &Path) -> Result<Self> {
        let sections_dir = base_path.join("sections");

        let mut directories_created = Vec::new();

        let sections_dir_created = !sections_dir.exists();
        if sections_dir_created {
            match create_dir_all(&sections_dir) {
                Ok(()) => {}
                Err(error) => return Err(error.into()),
            }
        }

        if sections_dir_created {
            directories_created.push(sections_dir.clone());
        }

        return Ok(Self {
            directories_created,
            section_files: Vec::new(),
            writer_config: WriterConfig::default(),
            sections_dir,
        });
    }
}

/// Extract sections with organized directory structure
///
/// This function provides advanced section extraction with:
/// - Categorized directory organization
/// - Progress reporting for large sections
/// - Detailed extraction statistics
///
/// # Arguments
///
/// * `input_path` - Path to the input cpinfo file
/// * `output_path` - Base directory for organized extraction
///
/// # Returns
///
/// `OrganizedExtractionResult` with detailed extraction information
///
/// # Errors
///
/// Returns an error if file reading fails, validation fails, or sections cannot be written
#[inline]
pub fn extract_sections_organized<P1: AsRef<Path>, P2: AsRef<Path>>(
    input_path: P1,
    output_path: P2,
) -> Result<OrganizedExtractionResult> {
    let input_path_ref = input_path.as_ref();
    let output_path_ref = output_path.as_ref();

    info!(
        "\u{1f680} Starting organized extraction from {:?}",
        input_path_ref
    );
    info!("\u{1f4c2} Output directory: {:?}", output_path_ref);

    match create_dir_all(output_path_ref) {
        Ok(()) => {}
        Err(create_error) => return Err(create_error.into()),
    }

    let file_content = match read_file_content(input_path_ref) {
        Ok(content) => content,
        Err(read_error) => return Err(read_error),
    };
    let lines: Vec<&str> = file_content.lines().collect();

    info!("\u{1f4ca} Total file lines: {}", lines.len());

    let extraction_state = match process_organized_sections(&lines, output_path_ref) {
        Ok(state) => state,
        Err(process_error) => return Err(process_error),
    };

    let extraction_params = crate::extraction::types::OrganizedExtractionParams {
        sections_extracted: extraction_state.section_files.len(),
        output_directory: output_path_ref.to_path_buf(),
        section_files: extraction_state.section_files,
        vsx_detected: false,      // VSX detection is handled in separate module
        virtual_systems_count: 0, // Virtual systems count for VSX
        directories_created: extraction_state.directories_created,
    };
    let result = OrganizedExtractionResult::new(extraction_params);

    info!(
        "\u{2705} Organized extraction complete: {} sections extracted",
        result.sections_extracted
    );

    return Ok(result);
}

/// Processes input lines to discover section blocks, write each section into the
/// "sections" directory under `output_path`, and returns the accumulated extraction state.
///
/// The function scans `lines` for section delimiters and names, extracts each section's
/// content, writes section files using the configured writer, and records created
/// directories and produced section file paths in the returned `OrganizedExtractionState`.
///
/// `lines` is the full file content split into lines. `output_path` is the base path
/// under which a `sections` directory will be created (or reused) to store extracted files.
///
/// # Returns
///
/// `OrganizedExtractionState` containing directories created, the path to the sections
/// directory, the writer configuration used, and the list of written section file paths.
///
/// # Examples
///
/// ```
/// use std::path::Path;
///
/// let lines = [
///     "-----",
///     "Section A",
///     "-----",
///     "Content line 1",
///     "Content line 2",
/// ];
///
/// // Writes into "./out/sections"
/// let state = process_organized_sections(&lines, Path::new("./out")).unwrap();
/// // State contains information about written files and created directories.
/// assert!(state.section_files.len() >= 0);
/// ```
#[allow(
clippy::single_call_fn,
reason = "Semantic clarity and code organization"
)]
#[inline]
fn process_organized_sections(
    lines: &[&str],
    output_path: &Path,
) -> Result<OrganizedExtractionState> {
    let mut state = OrganizedExtractionState::new(output_path)?;
    let total_sections = count_estimated_sections(lines);

    info!(
        "\u{1f4c8} Estimated sections to process: {}",
        total_sections
    );

    let mut section_index = 0;

    // Skip header
    let mut section_index_start = skip_file_header(lines);

    // Process sections
    while section_index_start < lines.len() {
        if let Some((section_name, content_start, content_end)) =
            find_next_section(lines, section_index_start)
        {
            section_index += 1;

            info!(
                "\u{1f4c4} Processing section {}/{}: {}",
                section_index, total_sections, section_name
            );

            let extraction_params = OrganizedExtractionParams {
                sections_dir: &state.sections_dir,
                content_end,
                content_start,
                lines,
                section_index,
                section_name: &section_name,
                total_sections,
            };

            let section_file_result =
                extract_organized_section(&extraction_params, &state.writer_config);

            match section_file_result {
                Ok(Some(section_file)) => {
                    state.add_section_file(section_file);
                }
                Ok(None) => {}
                Err(write_error) => return Err(write_error),
            }

            section_index_start = content_end + 1; // Move past this section
        } else {
            section_index_start += 1; // Move to next line if no section found
        }
    }

    return Ok(state);
}

/// Read file content with fallback for non-UTF8 files
#[allow(
    clippy::single_call_fn,
    reason = "Semantic clarity and code organization"
)]
#[inline]
fn read_file_content(path: &Path) -> Result<String> {
    if let Ok(content) = std::fs::read_to_string(path) {
        return Ok(content);
    }

    warn!("Failed to read as UTF-8, using lossy conversion");
    let file_bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(read_error) => return Err(read_error.into()),
    };
    return Ok(String::from_utf8_lossy(&file_bytes).into_owned());
}

/// Skip the file header to get to actual sections
#[allow(
    clippy::single_call_fn,
    reason = "Semantic clarity and code organization"
)]
#[inline]
fn skip_file_header(lines: &[&str]) -> usize {
    const DELIMITER: &str = "==============================================";

    for (i, line) in lines.iter().enumerate() {
        if line.contains("Check Point Support Information") {
            // Look for the first delimiter after the header
            for j in (i + 1)..lines.len() {
                if let Some(delimiter_line) = lines.get(j) {
                    if delimiter_line.trim() == DELIMITER {
                        return j + 1;
                    }
                }
            }
        }
    }

    return 0; // No header found, start from beginning
}

/// Locate the next section delimiter block and return the section name and content range.
///
/// Scans `lines` beginning at `start_index` for a delimiter line equal to
/// `"=============================================="` followed by a non-empty section name
/// and a matching closing delimiter. When found, returns `Some((section_name, content_start, content_end))`
/// where `content_start` is the index of the first line of the section content and
/// `content_end` is the index one past the last content line. If no valid section is found,
/// returns `None`.
///
/// # Examples
///
/// ```
/// let lines = vec![
///     "header",
///     "==============================================",
///     "Section A",
///     "==============================================",
///     "line1",
///     "line2",
///     "==============================================",
///     "Section B",
///     "==============================================",
///     "b1",
/// ];
/// let slice: Vec<&str> = lines.iter().map(|s| *s).collect();
/// let res = find_next_section(&slice, 0);
/// assert_eq!(res, Some(("Section A".to_string(), 4, 6)));
/// ```
#[allow(
clippy::single_call_fn,
reason = "Semantic clarity and code organization"
)]
fn find_next_section(lines: &[&str], start_index: usize) -> Option<(String, usize, usize)> {
    const DELIMITER: &str = "==============================================";

    for line_index in start_index..lines.len() {
        let current_line = match lines.get(line_index) {
            Some(line_content) => line_content.trim(),
            None => continue,
        };

        if current_line == DELIMITER && line_index + 2 < lines.len() {
            let section_name = match lines.get(line_index + 1) {
                Some(name) => name.trim().to_owned(),
                None => continue,
            };

            let closing_candidate = match lines.get(line_index + 2) {
                Some(line) => line.trim(),
                None => continue,
            };

            if section_name.is_empty()
                || section_name == DELIMITER
                || closing_candidate != DELIMITER
            {
                continue;
            }

            if line_index + 3 >= lines.len() {
                return Some((section_name, line_index + 3, lines.len()));
            }

            if !section_name.is_empty() && section_name != DELIMITER {
                // Find content boundaries
                let content_start = line_index + 3; // Skip delimiter, name, and closing delimiter
                let content_end = find_section_content_end(lines, content_start);

                return Some((section_name, content_start, content_end));
            }
        }
    }

    return None;
}

/// Locates the line index that marks the end of a section's content.
///
/// Scans forward from `start_index` to find a delimiter block of the form:
/// DELIMITER, non-empty section name line, DELIMITER. When such a block is found,
/// the function returns the index of the first delimiter line (the start of the
/// next section header). If no valid delimiter block is found, returns `lines.len()`.
///
/// # Parameters
///
/// - `lines`: Slice of lines to scan.
/// - `start_index`: Index to start scanning from.
///
/// # Returns
///
/// The index of the line that begins the next section delimiter block, or `lines.len()` if none is found.
///
/// # Examples
///
/// ```
/// let lines = [
///     "some content",
///     "more content",
///     "==============================================",
///     "Next Section",
///     "==============================================",
///     "following content",
/// ];
/// let end = find_section_content_end(&lines, 0);
/// assert_eq!(end, 2);
/// ```
fn find_section_content_end(lines: &[&str], start_index: usize) -> usize {
    const DELIMITER: &str = "==============================================";

    let mut index = start_index;
    while index + 2 < lines.len() {
        let current_line = match lines.get(index) {
            Some(line) => line.trim(),
            None => {
                index += 1;
                continue;
            }
        };

        if current_line != DELIMITER {
            index += 1;
            continue;
        }

        let potential_name = match lines.get(index + 1) {
            Some(line) => line.trim(),
            None => {
                index += 1;
                continue;
            }
        };

        if potential_name.is_empty() || potential_name == DELIMITER {
            index += 1;
            continue;
        }

        let closing_line = match lines.get(index + 2) {
            Some(line) => line.trim(),
            None => {
                index += 1;
                continue;
            }
        };

        if closing_line == DELIMITER {
            return index;
        }

        index += 1;
    }

    return lines.len();
}

/// Count estimated number of sections for progress reporting
#[allow(
    clippy::single_call_fn,
    reason = "Semantic clarity and code organization"
)]
#[inline]
fn count_estimated_sections(lines: &[&str]) -> usize {
    const DELIMITER: &str = "==============================================";
    return cmp::max(
        1,
        lines
            .iter()
            .filter(|line| return line.trim() == DELIMITER)
            .count()
            .saturating_sub(1),
    );
}

/// Writes the specified section to a sanitized file inside the sections directory.
///
/// The section content is written to a file named "<sanitized_section_name>.txt" located
/// in `params.sections_dir`. Returns the path to the written file when a file is produced,
/// or `None` when no file was created for the section.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
///
/// // Minimal example; types `OrganizedExtractionParams` and `WriterConfig` are assumed to be
/// // available in the current crate/module.
/// let lines = vec!["line 1".to_string(), "line 2".to_string()];
/// let params = OrganizedExtractionParams {
///     sections_dir: PathBuf::from("out/sections"),
///     content_start: 0,
///     content_end: 1,
///     lines: &lines,
///     section_index: 1,
///     section_name: "Example Section",
///     total_sections: 1,
/// };
/// let writer_cfg = WriterConfig::default();
/// let result = extract_organized_section(&params, &writer_cfg);
/// assert!(result.is_ok());
/// ```
#[allow(
clippy::single_call_fn,
reason = "Semantic clarity and code organization"
)]
#[inline]
fn extract_organized_section(
    params: &OrganizedExtractionParams,
    writer_config: &WriterConfig,
) -> Result<Option<PathBuf>> {
    let safe_name = sanitize_filename(params.section_name);
    let section_file = params.sections_dir.join(format!("{safe_name}.txt"));

    debug!("   \u{1f3af} Target file: {:?}", section_file);
    debug!(
        "   \u{1f4cf} Content range: lines {}-{}",
        params.content_start, params.content_end
    );

    let write_params = SectionWriteParams {
        section_name: params.section_name,
        lines: params.lines,
        content_start: params.content_start,
        content_end: params.content_end,
        output_file: &section_file,
        section_index: params.section_index,
        total_sections: params.total_sections,
    };

    return write_section_with_progress(&write_params, writer_config);
}

// Tests moved to tests/organized_extraction_tests.rs for cleaner code organization