//! Test 46: File corruption detection and graceful handling
//!
//! Phase 5: Error Handling Test 46 - Canon TDD implementation

use cpinfo_parser::{CpinfoError, CpinfoParser};
use std::io::Write as _;
use tempfile::NamedTempFile;

/// Test 46: Should handle corrupted cpinfo file headers gracefully
/// Purpose: Robustness against corruption
#[test]
fn test_corrupted_cpinfo_file_headers_graceful_handling() {
    // Arrange: Create a file with malformed header
    let mut corrupted_file = NamedTempFile::with_suffix(".info").unwrap();
    writeln!(corrupted_file, "CORRUPTED HEADER - NOT VALID CPINFO").unwrap();
    writeln!(corrupted_file, "Random garbage data").unwrap();
    writeln!(
        corrupted_file,
        "=============================================="
    )
    .unwrap();
    writeln!(corrupted_file, "Some content section").unwrap();
    writeln!(
        corrupted_file,
        "=============================================="
    )
    .unwrap();
    corrupted_file.flush().unwrap();

    let parser = CpinfoParser::new();

    // Act: Attempt to parse the corrupted file
    let result = parser.detect_format(corrupted_file.path());

    // Assert: Should get clear error message without crash
    match result {
        Err(CpinfoError::FileCorruption { reason }) => {
            assert!(
                reason.contains("CORRUPTED HEADER - NOT VALID CPINFO"),
                "Error message should mention corrupted header: {reason}"
            );
        }
        Err(other_error) => panic!("Expected FileCorruption error, but got: {other_error}"),
        Ok(_) => panic!("Corrupted file should be rejected, but parsing succeeded"),
    }
}
