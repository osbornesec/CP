//\! Unit tests for extraction::writer module

use cpinfo_parser::extraction::writer::{sanitize_filename, write_section_simple};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_sanitize_filename_basic() {
    assert_eq\!(sanitize_filename("Normal Name"), "Normal_Name");
}

#[test]
fn test_sanitize_filename_path_separators() {
    assert_eq\!(
        sanitize_filename("Path/With\\Separators"),
        "Path_With_Separators"
    );
}

#[test]
fn test_sanitize_filename_special_characters() {
    assert_eq\!(
        sanitize_filename("Special<>|?*\"Chars"),
        "Special______Chars"
    );
}

#[test]
fn test_sanitize_filename_colons() {
    assert_eq\!(sanitize_filename("Colon:In:Name"), "Colon_In_Name");
}

#[test]
fn test_write_section_simple_basic() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let output_file = temp_dir.path().join("test_section.txt");
    let content = "Line 1\nLine 2\nLine 3";

    let result = write_section_simple(content, &output_file);
    assert\!(result.is_ok());

    let result_path = result.unwrap();
    assert\!(result_path.is_some());
    assert_eq\!(result_path.unwrap(), output_file);

    let written_content = fs::read_to_string(&output_file).expect("Failed to read written file");
    assert_eq\!(written_content, content);
}

#[test]
fn test_write_section_simple_empty_content() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let output_file = temp_dir.path().join("empty_section.txt");

    let result = write_section_simple("   \n\n  ", &output_file);
    assert\!(result.is_ok());

    let result_path = result.unwrap();
    assert\!(result_path.is_none());
    assert\!(\!output_file.exists());
}

#[test]
fn test_write_section_simple_whitespace_trimming() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let output_file = temp_dir.path().join("trimmed.txt");
    let content = "  \n\nContent\n\n  ";

    let result = write_section_simple(content, &output_file);
    assert\!(result.is_ok());
    assert\!(result.unwrap().is_some());

    let written = fs::read_to_string(&output_file).expect("Failed to read file");
    assert\!(written.contains("Content"));
}

#[test]
fn test_write_section_simple_multiline() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let output_file = temp_dir.path().join("multiline.txt");
    let content = "Line 1\n\nLine 3\n  Line 4";

    let result = write_section_simple(content, &output_file);
    assert\!(result.is_ok());
    assert\!(result.unwrap().is_some());

    let written = fs::read_to_string(&output_file).expect("Failed to read file");
    assert\!(written.contains("Line 1"));
    assert\!(written.contains("Line 3"));
}

#[test]
fn test_write_section_simple_unicode() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let output_file = temp_dir.path().join("unicode.txt");
    let content = "Hello 世界 🌍";

    let result = write_section_simple(content, &output_file);
    assert\!(result.is_ok());
    assert\!(result.unwrap().is_some());

    let written = fs::read_to_string(&output_file).expect("Failed to read file");
    assert_eq\!(written, content);
}

#[test]
fn test_write_section_simple_creates_parent_dirs() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let nested_path = temp_dir.path().join("nested/dirs/file.txt");
    let content = "Test content";

    let result = write_section_simple(content, &nested_path);
    assert\!(result.is_ok());
    assert\!(nested_path.exists());
}