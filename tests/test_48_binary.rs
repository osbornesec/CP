//! Test 48: Binary content in text sections handling  
//!
//! Phase 5: Error Handling Test 48 - Canon TDD implementation

use cpinfo_parser::CpinfoParser;
use std::io::Write;
use tempfile::NamedTempFile;

mod common;

/// Test 48: Should handle unexpected binary content in text sections
/// Purpose: Mixed content handling
#[test]
fn test_binary_content_in_text_sections_handling() {
    // Arrange: Create file with binary data in text section
    let mut mixed_content_file = NamedTempFile::with_suffix(".info").unwrap();
    writeln!(mixed_content_file, "Check Point Support Information").unwrap();
    writeln!(
        mixed_content_file,
        "=============================================="
    )
    .unwrap();
    writeln!(mixed_content_file, "System Information").unwrap();
    writeln!(
        mixed_content_file,
        "=============================================="
    )
    .unwrap();
    writeln!(mixed_content_file, "Normal text content").unwrap();
    // Insert binary data
    mixed_content_file
        .write_all(&[0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD])
        .unwrap();
    writeln!(mixed_content_file, "\nMore normal text").unwrap();
    writeln!(
        mixed_content_file,
        "=============================================="
    )
    .unwrap();
    mixed_content_file.flush().unwrap();

    let parser = CpinfoParser::new();
    let output_dir = common::create_temp_output_dir();

    // Act: Attempt to process file with binary content
    let result =
        parser.extract_sections_with_binary_detection(mixed_content_file.path(), output_dir.path());

    // Assert: Should detect binary content and handle appropriately
    match result {
        Ok(extraction_result) => {
            assert!(
                extraction_result.binary_sections_detected > 0,
                "Should detect binary content in sections"
            );
            assert!(
                extraction_result.sections_extracted >= 1,
                "Should still extract valid text sections"
            );
            assert!(
                extraction_result
                    .warnings
                    .iter()
                    .any(|w| w.contains("binary")),
                "Should warn about binary content"
            );
        }
        Err(e) => panic!("Should handle binary content gracefully, but got error: {e}"),
    }
}
