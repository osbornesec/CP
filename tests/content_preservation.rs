//! Content preservation tests
//!
//! These tests verify that section content is preserved exactly during extraction

mod common;

use cpinfo_parser::parser::CpinfoParser;
use std::fs;
use std::io::Write;

/// Test 7: Content preservation verification
///
/// This test ensures that extracted content matches the original exactly,
/// including whitespace, special characters, and formatting.
/// This is critical for forensic analysis and compliance.
#[test]
fn test_content_preservation_verification() {
    // Arrange: Create a cpinfo file with various content types to preserve
    let mut temp_file = tempfile::NamedTempFile::with_suffix(".info").unwrap();
    writeln!(temp_file, "Check Point Support Information").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "General Information").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    // Content with various formatting challenges
    writeln!(temp_file, "Version: R81.10").unwrap();
    writeln!(temp_file, "Build:   123456   ").unwrap(); // Extra spaces
    writeln!(temp_file).unwrap(); // Empty line
    writeln!(temp_file, "  Indented line with spaces").unwrap(); // Indentation
    writeln!(temp_file, "Special chars: !@#$%^&*()[]{{}}").unwrap(); // Special characters
    writeln!(temp_file, "Unicode: \u{3b1}\u{3b2}\u{3b3} \u{4e2d}\u{6587} \u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629}").unwrap(); // Unicode content
    writeln!(temp_file, "Trailing whitespace    ").unwrap(); // Trailing whitespace
    writeln!(temp_file, "==============================================").unwrap();
    temp_file.flush().unwrap();

    let file_path = temp_file.path();

    // Store the original content for comparison
    let original_section_content = ["Version: R81.10",
        "Build:   123456   ",
        "",
        "  Indented line with spaces",
        "Special chars: !@#$%^&*()[]{}",
        "Unicode: \u{3b1}\u{3b2}\u{3b3} \u{4e2d}\u{6587} \u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629}",
        "Trailing whitespace    "];

    // Create temporary output directory
    let output_dir = tempfile::tempdir().unwrap();
    let output_path = output_dir.path();

    // Act: Extract sections from the file
    let parser = CpinfoParser::new();
    let result = parser.extract_sections(file_path, output_path);

    // Assert: Content should be preserved exactly
    match result {
        Ok(extraction_result) => {
            assert_eq!(
                extraction_result.sections_extracted, 1,
                "Should extract exactly 1 section"
            );

            // Check that the section file was created
            let section_file = output_path.join("General_Information.txt");
            assert!(section_file.exists(), "Section file should be created");

            // Read the extracted content
            let extracted_content = fs::read_to_string(&section_file).unwrap();
            let extracted_lines: Vec<&str> = extracted_content.lines().collect();

            // Verify content preservation line by line
            assert_eq!(
                extracted_lines.len(),
                original_section_content.len(),
                "Should preserve exact number of lines"
            );

            for (i, (extracted, original)) in extracted_lines
                .iter()
                .zip(original_section_content.iter())
                .enumerate()
            {
                assert_eq!(
                    extracted,
                    original,
                    "Line {} should be preserved exactly. Expected: '{}', Got: '{}'",
                    i + 1,
                    original,
                    extracted
                );
            }

            // Verify specific preservation aspects
            assert!(
                extracted_content.contains("Build:   123456   "),
                "Should preserve extra spaces in content"
            );
            assert!(
                extracted_content.contains("  Indented line with spaces"),
                "Should preserve leading indentation"
            );
            assert!(
                extracted_content.contains("Special chars: !@#$%^&*()[]{}"),
                "Should preserve special characters"
            );
            assert!(
                extracted_content.contains("Unicode: \u{3b1}\u{3b2}\u{3b3} \u{4e2d}\u{6587} \u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629}"),
                "Should preserve Unicode characters"
            );
            assert!(
                extracted_content.contains("Trailing whitespace    "),
                "Should preserve trailing whitespace"
            );

            // Verify empty lines are preserved
            let empty_line_count = extracted_lines
                .iter()
                .filter(|line| line.is_empty())
                .count();
            assert_eq!(empty_line_count, 1, "Should preserve empty lines");
        }
        Err(e) => panic!("Content preservation test should succeed, but got error: {e}"),
    }
}
