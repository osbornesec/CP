//! Content-based classification using rule engine

use super::super::traits::{ClassificationRule, Classifier};
use super::super::ClassificationLevel;
use super::rules_standard::{
    HardwareInfoRule, NetworkConfigRule, PublicInfoRule, SecurityPolicyRule, SystemStatusRule,
};
use crate::error::Result;

/// Content-based classifier using predefined rules
pub struct ContentClassifier {
    rules: Vec<Box<dyn ClassificationRule + Send + Sync>>,
}

impl ContentClassifier {
    /// Add default classification rules
    fn add_default_rules(&mut self) {
        self.add_rule(Box::new(HardwareInfoRule));
        self.add_rule(Box::new(NetworkConfigRule));
        self.add_rule(Box::new(PublicInfoRule));
        self.add_rule(Box::new(SecurityPolicyRule));
        self.add_rule(Box::new(SystemStatusRule));
    }

    /// Add custom classification rule
    #[inline]
    pub fn add_rule(&mut self, rule: Box<dyn ClassificationRule + Send + Sync>) {
        self.rules.push(rule);
    }

    /// Get rule descriptions for debugging
    #[must_use]
    #[inline]
    pub fn get_rule_descriptions(&self) -> Vec<&'static str> {
        return self
            .rules
            .iter()
            .map(|classification_rule| {
                return classification_rule.description();
            })
            .collect();
    }

    /// Create new content classifier with default rules
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        let mut classifier = Self { rules: Vec::new() };
        classifier.add_default_rules();
        return classifier;
    }

    /// Get count of loaded rules
    #[must_use]
    #[inline]
    pub fn rule_count(&self) -> usize {
        return self.rules.len();
    }
}

impl Classifier for ContentClassifier {
    #[inline]
    fn classify(&self, name: &str, content: &str) -> Result<ClassificationLevel> {
        // Find the highest classification level from applicable rules
        let classification_level = self
            .rules
            .iter()
            .filter(|classification_rule| {
                return classification_rule.applies(name, content);
            })
            .map(|classification_rule| {
                return classification_rule.level();
            })
            .max()
            .unwrap_or(ClassificationLevel::Public);

        return Ok(classification_level);
    }

    #[inline]
    fn name(&self) -> &'static str {
        return "ContentClassifier";
    }
}

impl Default for ContentClassifier {
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}
