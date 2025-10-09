//\! Unit tests for parser::binary_extraction module

use cpinfo_parser::parser::binary_extraction::extract_sections_with_binary_detection;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_extract_sections_with_binary_detection_basic() {
    let temp_input_dir = tempdir().expect("Failed to create temp input directory");
    let temp_output_dir = tempdir().expect("Failed to create temp output directory");

    let input_file = temp_input_dir.path().join("test.cpinfo");
    let test_content = "Check Point Support Information\n\n==============================================\nTest Section 1\n==============================================\nThis is normal content\nSome more normal text\n\n==============================================\n";

    fs::write(&input_file, test_content).expect("Failed to write test file");

    let result = extract_sections_with_binary_detection(&input_file, temp_output_dir.path());
    assert\!(result.is_ok());

    let extraction_result = result.unwrap();
    assert_eq\!(extraction_result.sections_extracted, 1);
    assert_eq\!(extraction_result.section_files.len(), 1);
    assert_eq\!(extraction_result.output_directory, temp_output_dir.path());
}

#[test]
fn test_extract_sections_with_binary_detection_multiple() {
    let temp_input_dir = tempdir().expect("Failed to create temp input directory");
    let temp_output_dir = tempdir().expect("Failed to create temp output directory");

    let input_file = temp_input_dir.path().join("test.cpinfo");
    let test_content = "==============================================\nSection 1\n==============================================\nContent 1\n\n==============================================\nSection 2\n==============================================\nContent 2\n==============================================\n";

    fs::write(&input_file, test_content).expect("Failed to write test file");

    let result = extract_sections_with_binary_detection(&input_file, temp_output_dir.path());
    assert\!(result.is_ok());

    let extraction_result = result.unwrap();
    assert_eq\!(extraction_result.sections_extracted, 2);
}

#[test]
fn test_extract_sections_with_binary_detection_empty() {
    let temp_input_dir = tempdir().expect("Failed to create temp input directory");
    let temp_output_dir = tempdir().expect("Failed to create temp output directory");

    let input_file = temp_input_dir.path().join("empty.cpinfo");
    fs::write(&input_file, "No sections").expect("Failed to write test file");

    let result = extract_sections_with_binary_detection(&input_file, temp_output_dir.path());
    assert\!(result.is_ok());

    let extraction_result = result.unwrap();
    assert_eq\!(extraction_result.sections_extracted, 0);
}

#[test]
fn test_extract_sections_with_binary_detection_nonexistent() {
    let temp_output_dir = tempdir().expect("Failed to create temp output directory");
    let nonexistent_file = std::path::PathBuf::from("/nonexistent/file.cpinfo");

    let result = extract_sections_with_binary_detection(&nonexistent_file, temp_output_dir.path());
    assert\!(result.is_err());
}

#[test]
fn test_extract_sections_with_binary_detection_creates_output_dir() {
    let temp_input_dir = tempdir().expect("Failed to create temp input directory");
    let temp_base = tempdir().expect("Failed to create temp base directory");

    let input_file = temp_input_dir.path().join("test.cpinfo");
    let output_dir = temp_base.path().join("new_output");

    fs::write(&input_file, "==============================================\nTest\n==============================================\nContent\n==============================================\n")
        .expect("Failed to write test file");

    assert\!(\!output_dir.exists());

    let result = extract_sections_with_binary_detection(&input_file, &output_dir);
    assert\!(result.is_ok());
    assert\!(output_dir.exists());
}