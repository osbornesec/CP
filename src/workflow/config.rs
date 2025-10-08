/// Configuration for the integrated workflow system.
///
/// This structure controls various aspects of workflow execution including
/// processing modes, parallelism settings, and output verbosity.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct IntegratedWorkflowConfig {
    /// Whether to show detailed progress information during execution.
    pub detailed_progress: bool,
    /// Whether to only extract data without further processing.
    pub extract_only: bool,
    /// Maximum number of sections to process in parallel.
    pub max_parallel_sections: usize,
    /// Whether to only parse sections without full workflow execution.
    pub parse_sections_only: bool,
}

impl Default for IntegratedWorkflowConfig {
    /// Creates a new instance with default configuration values.
    ///
    /// # Returns
    ///
    /// A new `IntegratedWorkflowConfig` with reasonable defaults for most use cases.
    #[inline]
    fn default() -> Self {
        return Self {
            detailed_progress: true,
            extract_only: false,
            max_parallel_sections: 4,
            parse_sections_only: false,
        };
    }
}

/// Statistics tracking for workflow phases.
///
/// This structure contains metrics and status information collected
/// during the execution of workflow phases.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PhaseStats {
    /// Number of commands that were successfully extracted.
    pub commands_extracted: usize,
    /// Duration of the phase execution.
    pub duration: core::time::Duration,
    /// Number of files that were successfully extracted.
    pub files_extracted: usize,
    /// Number of sections that were processed.
    pub sections_processed: usize,
    /// Whether the phase completed successfully.
    pub success: bool,
}
