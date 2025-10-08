//! Error types for the `CPInfo` parser

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Main error type for `CPInfo` parsing operations
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum CpinfoError {
    /// Async task join error
    #[error("Async task failed: {message}")]
    AsyncTaskError { message: String },

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    /// File corruption error  
    #[error("File corruption detected: {reason}")]
    FileCorruption { reason: String },

    /// File not found error
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    /// Invalid file extension error
    #[error("Invalid file extension: expected '.info', got '{extension}' for file {path}")]
    InvalidExtension { extension: String, path: PathBuf },

    /// Invalid cpinfo format error
    #[error("Invalid cpinfo format: {reason}")]
    InvalidFormat { reason: String },

    /// I/O error wrapper
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Memory pressure error
    #[error("Memory pressure detected: {message}")]
    MemoryPressure { message: String },

    /// Mutex poisoning error
    #[error("Mutex was poisoned")]
    MutexPoisoned,

    /// Network error with retry information
    #[error("Network error after {attempts} attempts: {message}")]
    NetworkError { attempts: usize, message: String },

    /// Parse error
    #[error("Parse error: {message} at line {line}")]
    ParseError { message: String, line: usize },

    /// Partial processing error
    #[error("Partial processing completed: {message}")]
    PartialProcessing { message: String },

    /// Resource exhaustion error
    #[error("Resource exhaustion: {resource_type} - {message}")]
    ResourceExhaustion {
        message: String,
        resource_type: String,
    },

    /// Security violation error
    #[error("Security violation: {reason}")]
    SecurityViolation { reason: String },

    /// Validation error
    #[error("Validation error: {message}")]
    ValidationError { message: String },
}

/// Result type alias for `CPInfo` operations
pub type Result<T> = core::result::Result<T, CpinfoError>;

impl CpinfoError {
    /// Create a new `AsyncTaskError`
    #[must_use]
    #[inline]
    pub fn async_task_error<S: Into<String>>(message: S) -> Self {
        return Self::AsyncTaskError {
            message: message.into(),
        };
    }

    /// Create a new `ConfigError`
    #[must_use]
    #[inline]
    pub fn config_error<S: Into<String>>(message: S) -> Self {
        return Self::ConfigError {
            message: message.into(),
        };
    }

    /// Create a new `FileCorruption` error
    #[must_use]
    #[inline]
    pub fn file_corruption<S: Into<String>>(reason: S) -> Self {
        return Self::FileCorruption {
            reason: reason.into(),
        };
    }

    /// Create a new `FileNotFound` error
    #[must_use]
    #[inline]
    pub fn file_not_found<P: Into<PathBuf>>(path: P) -> Self {
        return Self::FileNotFound { path: path.into() };
    }

    /// Create a new `InvalidExtension` error
    #[must_use]
    #[inline]
    pub fn invalid_extension<P: Into<PathBuf>>(path: P, extension: String) -> Self {
        return Self::InvalidExtension {
            extension,
            path: path.into(),
        };
    }

    /// Create a new `InvalidFormat` error
    #[must_use]
    #[inline]
    pub fn invalid_format<S: Into<String>>(reason: S) -> Self {
        return Self::InvalidFormat {
            reason: reason.into(),
        };
    }

    /// Create a new `MemoryPressure` error
    #[must_use]
    #[inline]
    pub fn memory_pressure<S: Into<String>>(message: S) -> Self {
        return Self::MemoryPressure {
            message: message.into(),
        };
    }

    /// Create a new `MutexPoisoned` error
    #[must_use]
    #[inline]
    pub const fn mutex_poisoned() -> Self {
        return Self::MutexPoisoned;
    }

    /// Create a new `NetworkError`
    #[must_use]
    #[inline]
    pub fn network_error<S: Into<String>>(attempts: usize, message: S) -> Self {
        return Self::NetworkError {
            attempts,
            message: message.into(),
        };
    }

    /// Create a new `ParseError`
    #[must_use]
    #[inline]
    pub fn parse_error<S: Into<String>>(message: S, line: usize) -> Self {
        return Self::ParseError {
            line,
            message: message.into(),
        };
    }

    /// Create a new `PartialProcessing` error
    #[must_use]
    #[inline]
    pub fn partial_processing<S: Into<String>>(message: S) -> Self {
        return Self::PartialProcessing {
            message: message.into(),
        };
    }

    /// Create a new `ResourceExhaustion` error
    #[must_use]
    #[inline]
    pub fn resource_exhaustion<S: Into<String>>(resource_type: S, message: S) -> Self {
        return Self::ResourceExhaustion {
            message: message.into(),
            resource_type: resource_type.into(),
        };
    }

    /// Create a new `SecurityViolation` error
    #[must_use]
    #[inline]
    pub fn security_violation<S: Into<String>>(reason: S) -> Self {
        return Self::SecurityViolation {
            reason: reason.into(),
        };
    }

    /// Create a new `ValidationError`
    #[must_use]
    #[inline]
    pub fn validation_error<S: Into<String>>(message: S) -> Self {
        return Self::ValidationError {
            message: message.into(),
        };
    }
}

/// Convert tokio `JoinError` to `CpinfoError`
impl From<tokio::task::JoinError> for CpinfoError {
    #[inline]
    fn from(error: tokio::task::JoinError) -> Self {
        return Self::AsyncTaskError {
            message: error.to_string(),
        };
    }
}
