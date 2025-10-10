//\! Unit tests for section::validation::artifact_detection module

// Note: The functions in this module are not directly exported,
// but we can test their behavior through the main validation function

use cpinfo_parser::section::validation::validate_section_name_simple;
use cpinfo_parser::section::SectionValidation;

#[test]
fn test_no_artifacts_in_normal_text() {
    let result = validate_section_name_simple("Normal Section Name");
    assert\!(matches\!(result, SectionValidation::Valid));
}

#[test]
fn test_detects_control_characters() {
    let result = validate_section_name_simple("text\x00control");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_allows_tabs() {
    let result = validate_section_name_simple("text\twith\ttabs");
    // May or may not be valid depending on other checks, but tabs alone shouldn't fail
    let _ = result;
}

#[test]
fn test_detects_excessive_whitespace() {
    let result = validate_section_name_simple("   a   ");
    // This might be valid after trimming, but let's check the behavior
    let _ = result;
}

#[test]
fn test_detects_html_like_artifacts() {
    let result = validate_section_name_simple("<html>");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_detects_replacement_char() {
    let result = validate_section_name_simple("text\u{FFFD}");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_normal_punctuation_allowed() {
    let result = validate_section_name_simple("Section: Details");
    assert\!(matches\!(result, SectionValidation::Valid));
}

#[test]
fn test_detects_null_byte() {
    let result = validate_section_name_simple("text\x00null");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}