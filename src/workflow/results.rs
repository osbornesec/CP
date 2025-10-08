use super::config::IntegratedWorkflowConfig;
use crate::extraction::OrganizedExtractionResult;

// Re-export PhaseStats so it can be imported from results module
pub use super::config::PhaseStats;

/// Represents the result of an integrated workflow execution.
///
/// Contains results from both phases, timing information, and configuration details.
/// This struct provides methods to analyze the overall processing results.
#[derive(Debug)]
#[non_exhaustive]
pub struct IntegratedWorkflowResult {
    pub config: IntegratedWorkflowConfig,
    pub phase_1_result: OrganizedExtractionResult,
    pub phase_2_stats: PhaseStats,
    pub success: bool,
    pub total_duration: core::time::Duration,
}

impl IntegratedWorkflowResult {
    /// Creates a summary string of the workflow results.
    ///
    /// # Returns
    ///
    /// A formatted string describing sections extracted, commands, files, and duration.
    #[must_use]
    #[inline]
    pub fn summary(&self) -> String {
        return format!(
            "Integrated workflow: {} sections extracted, {} commands, {} files (took {:?})",
            self.phase_1_result.sections_extracted,
            self.phase_2_stats.commands_extracted,
            self.phase_2_stats.files_extracted,
            self.total_duration
        );
    }

    /// Calculates the total number of files created across all phases.
    ///
    /// # Returns
    ///
    /// Sum of sections extracted, commands extracted, and files extracted.
    #[must_use]
    #[inline]
    pub const fn total_files_created(&self) -> usize {
        return self.phase_1_result.sections_extracted
            + self.phase_2_stats.commands_extracted
            + self.phase_2_stats.files_extracted;
    }

    /// Calculates the total number of sections processed across all phases.
    ///
    /// # Returns
    ///
    /// Sum of sections extracted in phase 1 and sections processed in phase 2.
    #[must_use]
    #[inline]
    pub const fn total_sections_processed(&self) -> usize {
        return self.phase_1_result.sections_extracted + self.phase_2_stats.sections_processed;
    }
}
