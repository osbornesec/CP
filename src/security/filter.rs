use crate::error::{CpinfoError, Result};
use regex::Regex;

/// Sensitive data patterns for detection and filtering
#[derive(Debug)]
pub struct SensitiveDataFilter {
    /// Patterns for detecting API keys and tokens
    api_key: Vec<Regex>,
    /// Check Point specific sensitive patterns
    checkpoint_sensitive: Vec<Regex>,
    /// Patterns for detecting IP addresses
    ip_address: Vec<Regex>,
    /// Patterns for detecting passwords and credentials
    password: Vec<Regex>,
    /// Patterns for detecting private cryptographic material
    private_key: Vec<Regex>,
}

/// Represents a detected sensitive data match
///
/// This structure contains information about a piece of sensitive data that was
/// detected in content during security scanning. It includes the type of pattern
/// that matched, the location of the match, and the actual matched text.
///
/// # Examples
///
/// ```ignore
/// use cpinfo_parser::security::SensitiveMatch;
///
/// let sensitive_match = SensitiveMatch {
///     pattern_type: "password".to_string(),
///     start: 10,
///     end: 25,
///     matched_text: "password=secret".to_string(),
/// };
///
/// assert_eq!(sensitive_match.pattern_type, "password");
/// assert_eq!(sensitive_match.start, 10);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SensitiveMatch {
    /// The ending byte position of the match in the original content
    pub end: usize,
    /// The actual text that matched the sensitive pattern
    pub matched_text: String,
    /// The type of sensitive pattern that was detected (e.g., "password", "`api_key`", "`private_key`")
    pub pattern_type: String,
    /// The starting byte position of the match in the original content
    pub start: usize,
}

impl SensitiveDataFilter {
    /// Helper to add pattern matches to the results
    #[inline]
    fn add_pattern_matches(
        matches: &mut Vec<SensitiveMatch>,
        content: &str,
        patterns: &[Regex],
        pattern_type: &str,
    ) {
        for pattern in patterns {
            for mat in pattern.find_iter(content) {
                matches.push(SensitiveMatch {
                    pattern_type: pattern_type.to_owned(),
                    start: mat.start(),
                    end: mat.end(),
                    matched_text: mat.as_str().to_owned(),
                });
            }
        }
    }

    /// Build API key detection patterns
    #[inline]
    #[allow(
        clippy::single_call_fn,
        reason = "Helper function to reduce complexity in new()"
    )]
    fn build_api_key_patterns() -> Result<Vec<Regex>> {
        let api_key_patterns = vec![
            r#"(?i)(api[_-]?key|secret|token)\s*[:=]\s*['"]?([A-Za-z0-9_\-]{16,})['"]?"#,
            "AKIA[0-9A-Z]{16}",
            "ghp_[A-Za-z0-9]{36}",
            "sk-[A-Za-z0-9]{32,}",
            r"(?i)(cp_api_key|checkpoint_key)\s*[:=]\s*([A-Za-z0-9_\-]{16,})",
        ];
        return api_key_patterns
            .into_iter()
            .map(|pattern| {
                return Regex::new(pattern).map_err(|error| {
                    return CpinfoError::validation_error(format!(
                        "Invalid API key regex: {error}"
                    ));
                });
            })
            .collect();
    }

    /// Build Check Point specific sensitive patterns
    #[inline]
    #[allow(
        clippy::single_call_fn,
        reason = "Helper function to reduce complexity in new()"
    )]
    fn build_checkpoint_patterns() -> Result<Vec<Regex>> {
        let checkpoint_patterns = vec![
            r"(?i)(sic_secret|sic_key)\s*[:=]\s*([A-Za-z0-9_\-]{8,})",
            r#"(?i)(snmp_community|community_string)\s*[:=]\s*([^\s"']+)"#,
            r#"(?i)(database_url|db_url)\s*[:=]\s*['"]?[^'"]*://[^:]+:[^@]+@[^'"]*['"]?"#,
        ];
        return checkpoint_patterns
            .into_iter()
            .map(|pattern| {
                return Regex::new(pattern).map_err(|error| {
                    return CpinfoError::validation_error(format!(
                        "Invalid Check Point regex: {error}"
                    ));
                });
            })
            .collect();
    }

    /// Build IP address detection patterns
    #[inline]
    #[allow(
        clippy::single_call_fn,
        reason = "Helper function to reduce complexity in new()"
    )]
    fn build_ip_patterns() -> Result<Vec<Regex>> {
        let ip_patterns = vec![
            r"\b(?:192\.168\.|10\.|172\.(?:1[6-9]|2[0-9]|3[01])\.)\d{1,3}\.\d{1,3}\b",
            r"(?i)(management_ip|internal_ip|cluster_ip)\s*[:=]\s*(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})",
        ];
        return ip_patterns
            .into_iter()
            .map(|pattern| {
                return Regex::new(pattern).map_err(|error| {
                    return CpinfoError::validation_error(format!("Invalid IP regex: {error}"));
                });
            })
            .collect();
    }

    /// Build password detection patterns based on OWASP guidelines
    #[inline]
    #[allow(
        clippy::single_call_fn,
        reason = "Helper function to reduce complexity in new()"
    )]
    fn build_password_patterns() -> Result<Vec<Regex>> {
        let password_patterns = vec![
            r#"(?i)(password|passwd|pwd)\s*[:=]\s*([^\s"']+|"[^"]*"|'[^']*')"#,
            r#"(?i)(admin_password|user_password|root_password)\s*[:=]\s*([^\s"']+)"#,
            r#"(?i)(cert_password|key_password|keystore_password)\s*[:=]\s*([^\s"']+)"#,
            r#"(?i)(db_password|database_password)\s*[:=]\s*([^\s"']+)"#,
        ];
        return password_patterns
            .into_iter()
            .map(|pattern| {
                return Regex::new(pattern).map_err(|error| {
                    return CpinfoError::validation_error(format!(
                        "Invalid password regex: {error}"
                    ));
                });
            })
            .collect();
    }

    /// Build private key detection patterns
    #[inline]
    #[allow(
        clippy::single_call_fn,
        reason = "Helper function to reduce complexity in new()"
    )]
    fn build_private_key_patterns() -> Result<Vec<Regex>> {
        let private_key_patterns = vec![
            r"-----BEGIN (RSA |DSA |EC |OPENSSH )?PRIVATE KEY-----[\s\S]*?-----END (RSA |DSA |EC |OPENSSH )?PRIVATE KEY-----",
            r"-----BEGIN ENCRYPTED PRIVATE KEY-----[\s\S]*?-----END ENCRYPTED PRIVATE KEY-----",
            r#"(?i)(private_key_file|ssh_private_key)\s*[:=]\s*([^\s"']+)"#,
        ];
        return private_key_patterns
            .into_iter()
            .map(|pattern| {
                return Regex::new(pattern).map_err(|error| {
                    return CpinfoError::validation_error(format!(
                        "Invalid private key regex: {error}"
                    ));
                });
            })
            .collect();
    }

    /// Detect sensitive patterns without filtering (for analysis)
    #[inline]
    #[must_use]
    pub fn detect_sensitive_patterns(&self, content: &str) -> Vec<SensitiveMatch> {
        let mut matches = Vec::new();

        Self::add_pattern_matches(&mut matches, content, &self.password, "password");
        Self::add_pattern_matches(&mut matches, content, &self.api_key, "api_key");
        Self::add_pattern_matches(&mut matches, content, &self.private_key, "private_key");
        Self::add_pattern_matches(&mut matches, content, &self.ip_address, "ip_address");
        Self::add_pattern_matches(
            &mut matches,
            content,
            &self.checkpoint_sensitive,
            "checkpoint_secret",
        );

        return matches;
    }

    /// Filter API key patterns from content
    #[inline]
    fn filter_api_keys(&self, content: &str) -> String {
        let mut filtered = content.to_owned();
        for pattern in &self.api_key {
            filtered = pattern
                .replace_all(&filtered, "[REDACTED_API_KEY]")
                .to_string();
        }
        return filtered;
    }

    /// Filter Check Point specific sensitive patterns
    #[inline]
    fn filter_checkpoint_specific(&self, content: &str) -> String {
        let mut filtered = content.to_owned();
        for pattern in &self.checkpoint_sensitive {
            filtered = pattern
                .replace_all(&filtered, |caps: &regex::Captures| {
                    return format!("{}[REDACTED_CHECKPOINT_SECRET]", &caps[1]);
                })
                .to_string();
        }
        return filtered;
    }

    /// Filter sensitive data from string content
    #[inline]
    #[must_use]
    pub fn filter_content(&self, content: &str) -> String {
        let mut filtered = content.to_owned();

        // Apply all filtering patterns
        filtered = self.filter_passwords(&filtered);
        filtered = self.filter_api_keys(&filtered);
        filtered = self.filter_private_keys(&filtered);
        filtered = self.filter_ip_addresses(&filtered);
        filtered = self.filter_checkpoint_specific(&filtered);

        return filtered;
    }

    /// Filter sensitive data from file content
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if regex patterns fail to compile.
    #[inline]
    pub fn filter_file_content<P: AsRef<std::path::Path>>(path: P) -> Result<String> {
        let filter = match Self::new() {
            Ok(value) => value,
            Err(error) => return Err(error),
        };
        let content = match std::fs::read_to_string(path) {
            Ok(file_content) => file_content,
            Err(error) => return Err(error.into()),
        };
        return Ok(filter.filter_content(&content));
    }

    /// Filter IP address patterns from content (anonymize internal IPs)
    #[inline]
    fn filter_ip_addresses(&self, content: &str) -> String {
        let mut filtered = content.to_owned();
        for pattern in &self.ip_address {
            filtered = pattern
                .replace_all(&filtered, |caps: &regex::Captures| {
                    if caps.len() > 1 {
                        return format!("{}: [REDACTED_IP]", &caps[1]);
                    } else {
                        return "[REDACTED_IP]".to_owned();
                    }
                })
                .to_string();
        }
        return filtered;
    }

    /// Filter password patterns from content
    #[inline]
    fn filter_passwords(&self, content: &str) -> String {
        let mut filtered = content.to_owned();
        for pattern in &self.password {
            filtered = pattern
                .replace_all(&filtered, |caps: &regex::Captures| {
                    return format!("{}[REDACTED_PASSWORD]", &caps[1]);
                })
                .to_string();
        }
        return filtered;
    }

    /// Filter private key patterns from content
    #[inline]
    fn filter_private_keys(&self, content: &str) -> String {
        let mut filtered = content.to_owned();
        for pattern in &self.private_key {
            filtered = pattern
                .replace_all(&filtered, "[REDACTED_PRIVATE_KEY]")
                .to_string();
        }
        return filtered;
    }

    /// Create new sensitive data filter with comprehensive patterns
    ///
    /// # Errors
    ///
    /// Returns an error if any regex pattern fails to compile.
    #[inline]
    pub fn new() -> Result<Self> {
        let password = match Self::build_password_patterns() {
            Ok(patterns) => patterns,
            Err(error) => return Err(error),
        };
        let api_key = match Self::build_api_key_patterns() {
            Ok(patterns) => patterns,
            Err(error) => return Err(error),
        };
        let private_key = match Self::build_private_key_patterns() {
            Ok(patterns) => patterns,
            Err(error) => return Err(error),
        };
        let ip_address = match Self::build_ip_patterns() {
            Ok(patterns) => patterns,
            Err(error) => return Err(error),
        };
        let checkpoint_sensitive = match Self::build_checkpoint_patterns() {
            Ok(patterns) => patterns,
            Err(error) => return Err(error),
        };

        return Ok(Self {
            api_key,
            checkpoint_sensitive,
            ip_address,
            password,
            private_key,
        });
    }
}

impl Default for SensitiveDataFilter {
    #[inline]
    fn default() -> Self {
        return Self::new().unwrap_or_else(|_error| {
            return Self {
                api_key: Vec::new(),
                checkpoint_sensitive: Vec::new(),
                ip_address: Vec::new(),
                password: Vec::new(),
                private_key: Vec::new(),
            };
        });
    }
}
