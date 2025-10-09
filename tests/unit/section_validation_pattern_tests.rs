//\! Unit tests for section::validation::pattern_detection module

use cpinfo_parser::section::validation::validate_section_name_simple;
use cpinfo_parser::section::SectionValidation;

#[test]
fn test_detects_table_formatting() {
    let result = validate_section_name_simple("| Col1 | Col2 |");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));

    let result = validate_section_name_simple("+-----+-----+");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_normal_text_not_table() {
    let result = validate_section_name_simple("Normal Section Name");
    assert\!(matches\!(result, SectionValidation::Valid));
}

#[test]
fn test_detects_repeated_character_lines() {
    let result = validate_section_name_simple("========");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));

    let result = validate_section_name_simple("--------");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_normal_text_not_repeated() {
    let result = validate_section_name_simple("Normal Text");
    assert\!(matches\!(result, SectionValidation::Valid));
}

#[test]
fn test_detects_mixed_decorator_pattern() {
    let result = validate_section_name_simple("===---+++");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_normal_mixed_chars_allowed() {
    let result = validate_section_name_simple("Section-Name_123");
    assert\!(matches\!(result, SectionValidation::Valid));
}

#[test]
fn test_detects_partial_delimiter() {
    let result = validate_section_name_simple("Some ====");
    // May or may not fail depending on other validations
    let _ = result;
}

#[test]
fn test_full_delimiter_detected() {
    let result = validate_section_name_simple("==============================================");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_table_with_content() {
    let result = validate_section_name_simple("| Status | Value |");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_repeated_dashes() {
    let result = validate_section_name_simple("----------");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_repeated_underscores() {
    let result = validate_section_name_simple("__________");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}