//! Pattern-based classification using regex matching

use super::super::traits::Classifier;
use super::super::ClassificationLevel;
use crate::error::Result;
use regex::Regex;

/// Pattern-based classifier using regex matching
pub struct PatternMatcher {
    patterns: Vec<(Regex, ClassificationLevel)>,
}

impl PatternMatcher {
    /// Add default sensitive patterns
    fn add_default_patterns(&mut self) {
        // Email patterns
        if let Ok(email_regex) = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
        {
            self.patterns
                .push((email_regex, ClassificationLevel::Internal));
        }

        // IP address patterns
        if let Ok(ip_regex) = Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b") {
            self.patterns
                .push((ip_regex, ClassificationLevel::Internal));
        }

        // Password/key patterns
        if let Ok(key_regex) = Regex::new(r"(?i)(password|key|secret|token)\s*[:=]\s*\S+") {
            self.patterns
                .push((key_regex, ClassificationLevel::Restricted));
        }

        // Certificate patterns
        if let Ok(cert_regex) = Regex::new(r"-----BEGIN [A-Z\s]+ CERTIFICATE-----") {
            self.patterns
                .push((cert_regex, ClassificationLevel::Confidential));
        }

        // MAC address patterns
        if let Ok(mac_regex) = Regex::new("([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})") {
            self.patterns
                .push((mac_regex, ClassificationLevel::Internal));
        }

        // URL patterns with credentials
        if let Ok(url_regex) = Regex::new(r"https?://[^:]+:[^@]+@[^\s]+") {
            self.patterns
                .push((url_regex, ClassificationLevel::Restricted));
        }
    }

    /// Add custom pattern with classification level
    ///
    /// # Arguments
    ///
    /// * `pattern` - The regex pattern to match against content
    /// * `level` - The security classification level to assign to matches
    ///
    /// # Returns
    ///
    /// `Ok(())` on successful pattern addition, or an error if the regex is invalid
    ///
    /// # Errors
    ///
    /// Returns an error if the provided pattern is not a valid regex
    #[inline]
    pub fn add_pattern(&mut self, pattern: &str, level: ClassificationLevel) -> Result<()> {
        let regex = match Regex::new(pattern) {
            Ok(pattern_regex) => pattern_regex,
            Err(error) => {
                return Err(crate::error::CpinfoError::validation_error(format!(
                    "Invalid regex pattern: {error}"
                )))
            }
        };

        self.patterns.push((regex, level));
        return Ok(());
    }

    /// Find all matching patterns in content
    #[inline]
    #[must_use]
    pub fn find_matches(&self, content: &str) -> Vec<(String, ClassificationLevel)> {
        let mut matches = Vec::new();
        for pattern_tuple in &self.patterns {
            if pattern_tuple.0.is_match(content) {
                matches.push((pattern_tuple.0.as_str().to_owned(), pattern_tuple.1));
            }
        }
        return matches;
    }

    /// Create new pattern matcher with default patterns
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let mut matcher = Self {
            patterns: Vec::new(),
        };

        matcher.add_default_patterns();
        return matcher;
    }

    /// Get count of loaded patterns
    #[inline]
    #[must_use]
    pub const fn pattern_count(&self) -> usize {
        return self.patterns.len();
    }
}

impl Classifier for PatternMatcher {
    #[inline]
    fn classify(&self, _name: &str, content: &str) -> Result<ClassificationLevel> {
        let mut max_level = ClassificationLevel::Public;
        for pattern_tuple in &self.patterns {
            if pattern_tuple.0.is_match(content) && pattern_tuple.1 > max_level {
                max_level = pattern_tuple.1;
            }
        }
        return Ok(max_level);
    }

    #[inline]
    fn name(&self) -> &'static str {
        return "PatternMatcher";
    }
}

impl Default for PatternMatcher {
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}
