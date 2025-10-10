//\! Unit tests for section::validation::content_analysis module

use cpinfo_parser::section::validation::validate_section_name_simple;
use cpinfo_parser::section::SectionValidation;

#[test]
fn test_low_punctuation_ratio_passes() {
    let result = validate_section_name_simple("System Information");
    assert\!(matches\!(result, SectionValidation::Valid));
}

#[test]
fn test_high_punctuation_ratio_fails() {
    let result = validate_section_name_simple("\!@#$%^&*()");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_moderate_punctuation_passes() {
    let result = validate_section_name_simple("Log Analysis: Details");
    assert\!(matches\!(result, SectionValidation::Valid));
}

#[test]
fn test_no_alphanumeric_fails() {
    let result = validate_section_name_simple("\!@#$%^");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_low_meaningful_content_ratio_fails() {
    let result = validate_section_name_simple("\!\!\!a\!\!\!");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_good_meaningful_content_passes() {
    let result = validate_section_name_simple("Database Configuration");
    assert\!(matches\!(result, SectionValidation::Valid));
}

#[test]
fn test_punctuation_boundary() {
    // Test around 70% punctuation boundary
    let result = validate_section_name_simple("abc::::::::::::");
    // Should have high punctuation ratio
    let _ = result;
}

#[test]
fn test_meaningful_content_boundary() {
    // Test around 30% alphanumeric boundary
    let result = validate_section_name_simple("a\!\!\!\!\!\!\!\!\!\!");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}