//! Classification traits for extensible security analysis

use super::ClassificationLevel;
use crate::error::Result;

/// Core trait for classification implementations
pub trait Classifier {
    /// Classify content and return security level
    ///
    /// # Errors
    ///
    /// Returns error if classification fails due to invalid input or processing errors
    fn classify(&self, name: &str, content: &str) -> Result<ClassificationLevel>;

    /// Get classifier name for debugging
    fn name(&self) -> &'static str;
}

/// Trait for specific classification rules
pub trait ClassificationRule {
    /// Check if rule applies to given content
    fn applies(&self, name: &str, content: &str) -> bool;

    /// Get rule description
    fn description(&self) -> &'static str;

    /// Get classification level for this rule
    fn level(&self) -> ClassificationLevel;
}

/// Trait for sensitivity detection algorithms
pub trait SensitivityDetector {
    /// Detect sensitive patterns in content
    fn detect_patterns(&self, content: &str) -> Vec<String>;

    /// Convert score to classification level
    #[inline]
    fn score_to_level(&self, score: f64) -> ClassificationLevel {
        match score {
            score_value if score_value >= 0.8_f64 => return ClassificationLevel::Restricted,
            score_value if score_value >= 0.6_f64 => return ClassificationLevel::Confidential,
            score_value if score_value >= 0.3_f64 => return ClassificationLevel::Internal,
            _ => return ClassificationLevel::Public,
        }
    }

    /// Calculate sensitivity score (0.0 to 1.0)
    fn sensitivity_score(&self, content: &str) -> f64;
}
