//! Data classification module for security analysis
//!
//! This module provides comprehensive data classification capabilities
//! using trait-based patterns for extensibility and testability.

pub mod levels;
pub mod result;
pub mod rules;
pub mod traits;

pub use levels::ClassificationLevel;
pub use result::{ClassificationResult, ClassifiedSection};
pub use rules::{ContentClassifier, PatternMatcher, SensitivityAnalyzer};
pub use traits::{ClassificationRule, Classifier, SensitivityDetector};

use crate::error::Result;
use crate::security::filter::SensitiveDataFilter;

/// Main data classifier that orchestrates classification process
pub struct DataClassifier {
    _sensitive_filter: SensitiveDataFilter,
    content_classifier: ContentClassifier,
    pattern_matcher: PatternMatcher,
    sensitivity_analyzer: SensitivityAnalyzer,
}

impl DataClassifier {
    /// Classify a single section using all available rules
    ///
    /// # Errors
    ///
    /// Returns an error if any of the classification steps fail.
    #[inline]
    pub fn classify_section(
        &self,
        section_name: &str,
        section_content: &str,
    ) -> Result<ClassifiedSection> {
        // Apply content classification
        let content_result = match self
            .content_classifier
            .classify(section_name, section_content)
        {
            Ok(result) => result,
            Err(error) => return Err(error),
        };

        // Apply pattern matching
        let pattern_result = match self.pattern_matcher.classify(section_name, section_content) {
            Ok(result) => result,
            Err(error) => return Err(error),
        };

        // Apply sensitivity analysis
        let sensitivity_result = match self
            .sensitivity_analyzer
            .classify(section_name, section_content)
        {
            Ok(result) => result,
            Err(error) => return Err(error),
        };

        // Determine final classification (highest security level wins)
        let final_classification = [content_result, pattern_result, sensitivity_result]
            .iter()
            .max()
            .copied()
            .unwrap_or(ClassificationLevel::Public);

        // Generate reasoning
        let content_preview = if section_content.len() > 100 {
            let preview_text = section_content.chars().take(97).collect::<String>();
            format!("{preview_text}...")
        } else {
            section_content.to_owned()
        };

        let level_counts = [content_result, pattern_result, sensitivity_result]
            .iter()
            .fold([0, 0, 0, 0], |mut counts, level| {
                match *level {
                    ClassificationLevel::Public => {
                        counts[0] += 1;
                    }
                    ClassificationLevel::Internal => {
                        counts[1] += 1;
                    }
                    ClassificationLevel::Confidential => {
                        counts[2] += 1;
                    }
                    ClassificationLevel::Restricted => {
                        counts[3] += 1;
                    }
                }
                return counts;
            });

        let reason = format!(
            "Section '{section_name}' classified as {final_classification:?} based on analysis: {} public, {} internal, {} confidential, {} restricted indicators. Content: '{content_preview}'",
            level_counts[0], level_counts[1], level_counts[2], level_counts[3]
        );

        return Ok(ClassifiedSection {
            name: section_name.to_owned(),
            content: section_content.to_owned(),
            classification: final_classification,
            reason,
        });
    }

    /// Classify multiple sections and return comprehensive result
    ///
    /// # Errors
    ///
    /// Returns an error if any of the section classifications fail.
    #[inline]
    pub fn classify_sections(&self, sections: &[(String, String)]) -> Result<ClassificationResult> {
        let mut result = ClassificationResult::new();

        for section_tuple in sections {
            let classified = match self.classify_section(&section_tuple.0, &section_tuple.1) {
                Ok(classified_section) => classified_section,
                Err(error) => return Err(error),
            };
            result.add_section(classified);
        }

        return Ok(result);
    }

    /// Create new data classifier
    ///
    /// # Errors
    ///
    /// Returns an error if the sensitive data filter cannot be initialized.
    #[inline]
    pub fn new() -> Result<Self> {
        let sensitive_filter = match SensitiveDataFilter::new() {
            Ok(filter) => filter,
            Err(error) => return Err(error),
        };

        return Ok(Self {
            _sensitive_filter: sensitive_filter,
            content_classifier: ContentClassifier::new(),
            pattern_matcher: PatternMatcher::new(),
            sensitivity_analyzer: SensitivityAnalyzer::new(),
        });
    }
}

impl Default for DataClassifier {
    #[inline]
    fn default() -> Self {
        return match Self::new() {
            Ok(classifier) => classifier,
            Err(error) => {
                // This is a programming error - the sensitive data filter should be initializable
                #[allow(
                    clippy::panic,
                    reason = "Default trait requires Self return, not Result"
                )]
                {
                    panic!("Failed to create default DataClassifier: {error:?}")
                }
            }
        };
    }
}
