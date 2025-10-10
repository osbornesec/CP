//\! Tests for parser::utils helpers

use tempfile::tempdir;

use cpinfo_parser::parser::utils::{save_section, contains_binary_data, get_memory_usage_mb};

#[test]
fn save_section_creates_sanitized_file() {
    let dir = tempdir().unwrap();
    let name = r#"test/\:*?"<>|section"#;
    let content = "test content";

    save_section(name, content, dir.path()).expect("save should succeed");

    // Backslash and other characters replaced with underscore
    let expected = dir.path().join("test_________section.txt");
    assert\!(expected.exists(), "expected file should exist: {:?}", expected);

    let read_back = std::fs::read_to_string(expected).unwrap();
    assert_eq\!(read_back, content);
}

#[test]
fn contains_binary_data_detects_controls() {
    assert\!(\!contains_binary_data("normal text"));
    assert\!(\!contains_binary_data("text with\ttab and \nnewline"));
    assert\!(contains_binary_data("has control \u{0001}"));
    assert\!(contains_binary_data("has replacement \u{FFFD}"));
}

#[test]
fn get_memory_usage_mb_returns_test_value_under_cfg_test() {
    // In test configuration, this function returns a deterministic value
    let mem = get_memory_usage_mb();
    assert_eq\!(mem, 45.0);
}