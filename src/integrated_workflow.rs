use crate::error::{CpinfoError, Result};
use crate::extraction::types::OrganizedExtractionResult;
use crate::extraction::SectionExtractor;
use crate::progress::ProgressReporter;
use crate::section_parser::parser::{SectionFileParser, SectionFileProcessingStats};
use core::time::Duration;
use std::path::Path;
use std::time::Instant;
use tokio::fs;
use tokio::task;
use tracing::{error, info};

/// Result of the integrated workflow containing phase results and timing
#[derive(Debug)]
#[non_exhaustive]
pub struct IntegratedWorkflowResult {
    /// Result from phase 1 (extraction)
    pub phase_1_result: OrganizedExtractionResult,
    /// Statistics from phase 2 (parsing)
    pub phase_2_stats: SectionFileProcessingStats,
    /// Total duration of the workflow
    pub total_duration: Duration,
}

/// Orchestrator for running the integrated workflow
#[non_exhaustive]
pub struct IntegratedWorkflowOrchestrator;

impl IntegratedWorkflowOrchestrator {
    /// Execute workflow on the provided runtime
    ///
    /// # Arguments
    /// * `runtime` - The tokio runtime to execute the workflow on
    /// * `input_path` - Path to the input file
    /// * `output_path` - Path to the output directory
    ///
    /// # Errors
    /// Returns `CpinfoError` if workflow execution fails
    #[inline]
    #[allow(
        clippy::needless_pass_by_value,
        reason = "Runtime is consumed by block_on"
    )]
    fn execute_workflow_on_runtime<P: AsRef<Path>>(
        &self,
        runtime: tokio::runtime::Runtime,
        input_path: P,
        output_path: P,
    ) -> Result<()> {
        return runtime.block_on(async move {
            let mut progress_reporter = None;
            let workflow_result = match self
                .process_cpinfo_integrated(
                    input_path.as_ref(),
                    output_path.as_ref(),
                    &mut progress_reporter,
                )
                .await
            {
                Ok(result) => result,
                Err(workflow_error) => return Err(workflow_error),
            };
            drop(workflow_result); // Explicitly drop instead of unused variable
            return Ok(());
        });
    }

    /// Creates a new integrated workflow orchestrator
    ///
    /// # Returns
    /// A new `IntegratedWorkflowOrchestrator` instance
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        return Self {};
    }

    /// Process the complete integrated workflow
    ///
    /// # Arguments
    /// * `input_path` - Path to the input cpinfo file
    /// * `output_dir` - Directory where output files will be written
    /// * `progress_reporter` - Optional progress reporter for status updates
    ///
    /// # Errors
    /// Returns `CpinfoError` if either phase of processing fails
    #[inline]
    pub async fn process_cpinfo_integrated(
        &self,
        input_path: &Path,
        output_dir: &Path,
        progress_reporter: &mut Option<ProgressReporter>,
    ) -> Result<IntegratedWorkflowResult> {
        let total_start_time = Instant::now();

        // Phase 1: Extract sections
        let phase_1_result =
            match SectionExtractor::extract_sections_organized(input_path, output_dir) {
                Ok(extraction_result) => extraction_result,
                Err(extraction_error) => return Err(extraction_error),
            };
        info!(
            "Phase 1 (Extraction) completed in {:?}",
            total_start_time.elapsed()
        );

        if let Some(reporter) = progress_reporter.as_mut() {
            reporter.set_operation("Phase 2: Parsing extracted sections");
        }

        // Phase 2: Parse extracted section files
        let section_files_dir = output_dir.join("sections");
        let phase_2_stats = match self
            .process_section_files(&section_files_dir, progress_reporter)
            .await
        {
            Ok(stats) => stats,
            Err(parsing_error) => return Err(parsing_error),
        };
        info!(
            "Phase 2 (Parsing) completed in {:?}",
            total_start_time.elapsed()
        );

        let total_duration = total_start_time.elapsed();
        info!("Integrated workflow finished in {:?}", total_duration);

        return Ok(IntegratedWorkflowResult {
            phase_1_result,
            phase_2_stats,
            total_duration,
        });
    }

    /// Process all section files in the given directory
    ///
    /// # Arguments
    /// * `sections_dir` - Directory containing section files to process
    /// * `progress_reporter` - Optional progress reporter for status updates
    ///
    /// # Errors
    /// Returns `CpinfoError` if directory reading or file processing fails
    #[inline]
    async fn process_section_files(
        &self,
        sections_dir: &Path,
        progress_reporter: &mut Option<ProgressReporter>,
    ) -> Result<SectionFileProcessingStats> {
        let mut read_dir = match fs::read_dir(sections_dir).await.map_err(|io_error| {
            return CpinfoError::Io(std::io::Error::new(
                io_error.kind(),
                format!(
                    "Failed to read sections directory: {}",
                    sections_dir.display()
                ),
            ));
        }) {
            Ok(directory_reader) => directory_reader,
            Err(read_error) => return Err(read_error),
        };

        let mut join_handles = Vec::new();
        let mut total_files = 0;
        let mut file_paths = Vec::new();

        while let Some(entry) = match read_dir.next_entry().await {
            Ok(directory_entry) => directory_entry,
            Err(entry_error) => return Err(entry_error.into()),
        } {
            let path = entry.path();
            if path.is_file() {
                total_files += 1;
                file_paths.push(path);
            }
        }

        if let Some(reporter) = progress_reporter.as_mut() {
            reporter.start("Parsing section files", Some(total_files));
        }

        for path in file_paths {
            let handle = task::spawn(async move {
                let parser = SectionFileParser::new();
                return parser.process_section_file_async(&path).await;
            });
            join_handles.push(handle);
        }

        let mut aggregated_stats = SectionFileProcessingStats::default();
        for handle in join_handles {
            match handle.await {
                Ok(Ok(result)) => {
                    aggregated_stats.total_files_processed += result.stats.total_files_processed;
                    aggregated_stats.total_commands_found += result.stats.total_commands_found;
                    aggregated_stats.total_files_found += result.stats.total_files_found;
                    aggregated_stats.total_bytes_processed += result.stats.total_bytes_processed;
                }
                Ok(Err(processing_error)) => {
                    error!("Error processing file: {processing_error}");
                }
                Err(task_error) => {
                    error!("Task error: {task_error}");
                }
            }
            if let Some(reporter) = progress_reporter.as_mut() {
                reporter.increment(1);
            }
        }

        if let Some(mut reporter) = progress_reporter.take() {
            reporter.finish(Some("Finished parsing section files"));
        }

        return Ok(aggregated_stats);
    }

    /// Run the integrated workflow synchronously
    ///
    /// # Arguments
    /// * `input_path` - Path to the input cpinfo file
    /// * `output_path` - Path to the output directory
    ///
    /// # Errors
    /// Returns `CpinfoError` if workflow processing fails at any stage.
    #[inline]
    pub fn run_integrated_workflow<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
    ) -> Result<()> {
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(error) => {
                return Err(CpinfoError::Io(std::io::Error::other(format!(
                    "Failed to create async runtime: {error}"
                ))));
            }
        };
        return self.execute_workflow_on_runtime(runtime, input_path, output_path);
    }
}

impl Default for IntegratedWorkflowOrchestrator {
    /// Creates a default integrated workflow orchestrator
    ///
    /// # Returns
    /// A new `IntegratedWorkflowOrchestrator` instance
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}
