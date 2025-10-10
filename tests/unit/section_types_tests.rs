//\! Unit tests for section::types module

use cpinfo_parser::section::{SectionDelimiter, SectionValidation};

#[test]
fn test_section_delimiter_new() {
    let delimiter = SectionDelimiter::new(42, "===".to_string());
    assert_eq\!(delimiter.line_number, 42);
    assert_eq\!(delimiter.content, "===");
}

#[test]
fn test_section_delimiter_with_long_content() {
    let delimiter = SectionDelimiter::new(1, "==============================================".to_string());
    assert_eq\!(delimiter.line_number, 1);
    assert_eq\!(delimiter.content, "==============================================");
}

#[test]
fn test_section_delimiter_line_zero() {
    let delimiter = SectionDelimiter::new(0, "---".to_string());
    assert_eq\!(delimiter.line_number, 0);
}

#[test]
fn test_section_delimiter_clone() {
    let delimiter1 = SectionDelimiter::new(10, "===".to_string());
    let delimiter2 = delimiter1.clone();
    assert_eq\!(delimiter1, delimiter2);
}

#[test]
fn test_section_validation_valid() {
    let validation = SectionValidation::Valid;
    assert\!(validation.is_valid());
    assert\!(\!validation.is_invalid());
    assert\!(validation.error_message().is_none());
}

#[test]
fn test_section_validation_invalid() {
    let validation = SectionValidation::Invalid("Test error".to_string());
    assert\!(\!validation.is_valid());
    assert\!(validation.is_invalid());
    assert_eq\!(validation.error_message(), Some("Test error"));
}

#[test]
fn test_section_validation_invalid_empty_message() {
    let validation = SectionValidation::Invalid(String::new());
    assert\!(validation.is_invalid());
    assert_eq\!(validation.error_message(), Some(""));
}

#[test]
fn test_section_validation_clone() {
    let validation1 = SectionValidation::Invalid("Error".to_string());
    let validation2 = validation1.clone();
    assert_eq\!(validation1, validation2);
}

#[test]
fn test_section_validation_equality() {
    let valid1 = SectionValidation::Valid;
    let valid2 = SectionValidation::Valid;
    assert_eq\!(valid1, valid2);

    let invalid1 = SectionValidation::Invalid("Error".to_string());
    let invalid2 = SectionValidation::Invalid("Error".to_string());
    assert_eq\!(invalid1, invalid2);

    let invalid3 = SectionValidation::Invalid("Different".to_string());
    assert_ne\!(invalid1, invalid3);
}
