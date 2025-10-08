//! Tests for section name validation
//!
//! This module contains comprehensive tests for all validation functionality.

use super::*;

#[test]
fn test_valid_section_names() {
    let valid_names = [
        "System Information",
        "Network Configuration", 
        "Security Settings",
        "Performance Metrics",
        "Log Analysis: Details",
        "Section 1: Overview",
    ];

    for name in &valid_names {
        let result = validate_section_name_simple(name);
        assert!(
            matches!(result, SectionValidation::Valid),
            "Failed for: {}",
            name
        );
    }
}

#[test]
fn test_invalid_section_names() {
    let invalid_names = [
        "",                // Empty
        "ab",              // Too short
        "========",        // Repeated characters
        "| Col1 | Col2 |", // Table formatting
        "===---+++",       // Mixed decorator
        "!@#$%^&*()",      // No meaningful content
    ];

    for name in &invalid_names {
        let result = validate_section_name_simple(name);
        assert!(
            matches!(result, SectionValidation::Invalid(_)),
            "Should fail for: {}",
            name
        );
    }
}

#[test]
fn test_validation_with_debug() {
    // This test ensures debug output doesn't change validation logic
    let test_name = "Valid Section Name";
    let result_no_debug = validate_section_name(test_name, false);
    let result_with_debug = validate_section_name(test_name, true);

    match (result_no_debug, result_with_debug) {
        (SectionValidation::Valid, SectionValidation::Valid) => (),
        (SectionValidation::Invalid(msg1), SectionValidation::Invalid(msg2)) => {
            assert_eq!(msg1, msg2);
        }
        _ => panic!("Debug flag should not change validation result"),
    }
}

#[test]
fn test_basic_constraints() {
    assert!(matches!(
        validate_basic_constraints("", false),
        Some(SectionValidation::Invalid(_))
    ));
    assert!(matches!(
        validate_basic_constraints("ab", false),
        Some(SectionValidation::Invalid(_))
    ));
    assert!(validate_basic_constraints("Valid Name", false).is_none());
}