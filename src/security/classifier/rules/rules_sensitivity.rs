//! Sensitivity analyzer using statistical keyword analysis

use super::super::traits::{Classifier, SensitivityDetector};
use super::super::ClassificationLevel;
use crate::error::Result;
use core::cmp;

/// Sensitivity analyzer using statistical analysis
pub struct SensitivityAnalyzer {
    keywords: Vec<(String, f64)>, // (keyword, weight)
}

impl SensitivityAnalyzer {
    /// Add default sensitive keywords
    ///
    /// # Details
    ///
    /// Populates the analyzer with common sensitive keywords and their
    /// associated weights for security classification.
    #[allow(
        clippy::cognitive_complexity,
        reason = "Complex initialization of keyword list"
    )]
    fn add_default_keywords(&mut self) {
        let keywords = [
            ("confidential", 0.8),
            ("secret", 0.9),
            ("private", 0.7),
            ("restricted", 0.8),
            ("classified", 0.8),
            ("internal", 0.5),
            ("proprietary", 0.7),
            ("password", 0.9),
            ("key", 0.8),
            ("token", 0.8),
            ("certificate", 0.7),
            ("security", 0.6),
            ("encryption", 0.6),
            ("firewall", 0.6),
            ("authentication", 0.6),
            ("authorization", 0.6),
            ("credential", 0.8),
            ("login", 0.5),
            ("admin", 0.6),
            ("root", 0.7),
        ];

        for (keyword, weight) in keywords {
            self.keywords.push((keyword.to_owned(), weight));
        }
    }

    /// Add custom keyword with weight
    ///
    /// # Arguments
    ///
    /// * `keyword` - The keyword to add for sensitivity detection
    /// * `weight` - The sensitivity weight (0.0-1.0), will be clamped to valid range
    #[inline]
    pub fn add_keyword(&mut self, keyword: &str, weight: f64) {
        let normalized_weight = weight.clamp(0.0, 1.0);
        self.keywords
            .push((keyword.to_lowercase(), normalized_weight));
    }

    /// Get all keywords with their weights
    ///
    /// # Returns
    ///
    /// A slice of tuples containing (keyword, weight) pairs used for sensitivity analysis.
    #[inline]
    #[must_use]
    pub fn get_keywords(&self) -> &[(String, f64)] {
        return &self.keywords;
    }

    /// Get keyword count
    ///
    /// # Returns
    ///
    /// The total number of keywords configured in the analyzer.
    #[inline]
    #[must_use]
    pub const fn keyword_count(&self) -> usize {
        return self.keywords.len();
    }

    /// Create new sensitivity analyzer
    ///
    /// # Returns
    ///
    /// A new `SensitivityAnalyzer` with default sensitivity keywords loaded.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let mut analyzer = Self {
            keywords: Vec::new(),
        };

        analyzer.add_default_keywords();
        return analyzer;
    }
}

impl Classifier for SensitivityAnalyzer {
    /// Classify content based on sensitivity analysis
    ///
    /// # Arguments
    ///
    /// * `_name` - The file name (unused in this implementation)
    /// * `content` - The content to analyze for sensitivity
    ///
    /// # Returns
    ///
    /// The classification level based on sensitivity score analysis.
    ///
    /// # Errors
    ///
    /// Currently does not return errors but uses Result for trait compatibility.
    #[inline]
    fn classify(&self, _name: &str, content: &str) -> Result<ClassificationLevel> {
        let score = self.sensitivity_score(content);
        return Ok(self.score_to_level(score));
    }

    /// Get the name of this classifier
    ///
    /// # Returns
    ///
    /// Static string identifying this classifier implementation.
    #[inline]
    fn name(&self) -> &'static str {
        return "SensitivityAnalyzer";
    }
}

impl SensitivityDetector for SensitivityAnalyzer {
    /// Detect sensitive patterns in content
    ///
    /// # Arguments
    ///
    /// * `content` - The text content to analyze for sensitive patterns
    ///
    /// # Returns
    ///
    /// Vector of detected sensitive keywords found in the content.
    #[inline]
    fn detect_patterns(&self, content: &str) -> Vec<String> {
        let content_lower = content.to_lowercase();

        return self
            .keywords
            .iter()
            .filter_map(|keyword_entry| {
                let keyword = &keyword_entry.0;
                if content_lower.contains(keyword) {
                    return Some(keyword.clone());
                }
                return None;
            })
            .collect();
    }

    /// Convert sensitivity score to classification level
    ///
    /// # Arguments
    ///
    /// * `score` - Sensitivity score between 0.0 and 1.0
    ///
    /// # Returns
    ///
    /// Classification level based on score thresholds.
    #[inline]
    fn score_to_level(&self, score: f64) -> ClassificationLevel {
        return match score {
            score_value if score_value >= 0.8 => ClassificationLevel::Restricted,
            score_value if score_value >= 0.6 => ClassificationLevel::Confidential,
            score_value if score_value >= 0.3 => ClassificationLevel::Internal,
            _ => ClassificationLevel::Public,
        };
    }

    /// Calculate sensitivity score for content
    ///
    /// # Arguments
    ///
    /// * `content` - The text content to score for sensitivity
    ///
    /// # Returns
    ///
    /// A sensitivity score between 0.0 and 1.0, where higher values indicate
    /// more sensitive content. Uses integer-based scoring to avoid floating point operations.
    #[inline]
    #[allow(
        clippy::cognitive_complexity,
        reason = "Complex scoring algorithm with multiple thresholds"
    )]
    #[allow(
        clippy::cast_possible_truncation,
        reason = "Safe truncation with bounds checking"
    )]
    fn sensitivity_score(&self, content: &str) -> f64 {
        let content_lower = content.to_lowercase();
        let content_len = content.len();

        if content_len == 0 {
            return 0.0;
        }

        // Use integer scoring with fixed point arithmetic (scale by 1000)
        let mut total_score = 0_u32;
        let max_score_per_match = 1000_u32; // Represents 1.0 in fixed point

        for keyword_entry in &self.keywords {
            let keyword = &keyword_entry.0;
            let keyword_weight = keyword_entry.1;
            let count = content_lower.matches(keyword).count();
            if count > 0 {
                // Convert weight to fixed point integer (0.0-1.0 -> 0-1000)
                let weight_fixed = if keyword_weight >= 1.0 {
                    1000_u32
                } else if keyword_weight <= 0.0 {
                    0_u32
                } else {
                    // Convert weight using integer arithmetic only
                    // Map common weight values to fixed point integers
                    if keyword_weight >= 0.9 {
                        900_u32
                    } else if keyword_weight >= 0.8 {
                        800_u32
                    } else if keyword_weight >= 0.7 {
                        700_u32
                    } else if keyword_weight >= 0.6 {
                        600_u32
                    } else if keyword_weight >= 0.5 {
                        500_u32
                    } else if keyword_weight >= 0.4 {
                        400_u32
                    } else if keyword_weight >= 0.3 {
                        300_u32
                    } else if keyword_weight >= 0.2 {
                        200_u32
                    } else if keyword_weight >= 0.1 {
                        100_u32
                    } else {
                        50_u32
                    } // Minimum non-zero weight
                };

                // Safe conversion with bounds checking
                let count_u32 = if count > u32::MAX as usize {
                    u32::MAX
                } else {
                    count as u32
                };
                let match_contribution = count_u32.saturating_mul(weight_fixed);
                total_score = total_score.saturating_add(match_contribution);
            }
        }

        if total_score == 0 {
            return 0.0;
        }

        // Normalize by content length (every 100 chars reduces score)
        let length_factor = cmp::max(content_len.saturating_div(100), 1);
        let length_factor_u32 = if length_factor > u32::MAX as usize {
            u32::MAX
        } else {
            length_factor as u32
        };

        let normalized_score = if length_factor_u32 > 0 {
            total_score.saturating_div(length_factor_u32)
        } else {
            total_score
        };

        // Convert to f64 using manual fraction to avoid division
        let capped_score = cmp::min(normalized_score, max_score_per_match);
        let result = if capped_score == 0 {
            0.0
        } else if capped_score >= max_score_per_match {
            1.0
        } else if capped_score >= 900 {
            0.9
        } else if capped_score >= 800 {
            0.8
        } else if capped_score >= 700 {
            0.7
        } else if capped_score >= 600 {
            0.6
        } else if capped_score >= 500 {
            0.5
        } else if capped_score >= 400 {
            0.4
        } else if capped_score >= 300 {
            0.3
        } else if capped_score >= 200 {
            0.2
        } else if capped_score >= 100 {
            0.1
        } else {
            0.05 // Minimum non-zero result
        };

        return result;
    }
}

impl Default for SensitivityAnalyzer {
    /// Create a default sensitivity analyzer
    ///
    /// # Returns
    ///
    /// A new `SensitivityAnalyzer` instance with default configuration.
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}
