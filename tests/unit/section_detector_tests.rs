//\! Unit tests for section::detector module

use cpinfo_parser::section::DelimiterDetector;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_delimiter_detector_new() {
    let detector = DelimiterDetector::new();
    let _ = detector;
}

#[test]
fn test_delimiter_detector_default() {
    let detector = DelimiterDetector::default();
    let _ = detector;
}

#[test]
fn test_validate_section_name_valid() {
    let result = DelimiterDetector::validate_section_name("Valid Section");
    assert\!(result.is_valid());
}

#[test]
fn test_validate_section_name_invalid() {
    let result = DelimiterDetector::validate_section_name("========");
    assert\!(result.is_invalid());
}

#[test]
fn test_find_valid_sections() -> Result<(), Box<dyn std::error::Error>> {
    let mut temp_file = NamedTempFile::new()?;
    writeln\!(temp_file, "Some content")?;
    writeln\!(temp_file, "==============")?;
    writeln\!(temp_file, "System Information")?;
    writeln\!(temp_file, "More content")?;
    writeln\!(temp_file, "--------------")?;
    writeln\!(temp_file, "Network Details")?;
    writeln\!(temp_file, "End")?;

    let sections = DelimiterDetector::find_valid_sections(temp_file.path())?;

    assert_eq\!(sections.len(), 2);
    assert_eq\!(sections[0].0, "System Information");
    assert_eq\!(sections[1].0, "Network Details");

    Ok(())
}

#[test]
fn test_find_valid_sections_empty_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut temp_file = NamedTempFile::new()?;
    writeln\!(temp_file, "No sections here")?;

    let sections = DelimiterDetector::find_valid_sections(temp_file.path())?;
    assert_eq\!(sections.len(), 0);

    Ok(())
}

#[test]
fn test_find_valid_sections_invalid_names() -> Result<(), Box<dyn std::error::Error>> {
    let mut temp_file = NamedTempFile::new()?;
    writeln\!(temp_file, "==============")?;
    writeln\!(temp_file, "========")?; // Invalid name
    writeln\!(temp_file, "Content")?;

    let sections = DelimiterDetector::find_valid_sections(temp_file.path())?;
    assert_eq\!(sections.len(), 0);

    Ok(())
}

#[test]
fn test_find_valid_sections_multiple_delimiters() -> Result<(), Box<dyn std::error::Error>> {
    let mut temp_file = NamedTempFile::new()?;
    writeln\!(temp_file, "==============================================")?;
    writeln\!(temp_file, "Section One")?;
    writeln\!(temp_file, "==============================================")?;
    writeln\!(temp_file, "Section Two")?;
    writeln\!(temp_file, "==============================================")?;
    writeln\!(temp_file, "Section Three")?;

    let sections = DelimiterDetector::find_valid_sections(temp_file.path())?;
    assert_eq\!(sections.len(), 3);

    Ok(())
}

#[test]
fn test_find_valid_sections_nonexistent_file() {
    let result = DelimiterDetector::find_valid_sections("/nonexistent/file.txt");
    assert\!(result.is_err());
}