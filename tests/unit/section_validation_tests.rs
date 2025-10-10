//\! Unit tests for section::validation module

use cpinfo_parser::section::validation::{validate_section_name, validate_section_name_simple};
use cpinfo_parser::section::SectionValidation;

#[test]
fn test_valid_section_names() {
    let valid_names = [
        "System Information",
        "Network Configuration",
        "Security Settings",
        "Performance Metrics",
        "Log Analysis: Details",
        "Section 1: Overview",
        "Database Status",
        "Cluster Information",
    ];

    for name in &valid_names {
        let result = validate_section_name_simple(name);
        assert\!(
            matches\!(result, SectionValidation::Valid),
            "Failed for valid name: {}",
            name
        );
    }
}

#[test]
fn test_invalid_empty_name() {
    let result = validate_section_name_simple("");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_invalid_too_short() {
    let result = validate_section_name_simple("ab");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_invalid_repeated_characters() {
    let result = validate_section_name_simple("========");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_invalid_table_formatting() {
    let result = validate_section_name_simple("| Col1 | Col2 |");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_invalid_mixed_decorator() {
    let result = validate_section_name_simple("===---+++");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_invalid_no_meaningful_content() {
    let result = validate_section_name_simple("\!@#$%^&*()");
    assert\!(matches\!(result, SectionValidation::Invalid(_)));
}

#[test]
fn test_validation_with_debug_flag() {
    let test_name = "Valid Section Name";
    let result_no_debug = validate_section_name(test_name, false);
    let result_with_debug = validate_section_name(test_name, true);

    // Debug flag should not change validation logic
    match (result_no_debug, result_with_debug) {
        (SectionValidation::Valid, SectionValidation::Valid) => (),
        (SectionValidation::Invalid(msg1), SectionValidation::Invalid(msg2)) => {
            assert_eq\!(msg1, msg2);
        }
        _ => panic\!("Debug flag changed validation result"),
    }
}

#[test]
fn test_validation_with_whitespace() {
    let result = validate_section_name_simple("  Valid Name  ");
    assert\!(matches\!(result, SectionValidation::Valid));
}

#[test]
fn test_validation_min_length_boundary() {
    // Exactly 3 characters - minimum valid length
    let result = validate_section_name_simple("abc");
    assert\!(matches\!(result, SectionValidation::Valid));
}

#[test]
fn test_validation_with_numbers() {
    let result = validate_section_name_simple("Section 123");
    assert\!(matches\!(result, SectionValidation::Valid));
}

#[test]
fn test_validation_with_hyphens() {
    let result = validate_section_name_simple("Pre-Production Config");
    assert\!(matches\!(result, SectionValidation::Valid));
}

#[test]
fn test_validation_with_underscores() {
    let result = validate_section_name_simple("system_status");
    assert\!(matches\!(result, SectionValidation::Valid));
}