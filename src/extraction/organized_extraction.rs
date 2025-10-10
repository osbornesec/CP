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

/// Internal state used while tracking organized extraction progress.
struct OrganizedExtractionState {
    directories_created: Vec<PathBuf>,
    section_files: Vec<PathBuf>,
    writer_config: WriterConfig,
    sections_dir: PathBuf,
}

impl OrganizedExtractionState {
    /// Record a created section file path in the extraction state.
    ///
    /// Appends `file` to the internal list of extracted section file paths so it will
    /// be included in results and any post-processing.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::{Path, PathBuf};
    /// // Construct a state for the current directory; unwrap for brevity in the example.
    /// let mut state = OrganizedExtractionState::new(Path::new(".")).unwrap();
    /// let file = PathBuf::from("sections/example.txt");
    /// state.add_section_file(file.clone());
    /// assert!(state.section_files.contains(&file));
    /// ```
    fn add_section_file(&mut self, file: PathBuf) {
        self.section_files.push(file);
    }

    /// Creates a new OrganizedExtractionState rooted at the given base path.
    ///
    /// Ensures a "sections" subdirectory exists under `base_path` (creating it and recording it
    /// when necessary), initializes an empty list of section files and a default writer configuration,
    /// and returns the initialized state.
    ///
    /// # Parameters
    ///
    /// - `base_path`: Base directory under which the `sections` directory will be created.
    ///
    /// # Returns
    ///
    /// `Ok(Self)` containing an OrganizedExtractionState with `sections_dir` set to `base_path.join("sections")`
    /// and `directories_created` containing the `sections` directory if it was newly created; `Err` if
    /// creating the directory fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// // Create state rooted at "/tmp/output" (creates "/tmp/output/sections" if needed)
    /// let state = crate::organized::OrganizedExtractionState::new(Path::new("/tmp/output")).unwrap();
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

/// Orchestrates organized extraction of sections from `lines` and writes them under `output_path`.
///
/// This scans `lines` for section boundaries, extracts each detected section into the state's
/// sections directory, and accumulates paths of written section files in the returned state.
/// The function skips any file header before scanning and reports progress using the configured
/// writer. Initialization or write failures are returned as an error.
///
/// # Parameters
///
/// - `lines`: The source text split into line slices to be scanned for sections.
/// - `output_path`: Base output directory used to initialize the organized extraction state.
///
/// # Returns
///
/// An `OrganizedExtractionState` containing metadata and the list of created section files on
/// success, or an error if state initialization or writing a section fails.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # fn example() -> anyhow::Result<()> {
/// let text = "==============================================\nSection A\n==============================================\ncontent line\n==============================================\nSection B\n==============================================\nmore content\n";
/// let lines: Vec<&str> = text.lines().collect();
/// let state = process_organized_sections(&lines, Path::new(".")).unwrap();
/// assert!(state.section_files.len() >= 1);
/// # Ok(()) }
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

/// Locate the next section header and compute the start/end indices of its content.
///
/// Scans `lines` beginning at `start_index` for a section block delimited by a line
/// equal to `"=============================================="`, followed by a non-empty
/// section name line, and a closing delimiter. When found, returns the section name
/// and the inclusive content range as `(content_start, content_end)` where `content_end`
/// is the index of the delimiter that terminates the section or `lines.len()` if none.
///
/// # Returns
///
/// `Some((name, content_start, content_end))` when a valid section is found, `None` otherwise.
///
/// # Examples
///
/// ```
/// let lines: Vec<&str> = vec![
///     "header",
///     "==============================================",
///     "Section A",
///     "==============================================",
///     "line 1",
///     "line 2",
///     "==============================================",
///     "Section B",
///     "==============================================",
///     "b line 1",
/// ];
///
/// let found = find_next_section(&lines, 0).unwrap();
/// assert_eq!(found.0, "Section A");
/// assert_eq!(found.1, 4); // content starts after opening delimiter, name, and closing delimiter
/// // content_end points to the delimiter before "Section B"
/// assert_eq!(found.2, 6);
/// ```
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

/// Finds the index where the current section's content ends by locating the next section boundary.
///
/// A section boundary is identified when a delimiter line ("==============================================")
/// is followed by a non-empty section name line and then another delimiter line. Scanning begins
/// at `start_index`; the function returns the index of the opening delimiter that starts the next
/// section. If no valid boundary is found, returns `lines.len()`.
///
/// # Examples
///
/// ```
/// let lines = vec![
///     "header",
///     "content line 1",
///     "==============================================",
///     "Next Section",
///     "==============================================",
///     "more content",
/// ];
/// let idx = find_section_content_end(&lines.iter().map(|s| *s).collect::<Vec<&str>>(), 0);
/// assert_eq!(idx, 2); // index of the delimiter that starts "Next Section"
///
/// let lines_no_boundary = vec!["a", "b", "c"];
/// let idx2 = find_section_content_end(&lines_no_boundary.iter().map(|s| *s).collect::<Vec<&str>>(), 0);
/// assert_eq!(idx2, lines_no_boundary.len());
/// ```
#[allow(
    clippy::single_call_fn,
    reason = "Semantic clarity and code organization"
)]
#[inline]
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

/// Extracts a single section and writes it to a file inside `params.sections_dir`.
///
/// The section name is sanitized to form a filename (`{safe_name}.txt`) and the
/// section content defined by `params.content_start..params.content_end` is
/// written using the provided `writer_config`.
///
/// # Returns
///
/// `Some(PathBuf)` with the path to the written file if the section was written,
/// `None` if the write was skipped.
///
/// # Examples
///
/// ```
/// // Given constructed `params: OrganizedExtractionParams` and `writer_config: WriterConfig`
/// // let result = extract_organized_section(&params, &writer_config)?;
/// // match result {
/// //     Some(path) => println!("Wrote section to {:?}", path),
/// //     None => println!("Section was skipped"),
/// // }
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
