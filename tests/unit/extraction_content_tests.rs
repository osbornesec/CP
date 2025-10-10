//\! Unit tests for extraction::content module

use cpinfo_parser::extraction::content::{extract_section_content, find_section_end};

#[test]
fn test_find_section_end_with_delimiter() {
    let lines = vec\![
        "Content line 1",
        "Content line 2",
        "==============================================",
        "Next section",
    ];

    let end = find_section_end(&lines, 0);
    assert_eq\!(end, 2);
}

#[test]
fn test_find_section_end_without_delimiter() {
    let lines = vec\!["Line 1", "Line 2", "Line 3"];

    let end = find_section_end(&lines, 0);
    assert_eq\!(end, 3); // Should return lines.len()
}

#[test]
fn test_find_section_end_from_middle() {
    let lines = vec\![
        "Line 1",
        "Line 2",
        "==============================================",
        "Line 4",
        "Line 5",
        "==============================================",
    ];

    let end = find_section_end(&lines, 3);
    assert_eq\!(end, 5);
}

#[test]
fn test_extract_section_content_normal() {
    let lines = vec\!["Line 1", "Line 2", "", "Line 4", "", ""];

    let content = extract_section_content(&lines, 0, 6);
    assert_eq\!(content, "Line 1\nLine 2\n\nLine 4");
}

#[test]
fn test_extract_section_content_empty() {
    let lines = vec\!["Line 1", "Line 2", "", ""];

    let empty_content = extract_section_content(&lines, 2, 4);
    assert_eq\!(empty_content, "");
}

#[test]
fn test_extract_section_content_invalid_range() {
    let lines = vec\!["Line 1", "Line 2"];

    // Start >= end
    let content = extract_section_content(&lines, 1, 1);
    assert_eq\!(content, "");

    // Start > end
    let content = extract_section_content(&lines, 2, 1);
    assert_eq\!(content, "");

    // Start >= lines.len()
    let content = extract_section_content(&lines, 5, 10);
    assert_eq\!(content, "");
}

#[test]
fn test_extract_section_content_single_line() {
    let lines = vec\!["Single line"];

    let content = extract_section_content(&lines, 0, 1);
    assert_eq\!(content, "Single line");
}

#[test]
fn test_extract_section_content_whitespace_only() {
    let lines = vec\!["", "  ", "\t"];

    let content = extract_section_content(&lines, 0, 3);
    assert_eq\!(content, "");
}

#[test]
fn test_extract_section_content_trailing_whitespace() {
    let lines = vec\!["Line 1", "Line 2", "", "  ", "\t"];

    let content = extract_section_content(&lines, 0, 5);
    assert_eq\!(content, "Line 1\nLine 2");
}

#[test]
fn test_extract_section_content_preserves_internal_blank_lines() {
    let lines = vec\!["Line 1", "", "", "Line 4"];

    let content = extract_section_content(&lines, 0, 4);
    assert_eq\!(content, "Line 1\n\n\nLine 4");
}

#[test]
fn test_find_section_end_at_start() {
    let lines = vec\!["==============================================", "Content"];

    let end = find_section_end(&lines, 0);
    assert_eq\!(end, 0);
}

#[test]
fn test_find_section_end_empty_input() {
    let lines: Vec<&str> = vec\![];

    let end = find_section_end(&lines, 0);
    assert_eq\!(end, 0);
}