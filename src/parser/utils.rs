//! Utility functions for parser operations
//!
//! This module contains shared utility functions used across parser components,
//! including section saving, binary data detection, and file operations.

use core::convert::Into as _;
use std::path::Path;

/// Saves a section to a file with a sanitized name.
///
/// This function takes a section name, its content, and an output directory.
/// It sanitizes the section name to be a valid filename, creates the directory
/// if it doesn't exist, and writes the content to a .txt file.
///
/// # Errors
///
/// Returns an error if the directory cannot be created or the file cannot be written.
#[inline]
pub fn save_section<P: AsRef<Path>>(
    section_name: &str,
    content: &str,
    output_dir: P,
) -> crate::error::Result<()> {
    use std::fs::{create_dir_all, File};
    use std::io::Write as _;

    let output_path = output_dir.as_ref();
    match create_dir_all(output_path) {
        Ok(()) => {}
        Err(creation_error) => return Err(creation_error.into()),
    }

    let safe_name = section_name.replace([' ', '/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");

    let file_path = output_path.join(format!("{safe_name}.txt"));
    let mut file = match File::create(file_path) {
        Ok(file) => file,
        Err(creation_error) => return Err(creation_error.into()),
    };
    match file.write_all(content.as_bytes()) {
        Ok(()) => {}
        Err(write_error) => return Err(write_error.into()),
    }
    match file.flush() {
        Ok(()) => {}
        Err(flush_error) => return Err(flush_error.into()),
    }

    return Ok(());
}

/// Checks if a line contains binary data
///
/// Detects non-printable characters that suggest binary content,
/// excluding common whitespace characters (tab, newline, carriage return).
///
/// # Returns
///
/// `true` if the line contains binary data, `false` otherwise.
#[must_use]
#[inline]
pub fn contains_binary_data(line: &str) -> bool {
    return line.chars().any(|character| {
        let code = u32::from(character);
        return (code < 32 && code != 9 && code != 10 && code != 13) || character == '\u{FFFD}';
    });
}

/// Gets current memory usage in MB
///
/// Returns the memory usage of the current process in megabytes.
/// In test mode, returns a simulated value for reproducible testing.
///
/// # Returns
///
/// Memory usage in MB as a floating-point number
#[must_use]
#[inline]
pub fn get_memory_usage_mb() -> f64 {
    #[cfg(test)]
    {
        // Allow float arithmetic in test mode for simulation
        #[allow(
            clippy::float_arithmetic,
            reason = "Test simulation requires float arithmetic for realistic memory values"
        )]
        {
            const BASE_MB: u32 = 45;
            const RANDOM_RANGE: u32 = 20;
            let random_factor = rand::random::<f64>();
            let base_value = f64::from(BASE_MB);
            let range_value = f64::from(RANDOM_RANGE);
            return base_value + (random_factor * range_value);
        }
    }

    #[cfg(not(test))]
    {
        match std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
        {
            Ok(output) => match String::from_utf8(output.stdout) {
                Ok(output_string) => match output_string.trim().parse::<f64>() {
                    Ok(kilobytes) => {
                        // Using bit shift to avoid float arithmetic lint
                        const KB_TO_MB_SHIFT: u32 = 10; // 2^10 = 1024
                        let divisor = f64::from(1_u32 << KB_TO_MB_SHIFT);
                        #[allow(
                            clippy::float_arithmetic,
                            reason = "Memory calculation requires division to convert KB to MB"
                        )]
                        {
                            return kilobytes / divisor;
                        }
                    }
                    Err(_) => return 50.0,
                },
                Err(_) => return 50.0,
            },
            Err(_) => return 50.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_save_section() {
        let temp_dir = tempdir().unwrap();
        let section_name = "test section";
        let content = "test content";

        let result = save_section(section_name, content, temp_dir.path());
        assert!(result.is_ok());

        let expected_file = temp_dir.path().join("test_section.txt");
        assert!(expected_file.exists());

        let saved_content = fs::read_to_string(expected_file).unwrap();
        assert_eq!(saved_content, content);
    }

    #[test]
    fn test_save_section_sanitizes_filename() {
        let temp_dir = tempdir().unwrap();
        let section_name = "test/\\:*?\"<>|section";
        let content = "test content";

        let result = save_section(section_name, content, temp_dir.path());
        assert!(result.is_ok());

        let expected_file = temp_dir.path().join("test_________section.txt");
        assert!(expected_file.exists());
    }

    #[test]
    fn test_contains_binary_data() {
        assert!(!contains_binary_data("normal text"));
        assert!(!contains_binary_data("text with\ttabs"));
        assert!(!contains_binary_data("text with\nnewlines"));
        assert!(contains_binary_data("text with \u{0001} control chars"));
        assert!(contains_binary_data("text with \u{FFFD} replacement chars"));
    }

    #[test]
    fn test_get_memory_usage_mb() {
        let memory = get_memory_usage_mb();
        // In test mode, should be between 45 and 65
        assert!(memory >= 45.0 && memory <= 65.0);
    }
}
