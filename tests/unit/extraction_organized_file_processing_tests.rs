//\! Unit tests for extraction::organized::file_processing module

use cpinfo_parser::extraction::organized::file_processing::skip_file_header;

#[test]
fn test_skip_file_header_with_header() {
    let lines = vec\![
        "Some preamble",
        "Check Point Support Information",
        "Version: R81.20",
        "==============================================",
        "Section 1",
        "content",
    ];

    let result = skip_file_header(&lines);
    assert_eq\!(result, 4);
}

#[test]
fn test_skip_file_header_no_header() {
    let lines = vec\!["Just content", "No header here"];

    let result = skip_file_header(&lines);
    assert_eq\!(result, 0);
}

#[test]
fn test_skip_file_header_partial_header() {
    let lines = vec\![
        "Check Point Support Information",
        "Version: R81.20",
        // No delimiter found
        "Section 1",
    ];

    let result = skip_file_header(&lines);
    assert_eq\!(result, 0);
}

#[test]
fn test_skip_file_header_empty_input() {
    let lines: Vec<&str> = vec\![];

    let result = skip_file_header(&lines);
    assert_eq\!(result, 0);
}

#[test]
fn test_skip_file_header_only_delimiter() {
    let lines = vec\!["=============================================="];

    let result = skip_file_header(&lines);
    assert_eq\!(result, 0);
}

#[test]
fn test_skip_file_header_delimiter_first() {
    let lines = vec\![
        "==============================================",
        "Section",
    ];

    let result = skip_file_header(&lines);
    assert_eq\!(result, 0);
}

#[test]
fn test_skip_file_header_checkpoint_without_version() {
    let lines = vec\![
        "Check Point Support Information",
        "==============================================",
        "Section",
    ];

    let result = skip_file_header(&lines);
    // Should find the delimiter at index 1, return 2
    assert_eq\!(result, 2);
}

#[test]
fn test_skip_file_header_multiline_preamble() {
    let lines = vec\![
        "Preamble line 1",
        "Preamble line 2",
        "Check Point Support Information",
        "Metadata",
        "==============================================",
        "First Section",
    ];

    let result = skip_file_header(&lines);
    assert_eq\!(result, 5);
}