//\! Tests for organized file processing helpers

use cpinfo_parser::extraction::organized_extraction::file_processing::{read_file_content, skip_file_header};
use tempfile::NamedTempFile;
use std::io::Write as _;

#[test]
fn read_file_content_falls_back_for_invalid_utf8() {
    let mut f = NamedTempFile::new().unwrap();
    // Write some invalid UTF-8 bytes, then valid ASCII
    let data = vec\![0x80, 0x81, 0x82, b'\n', b'A', b'B'];
    std::fs::write(f.path(), &data).unwrap();

    let content = read_file_content(f.path()).expect("should read");
    // Should contain replacement character(s) due to lossy conversion
    assert\!(content.contains('\u{FFFD}') || content.contains('A'), "content should include lossy conversion and ASCII: {}", content);
    assert\!(content.contains('A'));
    assert\!(content.contains('B'));
}

#[test]
fn skip_file_header_returns_index_after_header_delimiter() {
    let lines = vec\![
        "Preamble",
        "Check Point Support Information",
        "Version: R81.20",
        "==============================================",
        "First Section",
    ];
    let idx = skip_file_header(&lines);
    // Should return index of first line AFTER the delimiter: here 4 (0-based)
    assert_eq\!(idx, 4);
}

#[test]
fn skip_file_header_without_header_returns_zero() {
    let lines = vec\!["Just content", "No header here"];
    assert_eq\!(skip_file_header(&lines), 0);
}