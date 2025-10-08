//! File validation tests
//!
//! These tests cover the foundation of file validation (Phase 1, Tests 1-5)

mod common;

use common::{create_test_file_invalid_extension, create_valid_cpinfo_file};
use cpinfo_parser::validation::FileValidator;

/// Test 1: Should accept valid cpinfo file with .info extension
///
/// This is our first TDD test - it should FAIL initially because
/// `FileValidator::validate_file` is not implemented yet.
#[test]
fn test_should_accept_valid_cpinfo_file_with_info_extension() {
    // Arrange: Create a valid cpinfo file with .info extension
    let temp_file = create_valid_cpinfo_file();
    let file_path = temp_file.path();

    // Act: Validate the file
    let result = FileValidator::validate_file(file_path);

    // Assert: File should be accepted
    match result {
        Ok(validated_path) => {
            assert_eq!(validated_path.path(), file_path);
            assert!(validated_path.size() > 0, "File should have content");
        }
        Err(e) => panic!("Valid cpinfo file should be accepted, but got error: {e}"),
    }
}

/// Test 2: Should reject non-existent file path
///
/// This test should FAIL initially to confirm our error handling works
#[test]
fn test_should_reject_non_existent_file_path() {
    // Arrange: Use a path that definitely doesn't exist
    let non_existent_path = "/definitely/does/not/exist/file.info";

    // Act: Try to validate the non-existent file
    let result = FileValidator::validate_file(non_existent_path);

    // Assert: Should get FileNotFound error
    match result {
        Err(cpinfo_parser::CpinfoError::FileNotFound { path }) => {
            assert_eq!(path.to_string_lossy(), non_existent_path);
        }
        Err(other_error) => panic!("Expected FileNotFound error, but got: {other_error}"),
        Ok(_) => panic!("Non-existent file should be rejected, but validation succeeded"),
    }
}

/// Test 3: Should reject file without .info extension
///
/// This test should validate that we reject files with wrong extensions
#[test]
fn test_should_reject_file_without_info_extension() {
    // Arrange: Create a file with .txt extension instead of .info
    let temp_file = create_test_file_invalid_extension();
    let file_path = temp_file.path();

    // Act: Try to validate the file with wrong extension
    let result = FileValidator::validate_file(file_path);

    // Assert: Should get InvalidExtension error
    match result {
        Err(cpinfo_parser::CpinfoError::InvalidExtension { path, extension }) => {
            assert_eq!(path, file_path);
            assert_eq!(extension, "txt");
        }
        Err(other_error) => panic!("Expected InvalidExtension error, but got: {other_error}"),
        Ok(_) => panic!("File with wrong extension should be rejected, but validation succeeded"),
    }
}
