use super::args::Args;
use super::section_handler::parse_section_file;
use crate::progress::{AccessibilityConfig, AccessibilityMode, ProgressReporter};
use crate::{CpinfoParser, IntegratedWorkflowOrchestrator, VERSION};
use anyhow::Result;
use clap::Parser as _;
use tracing::{error, info};

/// Run the CLI application
///
/// # Errors
/// Returns an error if argument parsing fails, logging initialization fails, file operations fail,
/// or if any of the processing modes (read-only, extract-only, integrated workflow) encounter issues.
#[inline]
pub async fn run_cli() -> Result<()> {
    let args = Args::parse();

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(if args.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .with_ansi(false)
        .finish();
    match tracing::subscriber::set_global_default(subscriber) {
        Ok(()) => {}
        Err(subscriber_error) => {
            return Err(anyhow::anyhow!(
                "Failed to set global default subscriber: {}",
                subscriber_error
            ))
        }
    }

    info!("CPInfo Parser v{} starting", VERSION);
    info!("Input file: {:?}", args.input);
    info!("Output directory: {:?}", args.output);

    let parser = CpinfoParser::new();

    let progress_reporter = args.progress.then(|| {
        let mode = if args.verbose || std::env::var("SCREENREADER").is_ok() {
            AccessibilityMode::ScreenReader
        } else if std::env::var("NO_COLOR").is_ok()
            || std::env::var("TERM").unwrap_or_default() == "dumb"
        {
            AccessibilityMode::NoColor
        } else {
            AccessibilityMode::Standard
        };

        let config = AccessibilityConfig {
            mode,
            ..Default::default()
        };
        return ProgressReporter::with_config(config);
    });

    if args.section_file {
        match parse_section_file(&args, progress_reporter).await {
            Ok(()) => {}
            Err(parse_error) => return Err(parse_error),
        }
    } else if args.read_only {
        match handle_read_only_mode(&args, &parser, progress_reporter) {
            Ok(()) => {}
            Err(read_error) => return Err(read_error),
        }
    } else if args.extract_only {
        match handle_extract_only_mode(&args, &parser, progress_reporter) {
            Ok(()) => {}
            Err(extract_error) => return Err(extract_error),
        }
    } else {
        match handle_integrated_workflow(&args, progress_reporter).await {
            Ok(()) => {}
            Err(workflow_error) => return Err(workflow_error),
        }
    }

    return Ok(());
}

#[allow(
    clippy::single_call_fn,
    reason = "Semantic clarity and code organization"
)]
#[inline]
fn handle_read_only_mode(
    args: &Args,
    parser: &CpinfoParser,
    mut progress_reporter: Option<ProgressReporter>,
) -> Result<()> {
    if let Some(ref mut progress) = progress_reporter {
        progress.start("Parsing cpinfo file", None);
    }

    match parser.parse_file(&args.input) {
        Ok(result) => {
            if let Some(ref mut progress) = progress_reporter {
                progress.update(result.section_count as u64);
                progress.finish(Some(&format!(
                    "Successfully processed {} sections",
                    result.section_count
                )));
            } else {
                info!("Successfully processed {} sections", result.section_count);
                info!("Processing completed in {:?}", result.duration);
                info!("Bytes processed: {}", result.bytes_processed);

                if args.verbose {
                    info!("Next steps suggestions:");
                    info!("  - Extract individual section files for detailed analysis");
                    info!("  - Use --section-file flag to parse extracted section files");
                    info!("  - Example: cpinfo-parser --section-file output/misc/CP_Status.txt");
                }
            }
        }
        Err(parse_error) => {
            if let Some(ref mut progress) = progress_reporter {
                progress.finish(Some("Processing failed"));
            }
            error!("Failed to parse cpinfo file: {parse_error}");
            return Err(anyhow::anyhow!(
                "Failed to parse cpinfo file: {parse_error}"
            ));
        }
    }
    return Ok(());
}

#[allow(
    clippy::single_call_fn,
    reason = "Semantic clarity and code organization"
)]
#[inline]
fn handle_extract_only_mode(
    args: &Args,
    parser: &CpinfoParser,
    mut progress_reporter: Option<ProgressReporter>,
) -> Result<()> {
    if let Some(ref mut progress) = progress_reporter {
        progress.start("Extracting sections from cpinfo file", None);
    } else {
        info!("Starting section extraction (extract-only mode)...");
    }

    match parser.extract_sections_organized(&args.input, &args.output) {
        Ok(result) => {
            if let Some(ref mut progress) = progress_reporter {
                progress.update(result.sections_extracted as u64);
                progress.finish(Some(&format!(
                    "Extracted {} sections",
                    result.sections_extracted
                )));
            } else {
                info!("\u{2705} Section extraction completed successfully!");
                info!(
                    "Extracted {} sections to {:?}",
                    result.sections_extracted, result.output_directory
                );

                if result.vsx_detected {
                    info!(
                        "\u{1f527} VSX detected with {} virtual systems",
                        result.virtual_systems_count
                    );
                }

                info!("\u{1f4c2} Created directories:");
                for dir in &result.directories_created {
                    info!("  - {:?}", dir);
                }

                if args.verbose {
                    info!("Next steps suggestions:");
                    info!("  - To parse the extracted sections automatically:");
                    info!(
                        "    cpinfo-parser {} {}",
                        args.input.display(),
                        args.output.display()
                    );
                    info!("  - To parse individual section files:");
                    info!("    cpinfo-parser --section-file output/sections/CP_Status.txt");
                }
            }
        }
        Err(extract_error) => {
            if let Some(ref mut progress) = progress_reporter {
                progress.finish(Some("Section extraction failed"));
            }
            error!("Failed to extract sections: {extract_error}");
            return Err(anyhow::anyhow!(
                "Failed to extract sections: {extract_error}"
            ));
        }
    }
    return Ok(());
}

#[expect(
    clippy::cognitive_complexity,
    reason = "Complex error handling and user feedback logic is necessary for comprehensive CLI experience"
)]
#[allow(
    clippy::single_call_fn,
    reason = "Semantic clarity and code organization"
)]
#[inline]
async fn handle_integrated_workflow(
    args: &Args,
    mut progress_reporter: Option<ProgressReporter>,
) -> Result<()> {
    if let Some(ref mut progress) = progress_reporter {
        progress.start("Starting integrated workflow (extraction + parsing)", None);
    } else {
        info!("\u{1f680} Starting integrated workflow: extracting and parsing sections...");
    }

    let orchestrator = IntegratedWorkflowOrchestrator::new();

    match orchestrator
        .process_cpinfo_integrated(&args.input, &args.output, progress_reporter.as_mut())
        .await
    {
        Ok(result) => {
            info!("\u{2705} Integrated workflow completed successfully!");
            info!("\u{1f4ca} PROCESSING SUMMARY:");
            info!(
                "Phase 1: Extracted {} sections",
                result.phase_1_result.sections_extracted
            );
            info!(
                "Phase 2: Processed {} section files",
                result.phase_2_stats.sections_processed
            );
            info!(
                "Phase 2: Extracted {} commands and {} files",
                result.phase_2_stats.commands_extracted, result.phase_2_stats.files_extracted
            );
            info!(
                "\u{1f4c1} Output directory: {:?}",
                result.phase_1_result.output_directory
            );
            info!(
                "\u{23f1}\u{fe0f} Total processing time: {:?}",
                result.total_duration
            );

            if result.phase_1_result.vsx_detected {
                info!(
                    "\u{1f527} VSX detected with {} virtual systems",
                    result.phase_1_result.virtual_systems_count
                );
            }

            info!("\u{1f4c2} Created directories:");
            for dir in &result.phase_1_result.directories_created {
                info!("  - {:?}", dir);
            }

            info!(
                "\u{1f4cb} Results: Both section files AND parsed command/file outputs available"
            );

            if args.verbose {
                info!("\u{1f4a1} Usage suggestions:");
                info!(
                    "  - Browse section files in: {:?}/sections/",
                    result.phase_1_result.output_directory
                );
                info!(
                    "  - Browse commands in: {:?}/commands/",
                    result.phase_1_result.output_directory
                );
                info!(
                    "  - Browse files in: {:?}/files/",
                    result.phase_1_result.output_directory
                );
            }
        }
        Err(workflow_error) => {
            if let Some(ref mut progress) = progress_reporter {
                progress.finish(Some("Integrated workflow failed"));
            }
            error!("\u{274c} Failed to complete integrated workflow: {workflow_error}");
            error!("\u{1f4a1} Troubleshooting:");
            error!(
                "  - Try extract-only mode: cpinfo-parser {} {} --extract-only",
                args.input.display(),
                args.output.display()
            );
            error!("  - Check available disk space and permissions");
            error!("  - Use --verbose flag for detailed error information");
            return Err(anyhow::anyhow!(
                "Failed to complete integrated workflow: {workflow_error}"
            ));
        }
    }
    return Ok(());
}
