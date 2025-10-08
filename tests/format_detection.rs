//! Format detection tests
//!
//! These tests cover cpinfo file format validation and header detection

mod common;

use common::create_valid_cpinfo_file;
use cpinfo_parser::parser::CpinfoParser;
use std::io::Write;

/// Test 4: Should detect cpinfo file format correctly
///
/// This test should FAIL initially because format detection is not implemented
#[test]
fn test_should_detect_cpinfo_file_format_correctly() {
    // Arrange: Create a valid cpinfo file with proper header
    let temp_file = create_valid_cpinfo_file();
    let file_path = temp_file.path();

    // Act: Parse the file and detect format
    let parser = CpinfoParser::new();
    let result = parser.detect_format(file_path);

    // Assert: Should detect valid cpinfo format
    match result {
        Ok(format_info) => {
            assert!(
                format_info.is_valid_cpinfo(),
                "Should be detected as valid cpinfo format"
            );
            assert!(
                format_info.version().is_some(),
                "Should detect version information"
            );
        }
        Err(e) => panic!("Valid cpinfo file format should be detected, but got error: {e}"),
    }
}

/// Test 5: Should identify section delimiter pattern  
///
/// This test validates section delimiter recognition
#[test]
fn test_should_identify_section_delimiter_pattern() {
    // Arrange: Create a test file with section delimiters
    let mut temp_file = tempfile::NamedTempFile::with_suffix(".info").unwrap();
    writeln!(temp_file, "Check Point Support Information").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "General Information").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "Version: R81.10").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "Network Information").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    temp_file.flush().unwrap();

    let file_path = temp_file.path();

    // Act: Parse and identify delimiters
    let parser = CpinfoParser::new();
    let result = parser.identify_section_delimiters(file_path);

    // Assert: Should find section delimiters
    match result {
        Ok(delimiters) => {
            assert!(
                delimiters.len() >= 3,
                "Should find at least 3 section delimiters"
            );
            // Verify they are at expected line positions
            assert_eq!(delimiters[0].line_number, 2); // First delimiter after header
            assert_eq!(delimiters[1].line_number, 4); // Second delimiter after General Info
            assert_eq!(delimiters[2].line_number, 6); // Third delimiter after version
        }
        Err(e) => panic!("Should identify section delimiters, but got error: {e}"),
    }
}
