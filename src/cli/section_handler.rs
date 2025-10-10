use super::args::Args;
use crate::progress::ProgressReporter;
use crate::{section_parser::sanitization, SectionFileParser};
use anyhow::Result;
use std::fs;
use tracing::info;

/// Parse a section file and extract sections to output directory
///
/// This is the main entry point that coordinates the CLI operation.
///
/// # Arguments
///
/// * `args` - Command line arguments containing input file and options
/// * `progress_reporter` - Optional progress reporter for user feedback
///
/// # Returns
///
/// Returns `Ok(())` on successful parsing/extraction, or `Err` with detailed context
///
/// # Errors
///
/// Returns an error if file validation, parsing, or command execution fails.
#[inline]
pub async fn parse_section_file(
    args: &Args,
    progress_reporter: Option<ProgressReporter>,
) -> Result<()> {
    info!("Parsing section file: {:?}", args.input);

    // Validate input file exists
    let file_content = match fs::read_to_string(&args.input) {
        Ok(content) => content,
        Err(io_error) => {
            return report_file_read_error_and_fail(args, &io_error);
        }
    };

    // Report processing information
    report_file_processing_info(&file_content, args);

    let section_parser = SectionFileParser::new();
    let mut progress_reporter_mutable = progress_reporter;

    // Execute appropriate operation based on read-only flag
    if args.read_only {
        return analyze_section_file_content(&section_parser, args, &mut progress_reporter_mutable)
            .await;
    } else {
        return extract_sections_to_output_content(
            &section_parser,
            args,
            &mut progress_reporter_mutable,
        )
        .await;
    }
}

/// Report file read error with recovery suggestions and return error
///
/// # Arguments
///
/// * `args` - Command line arguments for context
/// * `io_error` - The I/O error that occurred during file reading
///
/// # Returns
///
/// Always returns an `Err` with the formatted error message
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Function separates concerns and improves readability"
)]
fn report_file_read_error_and_fail(args: &Args, io_error: &std::io::Error) -> Result<()> {
    tracing::error!(
        "Error: Could not read section file '{}'",
        args.input.display()
    );
    tracing::error!("Suggestion: Check file path and permissions");
    tracing::error!(
        "  - Verify the file exists: ls -la {}",
        args.input.display()
    );
    tracing::error!("  - Check file permissions: file {}", args.input.display());
    tracing::error!("  - For section files, common locations are:");
    tracing::error!("    * results/misc/CP_Status.txt");
    tracing::error!("    * results/security/FireWall_Status.txt");
    tracing::error!("  - Original error: {io_error}");
    return Err(anyhow::anyhow!("Failed to read input file: {io_error}"));
}

/// Report file processing information including size and batch suggestions
///
/// # Arguments
///
/// * `file_content` - Content of the file for size analysis
/// * `args` - Command line arguments for configuration
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Function separates concerns and improves readability"
)]
fn report_file_processing_info(file_content: &str, args: &Args) {
    // Report file size if large
    let file_size = file_content.len();
    if file_size > 1_000_000 {
        let file_size_mb = file_size.checked_div(1_000_000_usize).unwrap_or_default();
        info!("Processing large section file ({} MB)", file_size_mb);
        if args.verbose {
            info!("Large file processing tips:");
            info!("  - Use --progress flag for progress updates");
            info!("  - Consider --read-only for analysis without extraction");
            info!("  - Use --commands-only or --files-only to reduce output");
        }
    }

    // Check for multiple section files
    if let Some(parent_directory) = args.input.parent() {
        if let Ok(directory_entries) = fs::read_dir(parent_directory) {
            let section_file_count = directory_entries
                .filter_map(core::result::Result::ok)
                .filter(|directory_entry| {
                    return directory_entry
                        .path()
                        .extension()
                        .and_then(|file_extension| return file_extension.to_str())
                        .is_some_and(|extension_str| {
                            return extension_str.to_lowercase() == "txt";
                        });
                })
                .count();

            if section_file_count > 1 && args.verbose {
                info!(
                    "Multiple section files detected in directory ({} files)",
                    section_file_count
                );
                info!("Consider batch processing with a script:");
                info!("  for file in *.txt; do");
                info!("    cpinfo-parser --section-file \"$$file\" -o \"output/$${{file%.txt}}/");
                info!("  done");
            }
        }
    }
}

/// Analyze section file without extraction (read-only mode)
///
/// # Arguments
///
/// * `section_parser` - Parser instance for processing section files
/// * `args` - Command line arguments containing configuration
/// * `progress_reporter` - Mutable reference to optional progress reporter
///
/// # Returns
///
/// Returns `Ok(())` on successful analysis, or `Err` with detailed context
///
/// # Errors
///
/// Returns an error if section file parsing fails or analysis cannot be completed
#[inline]
pub async fn analyze_section_file_content(
    section_parser: &SectionFileParser,
    args: &Args,
    progress_reporter: &mut Option<ProgressReporter>,
) -> Result<()> {
    if let Some(progress_reference) = progress_reporter.as_mut() {
        progress_reference.start("Analyzing section file", None);
    }

    match section_parser.process_section_file_async(&args.input).await {
        Ok(parse_result) => {
            let command_sections = &parse_result.command_sections;
            let file_sections = &parse_result.file_sections;
            let total_sections = command_sections.len() + file_sections.len();

            if let Some(progress_reporter_reference) = progress_reporter.as_mut() {
                progress_reporter_reference.update(u64::try_from(total_sections).unwrap_or(0_u64));
                progress_reporter_reference.finish(Some(&format!(
                    "Found {} command sections and {} file sections",
                    command_sections.len(),
                    file_sections.len()
                )));
            } else {
                report_analysis_details_to_console(command_sections, file_sections);
            }
            return Ok(());
        }
        Err(parse_error) => {
            if let Some(progress_reporter_reference) = progress_reporter.as_mut() {
                progress_reporter_reference.finish(Some("Analysis failed"));
            }
            return report_analysis_error_with_suggestions(&parse_error);
        }
    }
}

/// Extract sections to output directory
///
/// # Arguments
///
/// * `section_parser` - Parser instance for processing section files
/// * `args` - Command line arguments containing configuration and output directory
/// * `progress_reporter` - Mutable reference to optional progress reporter
///
/// # Returns
///
/// Returns `Ok(())` on successful extraction, or `Err` with detailed context
///
/// # Errors
///
/// Returns an error if directory creation, parsing, or file writing fails
#[inline]
pub async fn extract_sections_to_output_content(
    section_parser: &SectionFileParser,
    args: &Args,
    progress_reporter: &mut Option<ProgressReporter>,
) -> Result<()> {
    if let Some(progress_reference) = progress_reporter.as_mut() {
        progress_reference.start("Extracting sections to output directory", None);
    }

    match fs::create_dir_all(&args.output) {
        Ok(()) => {}
        Err(io_error) => {
            return Err(anyhow::anyhow!(
                "Failed to create output directory: {}",
                io_error
            ))
        }
    }

    match section_parser.process_section_file_async(&args.input).await {
        Ok(parse_result) => {
            let sections_written = match perform_section_extraction(&parse_result, args) {
                Ok(section_count) => section_count,
                Err(extraction_error) => return Err(extraction_error),
            };

            if let Some(progress_reporter_reference) = progress_reporter.as_mut() {
                progress_reporter_reference
                    .update(u64::try_from(sections_written).unwrap_or(0_u64));
                progress_reporter_reference.finish(Some(&format!(
                    "Successfully extracted {sections_written} sections"
                )));
            } else {
                info!(
                    "Successfully extracted {} sections to {:?}",
                    sections_written, args.output
                );
            }
            return Ok(());
        }
        Err(parse_error) => {
            if let Some(progress_reporter_reference) = progress_reporter.as_mut() {
                progress_reporter_reference.finish(Some("Extraction failed"));
            }
            return report_extraction_error_with_guidance(&parse_error, args);
        }
    }
}

/// Report analysis details to console with section information
///
/// # Arguments
///
/// * `command_sections` - List of command sections found
/// * `file_sections` - List of file sections found
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Function separates concerns and improves readability"
)]
fn report_analysis_details_to_console(
    command_sections: &[crate::section_parser::types::CommandSection],
    file_sections: &[crate::section_parser::types::FileSection],
) {
    info!("Successfully parsed section file");
    info!(
        "Found {} command sections and {} file sections",
        command_sections.len(),
        file_sections.len()
    );

    for (section_index, section) in command_sections.iter().enumerate() {
        info!(
            "Command {}: {name} ({content_len} bytes)",
            section_index + 1,
            name = section.name,
            content_len = section.content.len()
        );
    }

    for (section_index, section) in file_sections.iter().enumerate() {
        info!(
            "File {}: {} ({} bytes)",
            section_index + 1,
            section.path,
            section.content.len()
        );
    }
}

/// Report analysis error with recovery suggestions
///
/// # Arguments
///
/// * `parse_error` - The parsing error that occurred
///
/// # Returns
///
/// Always returns an `Err` with the formatted error message
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Function separates concerns and improves readability"
)]
fn report_analysis_error_with_suggestions(parse_error: &crate::error::CpinfoError) -> Result<()> {
    tracing::error!("Failed to parse section file: {parse_error}");
    tracing::error!("Recovery suggestions:");
    tracing::error!("  - Verify this is a valid Check Point section file");
    tracing::error!("  - Section files should contain command delimiters (24 or 23 dashes)");
    tracing::error!("  - Example format:");
    tracing::error!("    ------------------------");
    tracing::error!("    Command Name");
    tracing::error!("    ------------------------");
    tracing::error!("    Command output content...");
    tracing::error!("  - Try --verbose flag for detailed parsing information");
    tracing::error!("  - Use --read-only to analyze file structure without extraction");
    return Err(anyhow::anyhow!(
        "Section file analysis failed: {parse_error}"
    ));
}

/// Perform the actual section extraction to files
///
/// # Arguments
///
/// * `parse_result` - The parsed sections to extract
/// * `args` - Command line arguments for configuration
///
/// # Returns
///
/// Returns the number of sections successfully written
///
/// # Errors
///
/// Returns an error if any file write operation fails
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Function separates concerns and improves readability"
)]
fn perform_section_extraction(
    parse_result: &crate::section_parser::parser::SectionFileProcessResult,
    args: &Args,
) -> Result<usize> {
    let command_sections = &parse_result.command_sections;
    let file_sections = &parse_result.file_sections;
    let mut sections_written = 0;

    if !args.files_only {
        for section in command_sections {
            let safe_filename = sanitization::command_output_filename(&section.name);
            let output_path = args.output.join(&safe_filename);

            match fs::write(&output_path, &section.content) {
                Ok(()) => {}
                Err(io_error) => {
                    return Err(anyhow::anyhow!(
                        "Failed to write command section to file: {}",
                        io_error
                    ));
                }
            }
            sections_written += 1;
        }
    }

    if !args.commands_only {
        for section in file_sections {
            let safe_filename = sanitization::file_output_filename(&section.path);
            let output_path = args.output.join(&safe_filename);

            match fs::write(&output_path, &section.content) {
                Ok(()) => {}
                Err(io_error) => {
                    return Err(anyhow::anyhow!(
                        "Failed to write file section to file: {}",
                        io_error
                    ));
                }
            }

            info!(
                "Extracted file section: {} -> {:?}",
                section.path, output_path
            );
            sections_written += 1;
        }
    }

    return Ok(sections_written);
}

/// Report extraction error with troubleshooting guidance
///
/// # Arguments
///
/// * `parse_error` - The parsing error that occurred
/// * `args` - Command line arguments for context
///
/// # Returns
///
/// Always returns an `Err` with the formatted error message
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Function separates concerns and improves readability"
)]
fn report_extraction_error_with_guidance(
    parse_error: &crate::error::CpinfoError,
    args: &Args,
) -> Result<()> {
    tracing::error!("Failed to extract sections: {parse_error}");
    tracing::error!("Troubleshooting steps:");
    tracing::error!(
        "  - Check output directory permissions: {}",
        args.output.display()
    );
    tracing::error!("  - Ensure sufficient disk space for extraction");
    tracing::error!("  - Verify section file format with --read-only flag first");
    tracing::error!("  - Try --commands-only or --files-only to extract specific section types");
    return Err(anyhow::anyhow!("Section extraction failed: {parse_error}"));
}
