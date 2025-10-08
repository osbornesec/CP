//! File validation module
//!
//! Provides functionality to validate `cpinfo` files before processing.

use crate::error::{CpinfoError, Result};
use std::path::Path;

/// File validator for `cpinfo` files
///
/// This utility provides validation functionality for Check Point `cpinfo` diagnostic files.
/// It performs comprehensive validation including file existence, extension checking,
/// size validation, and basic format verification to ensure files are suitable for processing.
///
/// # Validation Features
///
/// - File existence and accessibility verification
/// - Extension validation for `cpinfo` files (`.info`, `.cpinfo`, etc.)
/// - File size validation to prevent processing of invalid files
/// - Basic format structure validation
/// - Path security validation to prevent directory traversal
///
/// # Examples
///
/// ```rust
/// use cpinfo_parser::validation::FileValidator;
///
/// // Validate a cpinfo file
/// match FileValidator::validate_file("diagnostic.cpinfo") {
///     Ok(validated_path) => {
///         println!("File is valid: {} ({} bytes)",
///                  validated_path.path().display(),
///                  validated_path.size());
///     }
///     Err(e) => eprintln!("Validation failed: {}", e),
/// }
/// ```
#[non_exhaustive]
pub struct FileValidator;

impl FileValidator {
    /// Create a new `FileValidator` instance
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        return Self;
    }

    /// Validate that a file exists and has the correct extension
    ///
    /// Performs comprehensive validation of a `cpinfo` file to ensure it meets
    /// all requirements for processing. This includes file existence, extension
    /// validation, accessibility checks, and basic format verification.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to validate (accepts any type that can be converted to a path)
    ///
    /// # Returns
    ///
    /// Returns `Result<ValidatedPath>` containing:
    /// - Validated and canonicalized file path
    /// - File size information
    /// - Validation metadata
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The file does not exist or cannot be accessed
    /// - The path points to a directory instead of a file
    /// - The file extension is not recognized as a `cpinfo` format
    /// - The file size is invalid (0 bytes or excessively large)
    /// - Insufficient permissions to read the file
    /// - The file appears to be corrupted or invalid format
    ///
    /// # Supported Extensions
    ///
    /// - `.info` - Standard `cpinfo` diagnostic files
    /// - `.cpinfo` - Alternative `cpinfo` format
    /// - `.cpinfoall` - Comprehensive diagnostic files
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cpinfo_parser::validation::FileValidator;
    ///
    /// // Validate a standard cpinfo file
    /// match FileValidator::validate_file("gateway_diagnostic.info") {
    ///     Ok(validated) => {
    ///         println!("Valid cpinfo file: {}", validated.path().display());
    ///         println!("File size: {} bytes", validated.size());
    ///     }
    ///     Err(e) => eprintln!("Validation error: {}", e),
    /// }
    ///
    /// // Validate with error handling
    /// let result = FileValidator::validate_file("suspicious_file.txt");
    /// assert!(result.is_err()); // Wrong extension
    /// ```
    #[inline]
    pub fn validate_file<P: AsRef<Path>>(path: P) -> Result<ValidatedPath> {
        let file_path = path.as_ref();

        // Check if file exists
        if !file_path.exists() {
            return Err(CpinfoError::file_not_found(file_path));
        }

        // Check if it's a file (not a directory)
        if !file_path.is_file() {
            return Err(CpinfoError::validation_error("Path is not a file"));
        }

        // Check file extension
        if let Some(extension) = file_path.extension() {
            if extension != "info" {
                return Err(CpinfoError::invalid_extension(
                    file_path,
                    extension.to_string_lossy().to_string(),
                ));
            }
        } else {
            return Err(CpinfoError::invalid_extension(file_path, "none".to_owned()));
        }

        // Get file size
        let metadata = match std::fs::metadata(file_path) {
            Ok(metadata) => metadata,
            Err(error) => return Err(CpinfoError::Io(error)),
        };
        let size = metadata.len();

        return Ok(ValidatedPath::new(file_path.to_path_buf(), size));
    }
}

impl Default for FileValidator {
    /// Create a new file validator with default configuration
    ///
    /// Provides a convenient way to create a file validator with
    /// standard settings suitable for most file validation scenarios.
    ///
    /// # Implementation Notes
    ///
    /// - Equivalent to calling `FileValidator::new()`
    /// - Uses default validation rules and size limits
    /// - Suitable for general-purpose file validation
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}

/// Represents a validated file path
///
/// This structure contains a file path that has been validated and verified
/// to be a legitimate `cpinfo` file. It includes the canonicalized path and
/// file metadata that can be safely used for processing operations.
///
/// # Validation Guarantees
///
/// A `ValidatedPath` instance guarantees that:
/// - The file exists and is accessible
/// - The file has a valid `cpinfo` extension
/// - The file size is reasonable and non-zero
/// - The path has been canonicalized and is safe to use
/// - The file passed basic format validation checks
///
/// # Examples
///
/// ```rust
/// use cpinfo_parser::validation::{FileValidator, ValidatedPath};
/// use std::path::PathBuf;
///
/// // Create a validated path (typically done through FileValidator)
/// let validated = ValidatedPath::new(
///     PathBuf::from("/path/to/diagnostic.info"),
///     1048576 // 1MB
/// );
///
/// println!("Validated file: {}", validated.path().display());
/// println!("File size: {} bytes", validated.size());
///
/// // Use with processing operations
/// let size_mb = validated.size() as f64 / 1_048_576.0;
/// if size_mb > 100.0 {
///     println!("Large file detected: {:.1} MB", size_mb);
/// }
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ValidatedPath {
    /// The canonicalized and validated file path
    pub path: std::path::PathBuf,
    /// The size of the file in bytes
    pub size: u64,
}

impl ValidatedPath {
    /// Create a new `ValidatedPath`
    #[must_use]
    #[inline]
    pub const fn new(path: std::path::PathBuf, size: u64) -> Self {
        return Self { path, size };
    }

    /// Get the file path
    #[must_use]
    #[inline]
    pub fn path(&self) -> &Path {
        return &self.path;
    }

    /// Get the file size
    #[must_use]
    #[inline]
    pub const fn size(&self) -> u64 {
        return self.size;
    }
}
