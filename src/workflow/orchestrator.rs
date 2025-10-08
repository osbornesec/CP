use super::config::IntegratedWorkflowConfig;
use super::results::{IntegratedWorkflowResult, PhaseStats};
use crate::error::Result;
use crate::parser::CpinfoParser;
use crate::progress::ProgressReporter;
use crate::section_parser::SectionFileParser;
use core::convert::AsRef;
use std::path::Path;

/// The orchestrator for integrated workflow processing.
///
/// This struct coordinates the complete cpinfo file processing workflow,
/// combining section extraction and parsing phases.
pub struct IntegratedWorkflowOrchestrator {
    _cpinfo_parser: CpinfoParser,
    _section_parser: SectionFileParser,
}

impl IntegratedWorkflowOrchestrator {
    /// Creates a new orchestrator with default parser instances.
    ///
    /// # Returns
    ///
    /// A new `IntegratedWorkflowOrchestrator` with default configuration.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        return Self {
            _cpinfo_parser: CpinfoParser::new(),
            _section_parser: SectionFileParser::new(),
        };
    }

    /// Processes a cpinfo file through the complete integrated workflow with default configuration.
    ///
    /// This method combines both extraction and parsing phases using default settings.
    ///
    /// # Arguments
    ///
    /// * `input_path` - Path to the input cpinfo file
    /// * `output_path` - Directory where output files will be saved
    /// * `progress_reporter` - Optional progress reporting interface
    ///
    /// # Returns
    ///
    /// Complete workflow result with statistics from both phases.
    ///
    /// # Errors
    ///
    /// Returns error if extraction or parsing fails, or if I/O operations fail.
    #[inline]
    pub async fn process_cpinfo_integrated<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        input_path: P1,
        output_path: P2,
        progress_reporter: Option<&mut ProgressReporter>,
    ) -> Result<IntegratedWorkflowResult> {
        let workflow_result = self
            .process_cpinfo_with_config(
                input_path,
                output_path,
                progress_reporter,
                IntegratedWorkflowConfig::default(),
            )
            .await;

        return workflow_result;
    }

    /// Processes a cpinfo file through the complete integrated workflow with custom configuration.
    ///
    /// This method combines both extraction and parsing phases using the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `input_path` - Path to the input cpinfo file
    /// * `output_path` - Directory where output files will be saved
    /// * `progress_reporter` - Optional progress reporting interface
    /// * `config` - Workflow configuration settings
    ///
    /// # Returns
    ///
    /// Complete workflow result with statistics from both phases.
    ///
    /// # Errors
    ///
    /// Returns error if extraction or parsing fails, or if I/O operations fail.
    #[inline]
    pub async fn process_cpinfo_with_config<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        input_file_path: P1,
        output_dir_path: P2,
        mut progress_reporter: Option<&mut ProgressReporter>,
        config: IntegratedWorkflowConfig,
    ) -> Result<IntegratedWorkflowResult> {
        let start_time = std::time::Instant::now();

        let resolved_input_path = input_file_path.as_ref();
        let resolved_output_path = output_dir_path.as_ref();

        let phase_1_result = match self.execute_phase_1(
            resolved_input_path,
            resolved_output_path,
            &mut progress_reporter,
        ) {
            Ok(extraction_result) => extraction_result,
            Err(phase_1_error) => {
                if let Some(ref mut progress) = progress_reporter {
                    progress.finish(Some("Phase 1 failed - extraction incomplete"));
                }
                return Err(phase_1_error);
            }
        };

        let (sections_processed, commands_extracted, files_extracted) = match self
            .execute_phase_2(&phase_1_result.output_directory, &mut progress_reporter)
            .await
        {
            Ok(phase_2_result) => phase_2_result,
            Err(phase_2_error) => {
                if let Some(ref mut progress) = progress_reporter {
                    progress.finish(Some("Phase 2 failed - parsing incomplete"));
                }
                return Err(phase_2_error);
            }
        };

        let total_duration = start_time.elapsed();

        let phase_2_stats = PhaseStats {
            sections_processed,
            commands_extracted,
            files_extracted,
            duration: total_duration,
            success: true,
        };

        let result = IntegratedWorkflowResult {
            phase_1_result,
            phase_2_stats,
            total_duration,
            success: true,
            config,
        };

        if let Some(ref mut progress) = progress_reporter {
            progress.finish(Some(&result.summary()));
        }

        return Ok(result);
    }

    /// Creates a new orchestrator with provided parser instances.
    ///
    /// # Arguments
    ///
    /// * `cpinfo_parser` - Configured cpinfo parser instance
    /// * `section_parser` - Configured section file parser instance
    ///
    /// # Returns
    ///
    /// A new `IntegratedWorkflowOrchestrator` with the provided parsers.
    #[inline]
    #[must_use]
    pub const fn with_parsers(
        cpinfo_parser: CpinfoParser,
        section_parser: SectionFileParser,
    ) -> Self {
        return Self {
            _cpinfo_parser: cpinfo_parser,
            _section_parser: section_parser,
        };
    }
}

impl Default for IntegratedWorkflowOrchestrator {
    /// Creates a default orchestrator instance.
    ///
    /// # Returns
    ///
    /// A new orchestrator with default configuration.
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}
