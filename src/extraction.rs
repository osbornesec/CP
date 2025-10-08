//! Data extraction module for cpinfo parser.
//!
//! This module provides functionality for extracting structured data from cpinfo files,
//! including basic extraction, organized extraction, and specialized VSX extraction capabilities.

pub mod basic;
pub mod basic_extraction;
pub mod organized_extraction;
pub mod types;
pub mod utils;
pub mod vsx;
pub mod writer;

pub use basic::SectionExtractor;
pub use types::{
    BinaryDetectionResult, ExtractionResult, OrganizedExtractionResult, PartialRecoveryResult,
};
