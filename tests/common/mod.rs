//! Common test utilities for cpinfo parser tests

use std::io::Write as _;
use tempfile::NamedTempFile;

/// Create a test cpinfo file with the given content
pub fn create_test_cpinfo_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::with_suffix(".info").unwrap();
    writeln!(file, "Check Point Support Information").unwrap();
    writeln!(file, "==============================================").unwrap();
    write!(file, "{content}").unwrap();
    file.flush().unwrap();
    return file;
}

/// Create a test cpinfo file with valid header only
pub fn create_valid_cpinfo_file() -> NamedTempFile {
    let mut file = NamedTempFile::with_suffix(".info").unwrap();
    writeln!(file, "Check Point Support Information").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "General Information").unwrap();
    writeln!(file, "==============================================").unwrap();
    writeln!(file, "Version: R81.10").unwrap();
    file.flush().unwrap();
    return file;
}

/// Create a test file with invalid extension
pub fn create_test_file_invalid_extension() -> NamedTempFile {
    let mut file = NamedTempFile::with_suffix(".txt").unwrap();
    writeln!(file, "This is not a cpinfo file").unwrap();
    file.flush().unwrap();
    return file;
}

/// Assert that a section was extracted correctly
#[expect(
    dead_code,
    reason = "Test utility function for section extraction validation"
)]
pub fn assert_section_extracted(
    output_dir: &std::path::Path,
    section_name: &str,
    expected_content: &str,
) {
    let section_file = output_dir.join(format!("{section_name}.txt"));
    assert!(section_file.exists(), "Section file should exist");
    let content = std::fs::read_to_string(section_file).unwrap();
    assert_eq!(content.trim(), expected_content);
}

/// Create a temporary directory for test outputs
pub fn create_temp_output_dir() -> tempfile::TempDir {
    return tempfile::tempdir().unwrap();
}
