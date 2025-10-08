//! `CPInfo` file format detection and validation

use crate::error::Result;
use std::path::Path;

/// `CPInfo` file format information
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CpinfoFormat {
    /// Whether the file has a valid cpinfo header
    pub is_valid: bool,
    /// File format variant (if detectable)
    pub variant: FormatVariant,
    /// Detected version string (if any)
    pub version: Option<String>,
}

/// `CPInfo` format variants
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FormatVariant {
    /// Standard cpinfo format
    Standard,
    /// Unknown or undetected format
    Unknown,
    /// VSX (Virtual System) format
    VSX,
}

impl CpinfoFormat {
    /// Check if this represents a valid cpinfo format
    #[must_use]
    #[inline]
    pub const fn is_valid_cpinfo(&self) -> bool {
        return self.is_valid;
    }

    /// Create a new format info structure
    #[must_use]
    #[inline]
    pub const fn new(is_valid: bool, version: Option<String>, variant: FormatVariant) -> Self {
        return Self {
            is_valid,
            variant,
            version,
        };
    }

    /// Get the format variant
    #[must_use]
    #[inline]
    pub const fn variant(&self) -> &FormatVariant {
        return &self.variant;
    }

    /// Get the detected version
    #[must_use]
    #[inline]
    pub fn version(&self) -> Option<&str> {
        return self.version.as_deref();
    }
}

/// Format detector for cpinfo files
#[non_exhaustive]
pub struct FormatDetector;

impl FormatDetector {
    /// Detect the format of a cpinfo file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened or read
    /// - The file has an invalid or corrupted header
    /// - The file format is not recognized as a valid cpinfo file
    #[inline]
    pub fn detect_format<P: AsRef<Path>>(path: P) -> Result<CpinfoFormat> {
        use std::fs::File;
        use std::io::{BufRead as _, BufReader};

        let file = match File::open(path.as_ref()) {
            Ok(opened_file) => opened_file,
            Err(error) => return Err(error.into()),
        };
        let reader = BufReader::new(file);

        let mut is_valid = false;
        let mut version = None;
        let mut first_line_content = String::new();

        // Read the first few lines to detect cpinfo format
        for (line_num, line_result) in reader.lines().enumerate() {
            let line = match line_result {
                Ok(line_content) => line_content,
                Err(error) => return Err(error.into()),
            };

            // Store first line for corruption analysis
            if line_num == 0 {
                first_line_content.clone_from(&line);

                // Check for valid cpinfo header
                if line.contains("Check Point Support Information") {
                    is_valid = true;
                } else {
                    // Check for common corruption patterns
                    if line.to_lowercase().contains("corrupted")
                        || line.contains("CORRUPTED")
                        || line.contains("Random garbage")
                        || !line.chars().all(|character| {
                            return character.is_ascii_graphic() || character.is_whitespace();
                        })
                    {
                        return Err(crate::error::CpinfoError::file_corruption(format!(
                            "Corrupted or malformed cpinfo header detected: '{line}'"
                        )));
                    }

                    // Check if it looks like a cpinfo file with minor corruption
                    if line.to_lowercase().contains("check point")
                        || line.to_lowercase().contains("checkpoint")
                        || line.contains("Support")
                    {
                        return Err(crate::error::CpinfoError::file_corruption(
                            "Invalid cpinfo header format - possibly corrupted file",
                        ));
                    }

                    // Generic invalid format for unrecognized headers
                    return Err(crate::error::CpinfoError::invalid_format(
                        format!("Invalid cpinfo file header: expected 'Check Point Support Information', found '{line}'")
                    ));
                }
            }

            // Look for version information in the first 20 lines
            if line_num < 20 && line.contains("Version:") {
                // Extract version string after "Version:"
                if let Some(version_start) = line.find("Version:") {
                    let version_part = line.get(version_start + 8..).map_or("", |slice| {
                        return slice.trim();
                    });
                    if !version_part.is_empty() {
                        version = Some(version_part.to_owned());
                    }
                }
            }

            // Don't read too far for initial detection
            if line_num > 50 {
                break;
            }
        }

        return Ok(CpinfoFormat::new(
            is_valid,
            version,
            FormatVariant::Standard, // Start with standard format detection
        ));
    }

    /// Create a new format detector
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        return Self;
    }
}

impl Default for FormatDetector {
    /// Create a new format detector with default configuration
    ///
    /// Provides a convenient way to create a format detector with
    /// standard settings suitable for most format detection scenarios.
    ///
    /// # Implementation Notes
    ///
    /// - Equivalent to calling `FormatDetector::new()`
    /// - Uses default detection patterns and rules
    /// - Suitable for general-purpose cpinfo format detection
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}
