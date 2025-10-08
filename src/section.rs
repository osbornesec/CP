//! Section parsing and delimiter detection.
//!
//! This module provides comprehensive functionality for parsing and validating
//! sections within `Check Point` `cpinfo` diagnostic files. It includes delimiter
//! detection, section name validation, and pattern recognition capabilities.
//!
//! # Architecture
//!
//! The module is organized into focused sub-modules:
//! - `types`: Core data structures
//! - `validation`: Section name validation logic
//! - `detector`: Delimiter detection functionality
//!
//! # Examples
//!
//! ```rust
//! use cpinfo_parser::section::{DelimiterDetector, SectionValidation};
//!
//! let detector = DelimiterDetector::new();
//! let validation = DelimiterDetector::validate_section_name("System Information");
//!
//! match validation {
//!     SectionValidation::Valid => println!("Valid section name"),
//!     SectionValidation::Invalid(reason) => println!("Invalid: {}", reason),
//! }
//! ```

pub mod detector;
pub mod types;
pub mod validation;

// Re-export main types and functions for convenient access
pub use detector::DelimiterDetector;
pub use types::{SectionDelimiter, SectionValidation};
pub use validation::{validate_section_name, validate_section_name_simple};

// Legacy compatibility methods are now consolidated in the detector module
