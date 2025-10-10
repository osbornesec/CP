//! Utility functions for parser operations
//!
//! This module contains shared utility functions used across parser components,
//! including section saving, binary data detection, and file operations.

use core::convert::Into as _;
use std::path::Path;

#[cfg(test)]
use crate::parser::monitoring::monitoring_config::DEFAULT_TEST_MEMORY_BASE;

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

/// Gets the current process memory usage in megabytes.
///
/// In test builds this returns the shared `DEFAULT_TEST_MEMORY_BASE`. If the function
/// cannot determine the memory usage at runtime, it falls back to 50.0 MB.
///
/// # Returns
///
/// Memory usage of the current process in megabytes.
///
/// # Examples
///
/// ```
/// use cpinfo_parser::parser::utils::get_memory_usage_mb;
///
/// let mb = get_memory_usage_mb();
/// assert!(mb > 0.0);
/// ```
#[must_use]
#[inline]
#[allow(
    clippy::missing_const_for_fn,
    reason = "Function invokes system utilities at runtime in non-test builds"
)]
pub fn get_memory_usage_mb() -> f64 {
    #[cfg(test)]
    {
        return DEFAULT_TEST_MEMORY_BASE;
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
