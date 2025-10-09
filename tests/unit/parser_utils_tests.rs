//\! Unit tests for parser::utils module

use cpinfo_parser::parser::utils::{contains_binary_data, get_memory_usage_mb, save_section};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_save_section_basic() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let section_name = "test section";
    let content = "test content";

    let result = save_section(section_name, content, temp_dir.path());
    assert\!(result.is_ok());

    let expected_file = temp_dir.path().join("test_section.txt");
    assert\!(expected_file.exists());

    let saved_content = fs::read_to_string(expected_file).expect("Failed to read saved file");
    assert_eq\!(saved_content, content);
}

#[test]
fn test_save_section_sanitizes_filename() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let section_name = "test/\\:*?\"<>|section";
    let content = "test content";

    let result = save_section(section_name, content, temp_dir.path());
    assert\!(result.is_ok());

    let expected_file = temp_dir.path().join("test_________section.txt");
    assert\!(expected_file.exists());
}

#[test]
fn test_save_section_creates_directory() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let nested_dir = temp_dir.path().join("nested/path");
    let section_name = "section";
    let content = "content";

    let result = save_section(section_name, content, &nested_dir);
    assert\!(result.is_ok());
    assert\!(nested_dir.exists());
}

#[test]
fn test_save_section_empty_content() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let section_name = "empty";
    let content = "";

    let result = save_section(section_name, content, temp_dir.path());
    assert\!(result.is_ok());

    let file = temp_dir.path().join("empty.txt");
    assert\!(file.exists());
    let saved_content = fs::read_to_string(file).expect("Failed to read file");
    assert_eq\!(saved_content, "");
}

#[test]
fn test_save_section_multiline_content() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let section_name = "multiline";
    let content = "line 1\nline 2\nline 3";

    let result = save_section(section_name, content, temp_dir.path());
    assert\!(result.is_ok());

    let file = temp_dir.path().join("multiline.txt");
    let saved_content = fs::read_to_string(file).expect("Failed to read file");
    assert_eq\!(saved_content, content);
}

#[test]
fn test_contains_binary_data_normal_text() {
    assert\!(\!contains_binary_data("normal text"));
    assert\!(\!contains_binary_data("text with spaces"));
    assert\!(\!contains_binary_data("text123"));
}

#[test]
fn test_contains_binary_data_with_tabs() {
    assert\!(\!contains_binary_data("text with\ttabs"));
}

#[test]
fn test_contains_binary_data_with_newlines() {
    assert\!(\!contains_binary_data("text with\nnewlines"));
}

#[test]
fn test_contains_binary_data_with_carriage_return() {
    assert\!(\!contains_binary_data("text with\rcarriage return"));
}

#[test]
fn test_contains_binary_data_control_chars() {
    assert\!(contains_binary_data("text with \u{0001} control chars"));
    assert\!(contains_binary_data("text\x00null"));
    assert\!(contains_binary_data("\x02binary"));
}

#[test]
fn test_contains_binary_data_replacement_char() {
    assert\!(contains_binary_data("text with \u{FFFD} replacement chars"));
}

#[test]
fn test_contains_binary_data_empty() {
    assert\!(\!contains_binary_data(""));
}

#[test]
fn test_contains_binary_data_mixed() {
    assert\!(contains_binary_data("normal\x00binary"));
    assert\!(contains_binary_data("text\x01mixed"));
}

#[test]
fn test_get_memory_usage_mb() {
    let memory = get_memory_usage_mb();
    // In test mode, should return exactly 45.0
    assert_eq\!(memory, 45.0);
}

#[test]
fn test_get_memory_usage_mb_consistency() {
    // Test that it returns consistent values in test mode
    let mem1 = get_memory_usage_mb();
    let mem2 = get_memory_usage_mb();
    assert_eq\!(mem1, mem2);
}