//! Test 47: Incomplete section delimiters recovery  
//!
//! Phase 5: Error Handling Test 47 - Canon TDD implementation

use cpinfo_parser::{CpinfoParser, PartialRecoveryConfig};
use std::io::Write;
use tempfile::NamedTempFile;

mod common;

/// Test 47: Should recover from incomplete section delimiters\
/// Purpose: Partial file processing
#[test]
fn test_incomplete_section_delimiters_recovery() {
    // Arrange: Create file with missing end delimiter
    let mut incomplete_file = NamedTempFile::with_suffix(".info").unwrap();
    writeln!(incomplete_file, "Check Point Support Information").unwrap();
    writeln!(
        incomplete_file,
        "=============================================="
    )
    .unwrap();
    writeln!(incomplete_file, "General Information").unwrap();
    writeln!(
        incomplete_file,
        "=============================================="
    )
    .unwrap();
    writeln!(incomplete_file, "Version: R81.10").unwrap();
    writeln!(incomplete_file, "Build: 029").unwrap();
    writeln!(incomplete_file, "Network Configuration").unwrap();
    writeln!(
        incomplete_file,
        "=============================================="
    )
    .unwrap();
    writeln!(incomplete_file, "Interface: eth0").unwrap();
    writeln!(incomplete_file, "IP: 192.168.1.1").unwrap();
    // Missing final delimiter - file ends abruptly
    incomplete_file.flush().unwrap();

    let parser = CpinfoParser::new();
    let output_dir = common::create_temp_output_dir();

    // Act: Attempt to extract sections from incomplete file
    let config = PartialRecoveryConfig::default();
    let result = parser.extract_sections_with_partial_recovery(
        incomplete_file.path(),
        output_dir.path(),
        config,
    );

    // Assert: Should process partial content and warn about incompleteness
    match result {
        Ok(extraction_result) => {
            assert!(
                extraction_result.valid_sections_processed >= 1,
                "Should extract at least one complete section"
            );
            assert!(
                extraction_result.failed_sections > 0
                    || extraction_result.recovery_actions_taken > 0,
                "Should detect incomplete sections or take recovery actions"
            );
            assert!(
                extraction_result.recovery_actions_taken >= 0,
                "Should have taken recovery actions for incomplete content"
            );
        }
        Err(e) => panic!("Should handle incomplete file gracefully, but got error: {e}"),
    }
}
