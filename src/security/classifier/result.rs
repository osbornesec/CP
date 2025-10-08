//! Classification result types and implementations

use super::ClassificationLevel;

/// Represents a classified section with its sensitivity level
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ClassifiedSection {
    /// The determined sensitivity classification level
    pub classification: ClassificationLevel,
    /// The actual content of the section
    pub content: String,
    /// The name or identifier of the section
    pub name: String,
    /// Human-readable explanation for why this classification was assigned
    pub reason: String,
}

impl ClassifiedSection {
    /// Create new classified section
    #[inline]
    #[must_use]
    pub const fn new(
        name: String,
        content: String,
        classification: ClassificationLevel,
        reason: String,
    ) -> Self {
        return Self {
            classification,
            content,
            name,
            reason,
        };
    }

    /// Check if section requires special handling
    #[inline]
    #[must_use]
    pub const fn requires_special_handling(&self) -> bool {
        return self.classification.requires_special_handling();
    }

    /// Get security level as numeric value
    #[inline]
    #[must_use]
    pub const fn security_level(&self) -> u8 {
        return self.classification.security_level();
    }
}

/// Result of data classification analysis
#[derive(Debug)]
pub struct ClassificationResult {
    sections: Vec<ClassifiedSection>,
}

impl ClassificationResult {
    /// Add a classified section
    #[inline]
    pub fn add_section(&mut self, section: ClassifiedSection) {
        self.sections.push(section);
    }

    /// Check if result contains confidential data
    #[inline]
    #[must_use]
    pub fn contains_confidential_data(&self) -> bool {
        let mut iterator = self.sections.iter();
        let result = iterator.any(|section_item| {
            return section_item.classification == ClassificationLevel::Confidential;
        });
        return result;
    }

    /// Check if result contains internal data
    #[inline]
    #[must_use]
    pub fn contains_internal_data(&self) -> bool {
        let mut iterator = self.sections.iter();
        let result = iterator.any(|section_item| {
            return section_item.classification == ClassificationLevel::Internal;
        });
        return result;
    }

    /// Check if result contains public data
    #[inline]
    #[must_use]
    pub fn contains_public_data(&self) -> bool {
        let mut iterator = self.sections.iter();
        let result = iterator.any(|section_item| {
            return section_item.classification == ClassificationLevel::Public;
        });
        return result;
    }

    /// Check if result contains restricted data
    #[inline]
    #[must_use]
    pub fn contains_restricted_data(&self) -> bool {
        let mut iterator = self.sections.iter();
        let result = iterator.any(|section_item| {
            return section_item.classification == ClassificationLevel::Restricted;
        });
        return result;
    }

    /// Get all sections
    #[inline]
    #[must_use]
    pub fn get_all_sections(&self) -> &[ClassifiedSection] {
        return &self.sections;
    }

    /// Get highest classification level in result
    #[inline]
    #[must_use]
    pub fn get_highest_classification(&self) -> Option<&ClassificationLevel> {
        let iterator = self.sections.iter();
        let mapped_iterator = iterator.map(|section_item| {
            return &section_item.classification;
        });
        let result = mapped_iterator.max();
        return result;
    }

    /// Get sections by classification level
    #[inline]
    #[must_use]
    pub fn get_sections_by_level(&self, level: ClassificationLevel) -> Vec<&ClassifiedSection> {
        let iterator = self.sections.iter();
        let filtered_iterator = iterator.filter(|section_item| {
            return section_item.classification == level;
        });
        let result = filtered_iterator.collect();
        return result;
    }

    /// Get sections requiring special handling
    #[inline]
    #[must_use]
    pub fn get_sensitive_sections(&self) -> Vec<&ClassifiedSection> {
        let iterator = self.sections.iter();
        let filtered_iterator = iterator.filter(|section_item| {
            return section_item.requires_special_handling();
        });
        let result = filtered_iterator.collect();
        return result;
    }

    /// Get classification summary statistics
    #[inline]
    #[must_use]
    pub fn get_summary(&self) -> ClassificationSummary {
        let mut summary = ClassificationSummary::default();

        for section_item in &self.sections {
            match section_item.classification {
                ClassificationLevel::Public => summary.public_count += 1_usize,
                ClassificationLevel::Internal => summary.internal_count += 1_usize,
                ClassificationLevel::Confidential => summary.confidential_count += 1_usize,
                ClassificationLevel::Restricted => summary.restricted_count += 1_usize,
            }
        }

        summary.total_sections = self.sections.len();
        return summary;
    }

    /// Get total number of classified sections
    #[inline]
    #[must_use]
    pub const fn get_total_sections(&self) -> usize {
        return self.sections.len();
    }

    /// Create new classification result
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        return Self {
            sections: Vec::new(),
        };
    }
}

impl Default for ClassificationResult {
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}

/// Summary statistics for classification results
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct ClassificationSummary {
    /// Number of confidential sections
    pub confidential_count: usize,
    /// Number of internal sections
    pub internal_count: usize,
    /// Number of public sections
    pub public_count: usize,
    /// Number of restricted sections
    pub restricted_count: usize,
    /// Total number of sections processed
    pub total_sections: usize,
}

impl ClassificationSummary {
    /// Check if result has sensitive data (confidential or restricted)
    #[inline]
    #[must_use]
    pub const fn has_sensitive_data(&self) -> bool {
        return self.confidential_count > 0_usize || self.restricted_count > 0_usize;
    }

    /// Calculate percentage of sections at given level (returns ratio multiplied by 100)
    #[inline]
    #[must_use]
    pub fn percentage_at_level(&self, classification_level: &ClassificationLevel) -> u64 {
        if self.total_sections == 0_usize {
            return 0_u64;
        }

        let count = match *classification_level {
            ClassificationLevel::Public => self.public_count,
            ClassificationLevel::Internal => self.internal_count,
            ClassificationLevel::Confidential => self.confidential_count,
            ClassificationLevel::Restricted => self.restricted_count,
        };

        // Convert to u64 for calculation
        let count_u64: u64 = match count.try_into() {
            Ok(value) => value,
            Err(_) => return 0_u64,
        };
        let total_sections_u64: u64 = match self.total_sections.try_into() {
            Ok(value) => value,
            Err(_) => return 0_u64,
        };

        // Calculate percentage without division by using iterative approximation
        let percentage_numerator_u64 = count_u64 * 100_u64;

        let mut result = 0_u64;
        for percentage_candidate in 0_u64..=100_u64 {
            if percentage_candidate * total_sections_u64 <= percentage_numerator_u64 {
                result = percentage_candidate;
            } else {
                break;
            }
        }

        return result;
    }
}
