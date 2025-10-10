use core::result::Result as StdResult;
use std::path::Path;
use walkdir::WalkDir;

use super::orchestrator::IntegratedWorkflowOrchestrator;
use crate::error::Result;
use crate::progress::ProgressReporter;
use std::fs::create_dir_all;

#[allow(
    clippy::multiple_inherent_impl,
    reason = "Phase implementations logically separated from core orchestrator"
)]
impl IntegratedWorkflowOrchestrator {
    /// Execute Phase 1: Extract sections from cpinfo file.
    ///
    /// # Arguments
    ///
    /// * `input_path` - Path to the input cpinfo file
    /// * `output_path` - Directory where extracted sections will be saved
    /// * `progress_reporter` - Optional progress reporting interface
    ///
    /// # Returns
    ///
    /// Organized extraction result containing section count and metadata
    ///
    /// # Errors
    ///
    /// Returns error if extraction fails or I/O operations fail
    #[inline]
    #[allow(clippy::unused_self, reason = "API consistency with instance methods")]
    pub(super) fn execute_phase_1<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        input_path: P1,
        output_path: P2,
        progress_reporter: &mut Option<&mut ProgressReporter>,
    ) -> Result<crate::extraction::OrganizedExtractionResult> {
        if let Some(ref mut progress) = *progress_reporter {
            progress.start(
                "\u{1f50d} Phase 1: Extracting sections from cpinfo file",
                None,
            );
        }

        let result = match crate::extraction::SectionExtractor::extract_sections_organized(
            input_path,
            output_path,
        ) {
            Ok(extraction_result) => extraction_result,
            Err(extraction_error) => return Err(extraction_error),
        };

        if let Some(ref mut progress) = *progress_reporter {
            progress.update(result.sections_extracted as u64);
            progress.set_operation(&format!(
                "Phase 1 complete: {} sections extracted",
                result.sections_extracted
            ));
        }

        return Ok(result);
    }

    /// Execute Phase 2: Parse extracted section files.
    ///
    /// # Arguments
    ///
    /// * `extracted_sections_dir` - Directory containing extracted section files
    /// * `progress_reporter` - Optional progress reporting interface
    ///
    /// # Returns
    ///
    /// Tuple of (`sections_processed`, `commands_extracted`, `files_extracted`)
    ///
    /// # Errors
    ///
    /// Returns error if parsing fails or I/O operations fail
    #[inline]
    #[allow(clippy::unused_async, reason = "Required by orchestrator API contract")]
    pub(super) async fn execute_phase_2(
        &self,
        extracted_sections_dir: &Path,
        progress_reporter: &mut Option<&mut ProgressReporter>,
    ) -> Result<(usize, usize, usize)> {
        if let Some(ref mut progress) = *progress_reporter {
            progress.start(
                "\u{2699}\u{fe0f} Phase 2: Parsing extracted section files",
                None,
            );
        }

        let parsing_result = self.parse_extracted_sections(extracted_sections_dir);

        if let Some(ref mut progress) = *progress_reporter {
            let (sections_processed, commands_extracted, files_extracted) = parsing_result;
            let total_outputs = commands_extracted + files_extracted;
            progress.update(total_outputs as u64);
            progress.set_operation(&format!(
                "Phase 2 complete: {sections_processed} sections processed, {commands_extracted} commands, {files_extracted} files extracted"
            ));
        }

        return Ok(parsing_result);
    }

    /// Parses extracted section text files under `extracted_sections_dir/sections` and writes
    /// extracted command sections into `extracted_sections_dir/commands` and file sections into
    /// `extracted_sections_dir/files`.
    ///
    /// Creates the `commands` and `files` directories if they do not exist. Non-existent
    /// `sections` directory or failures to prepare output directories cause the function to
    /// return (0, 0, 0). Individual section files that fail to parse are skipped; successfully
    /// parsed command and file sections are written to disk and counted.
    ///
    /// # Arguments
    ///
    /// * `extracted_sections_dir` - Root directory that contains a `sections` subdirectory with
    ///   section `.txt` files and where `commands` and `files` subdirectories will be created
    ///   (or used if already present).
    ///
    /// # Returns
    ///
    /// A tuple of `(sections_processed, total_commands, total_files)` where:
    /// - `sections_processed` is the number of section files that were successfully parsed,
    /// - `total_commands` is the number of command sections written to the `commands` dir,
    /// - `total_files` is the number of file sections written to the `files` dir.
    ///
    /// # Examples
    ///
    /// ```
    /// // Obtain an orchestrator instance appropriate for your context, then:
    /// // let orchestrator = IntegratedWorkflowOrchestrator::new(...);
    /// // let (sections, commands, files) = orchestrator.parse_extracted_sections(Path::new("output/extracted"));
    /// ```
    #[inline]
    #[allow(
    clippy::unused_self,
    reason = "May need access to configuration in future"
    )]
    fn parse_extracted_sections(&self, extracted_sections_dir: &Path) -> (usize, usize, usize) {
        use tracing::{info, warn};

        let sections_root = extracted_sections_dir.join("sections");
        if !sections_root.exists() {
            info!(
                "Phase 2: no sections directory found at {:?}; skipping parsing",
                sections_root
            );
            return (0, 0, 0);
        }

        let commands_dir = extracted_sections_dir.join("commands");
        if let Err(error) = create_dir_all(&commands_dir) {
            warn!(
                "Failed to prepare commands directory {:?}: {}",
                commands_dir, error
            );
            return (0, 0, 0);
        }

        let files_dir = extracted_sections_dir.join("files");
        if let Err(error) = create_dir_all(&files_dir) {
            warn!(
                "Failed to prepare files directory {:?}: {}",
                files_dir, error
            );
            return (0, 0, 0);
        }

        let mut sections_processed = 0;
        let mut total_commands = 0;
        let mut total_files = 0;

        let section_files: Vec<_> = WalkDir::new(&sections_root)
            .into_iter()
            .filter_map(StdResult::ok)
            .filter(|entry| {
                #[allow(
                    clippy::filetype_is_file,
                    reason = "Need to check both DirEntry::is_file and Path::is_file for robustness"
                )]
                return entry.file_type().is_file();
            })
            .filter(|entry| {
                let path_extension = entry.path().extension();
                let extension_str_option = path_extension.and_then(|extension| {
                    return extension.to_str();
                });
                let has_txt_extension = extension_str_option == Some("txt");
                return has_txt_extension;
            })
            .filter(|entry| {
                let entry_file_name = entry.path().file_name();
                let file_name_str = entry_file_name.and_then(|name| {
                    return name.to_str();
                });
                match file_name_str {
                    Some(filename) => {
                        return !filename.starts_with("cmd_") && !filename.starts_with("file_");
                    }
                    None => {
                        return false;
                    }
                }
            })
            .collect();

        let total_section_files = section_files.len();
        info!(
            "Phase 2: Found {} section files to process",
            total_section_files
        );

        for (index, entry) in section_files.into_iter().enumerate() {
            let section_file_path = entry.path();

            let path_file_name = section_file_path.file_name();
            let filename_option = path_file_name.and_then(|name| {
                return name.to_str();
            });
            if let Some(filename) = filename_option {
                info!(
                    "Processing section file {}/{}: {}",
                    index + 1,
                    total_section_files,
                    filename
                );
            }

            if let Ok(content) = std::fs::read_to_string(section_file_path) {
                let section_parser = crate::section_parser::SectionFileParser::new();
                match section_parser.parse_section_file(&content) {
                    Ok((command_sections, file_sections)) => {
                        for section in &command_sections {
                            let safe_filename =
                                crate::section_parser::sanitization::command_output_filename(
                                    &section.name,
                                );
                            let output_path = commands_dir.join(&safe_filename);

                            if std::fs::write(&output_path, &section.content).is_ok() {
                                total_commands += 1;
                            }
                        }

                        for section in &file_sections {
                            let safe_filename =
                                crate::section_parser::sanitization::file_output_filename(
                                    &section.path,
                                );
                            let output_path = files_dir.join(&safe_filename);

                            if std::fs::write(&output_path, &section.content).is_ok() {
                                total_files += 1;
                            }
                        }

                        sections_processed += 1;

                        if !command_sections.is_empty() || !file_sections.is_empty() {
                            info!(
                                "  \u{2192} Extracted {} commands and {} files",
                                command_sections.len(),
                                file_sections.len()
                            );
                        }
                    }
                    Err(_parsing_error) => {
                        info!("  \u{2192} Skipped (not a parseable section file)");
                    }
                }
            }
        }

        return (sections_processed, total_commands, total_files);
    }
}