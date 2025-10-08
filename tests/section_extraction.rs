//! Section extraction tests
//!
//! These tests cover extracting sections from cpinfo files

mod common;

use cpinfo_parser::parser::CpinfoParser;
use std::fs;
use std::io::Write;

/// Test 6: Should extract single section with valid name
///
/// This test should FAIL initially because section extraction is not implemented
#[test]
fn test_should_extract_single_section_with_valid_name() {
    // Arrange: Create a cpinfo file with a single identifiable section
    let mut temp_file = tempfile::NamedTempFile::with_suffix(".info").unwrap();
    writeln!(temp_file, "Check Point Support Information").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "General Information").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "Version: R81.10").unwrap();
    writeln!(temp_file, "Build: 123456").unwrap();
    writeln!(temp_file, "Date: 2024-01-01").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    temp_file.flush().unwrap();

    let file_path = temp_file.path();

    // Create temporary output directory
    let output_dir = tempfile::tempdir().unwrap();
    let output_path = output_dir.path();

    // Act: Extract sections from the file
    let parser = CpinfoParser::new();
    let result = parser.extract_sections(file_path, output_path);

    // Assert: Should successfully extract the section
    match result {
        Ok(extraction_result) => {
            assert_eq!(
                extraction_result.sections_extracted, 1,
                "Should extract exactly 1 section"
            );

            // Check that a file was created for the "General Information" section
            let expected_file = output_path.join("General_Information.txt");
            assert!(
                expected_file.exists(),
                "Section file should be created: {expected_file:?}"
            );

            // Verify the content is preserved
            let content = fs::read_to_string(&expected_file).unwrap();
            assert!(
                content.contains("Version: R81.10"),
                "Should preserve version information"
            );
            assert!(
                content.contains("Build: 123456"),
                "Should preserve build information"
            );
            assert!(
                content.contains("Date: 2024-01-01"),
                "Should preserve date information"
            );
        }
        Err(e) => panic!("Section extraction should succeed, but got error: {e}"),
    }
}
