//! File processing operations for organized extraction
//!
//! This module handles file reading and header processing for cpinfo files,
//! providing utilities for reading file content and skipping header sections.

use std::path::Path;

use tracing::warn;

use crate::error::Result;

/// Read file content with fallback for non-UTF8 files
///
/// This function attempts to read a file as UTF-8 text, falling back to
/// lossy conversion if the file contains invalid UTF-8 sequences.
///
/// # Arguments
///
/// * `path` - Path to the file to read
///
/// # Returns
///
/// File content as a string, with invalid UTF-8 converted to replacement characters
///
/// # Errors
///
/// Returns an error if the file cannot be read from the filesystem
pub fn read_file_content(path: &Path) -> Result<String> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(_) => {
            warn!("Failed to read as UTF-8, using lossy conversion");
            let file_bytes = match std::fs::read(path) {
                Ok(bytes) => bytes,
                Err(e) => return Err(e.into()),
            };
            Ok(String::from_utf8_lossy(&file_bytes).into_owned())
        }
    }
}

/// Skip the file header to get to actual sections
///
/// Finds the first section delimiter after the "Check Point Support Information" header.
/// This allows the parser to skip standard cpinfo file headers and begin processing
/// actual section content.
///
/// # Arguments
///
/// * `lines` - All lines from the cpinfo file
///
/// # Returns
///
/// Index of the first line after the header, or 0 if no header is found
pub fn skip_file_header(lines: &[&str]) -> usize {
    const DELIMITER: &str = "==============================================";

    for (i, line) in lines.iter().enumerate() {
        if line.contains("Check Point Support Information") {
            // Look for the first delimiter after the header
            for j in (i + 1)..lines.len() {
                if lines[j].trim() == DELIMITER {
                    return j + 1;
                }
            }
        }
    }

    0 // No header found, start from beginning
}

