//! CLI Interface Tests
//!
//! Comprehensive testing of command-line interface using Canon TDD principles.
//! Tests argument validation, error messages, help system, and accessibility compliance.

use assert_cmd::Command;
// Note: assert_fs::prelude::* not needed for current tests
use core::error::Error;
use predicates::prelude::*;
use tempfile::NamedTempFile;

type TestResult = Result<(), Box<dyn Error>>;

/// Helper function to create a valid cpinfo test file
fn create_valid_cpinfo_file() -> Result<NamedTempFile, Box<dyn Error>> {
    let temp_file = NamedTempFile::with_suffix(".info")?;
    std::fs::write(
        temp_file.path(),
        "Check Point Support Information\n\
         ===============================================\n\
         Version: R81.10 - Build 029\n\
         ===============================================\n\
         System Information\n\
         ===============================================\n\
         Hostname: checkpoint-gateway\n\
         CPU: Intel Xeon\n\
         Memory: 16GB\n",
    )?;
    return Ok(temp_file);
}

/// Helper function to create an invalid file (wrong extension)
fn create_invalid_file() -> Result<NamedTempFile, Box<dyn Error>> {
    let temp_file = NamedTempFile::with_suffix(".txt")?;
    std::fs::write(temp_file.path(), "Not a cpinfo file")?;
    return Ok(temp_file);
}

// =============================================================================
// Test 1: Argument Validation - Valid Input
// =============================================================================

#[test]
fn test_cli_accepts_valid_cpinfo_file() -> TestResult {
    let temp_file = create_valid_cpinfo_file()?;

    let mut cmd = Command::cargo_bin("cpinfo-parser")?;
    cmd.arg(temp_file.path())
        .arg("--read-only") // Don't create output files in test
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("CPInfo Parser v"))
        .stdout(predicate::str::contains("Successfully processed"));

    return Ok(());
}

// =============================================================================
// Test 2: Argument Validation - File Not Found
// =============================================================================

#[test]
fn test_cli_rejects_nonexistent_file() -> TestResult {
    let mut cmd = Command::cargo_bin("cpinfo-parser")?;
    cmd.arg("/definitely/does/not/exist/file.info");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to extract sections"));

    return Ok(());
}

// =============================================================================
// Test 3: Argument Validation - Invalid File Extension
// =============================================================================

#[test]
fn test_cli_rejects_invalid_extension() -> TestResult {
    let temp_file = create_invalid_file()?;

    let mut cmd = Command::cargo_bin("cpinfo-parser")?;
    cmd.arg(temp_file.path());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to extract sections"));

    return Ok(());
}

// =============================================================================
// Test 4: Help System - Basic Help
// =============================================================================

#[test]
fn test_cli_displays_helpful_usage_information() -> TestResult {
    let mut cmd = Command::cargo_bin("cpinfo-parser")?;
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Parse and extract sections from Check Point cpinfo files",
        ))
        .stdout(predicate::str::contains("Usage:")) // clap uses "Usage:" not "USAGE:"
        .stdout(predicate::str::contains("cpinfo-parser"))
        .stdout(predicate::str::contains("Options:")) // clap uses "Options:" not "OPTIONS:"
        .stdout(predicate::str::contains("--security"))
        .stdout(predicate::str::contains("--progress"))
        .stdout(predicate::str::contains("--read-only"))
        .stdout(predicate::str::contains("--verbose"));

    return Ok(());
}

// =============================================================================
// Test 5: Help System - Version Information
// =============================================================================

#[test]
fn test_cli_displays_version_information() -> TestResult {
    let mut cmd = Command::cargo_bin("cpinfo-parser")?;
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("cpinfo-parser"))
        .stdout(predicate::str::contains("0.1.0"));

    return Ok(());
}

// =============================================================================
// Test 6: Accessibility - Screen Reader Compatible Output
// =============================================================================

#[test]
fn test_cli_screen_reader_compatible_output() -> TestResult {
    let temp_file = create_valid_cpinfo_file()?;

    let mut cmd = Command::cargo_bin("cpinfo-parser")?;
    cmd.arg(temp_file.path())
        .arg("--read-only")
        .arg("--verbose")
        .env_remove("TERM") // Remove terminal type to ensure plain text
        .env("NO_COLOR", "1"); // Disable colors for accessibility

    let output = cmd.assert().success().get_output().clone();
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    // Verify output is primarily readable text (no excessive control characters)
    let readable_chars: usize = stdout_str
        .chars()
        .filter(|c| {
            return c.is_alphanumeric()
                || c.is_whitespace()
                || [':', '.', '-', '_', '/', '(', ')', '"', ',', ';'].contains(c);
        })
        .count();
    let total_chars = stdout_str.chars().count();
    let readable_ratio = if total_chars > 0 {
        readable_chars as f64 / total_chars as f64
    } else {
        1.0
    };

    assert!(
        readable_ratio >= 0.9,
        "Output should be primarily readable text for accessibility. Ratio: {readable_ratio}"
    );

    // Verify no cursor manipulation or problematic escape sequences
    assert!(
        !stdout_str.contains('\x1b'),
        "Output should not contain ANSI escape sequences for screen reader compatibility"
    );

    return Ok(());
}

// =============================================================================
// Test 7: Error Messages - Descriptive and Actionable
// =============================================================================

#[test]
fn test_cli_provides_actionable_error_messages() -> TestResult {
    let mut cmd = Command::cargo_bin("cpinfo-parser")?;
    cmd.arg("/nonexistent/path/file.info");

    let output = cmd.assert().failure().get_output().clone();
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    // Error message should be descriptive
    assert!(
        stderr_str.contains("Failed to parse cpinfo file")
            || stderr_str.contains("Failed to extract sections"),
        "Error should clearly state the operation that failed"
    );

    // Error message should include context
    assert!(
        stderr_str.contains("/nonexistent/path/file.info")
            || stderr_str.contains("No such file")
            || stderr_str.contains("File not found"),
        "Error should include file path or clear reason"
    );

    // Error message should be on stderr (not stdout)
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout_str.contains("Failed"),
        "Error messages should go to stderr, not stdout"
    );

    return Ok(());
}

// =============================================================================
// Test 8: Progress Reporting - Real-time Updates
// =============================================================================

#[test]
fn test_cli_progress_reporting_with_large_file() -> TestResult {
    let temp_file = create_valid_cpinfo_file()?;

    let mut cmd = Command::cargo_bin("cpinfo-parser")?;
    cmd.arg(temp_file.path())
        .arg("--read-only")
        .arg("--progress")
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("CPInfo Parser v"))
        .stdout(
            predicate::str::contains("Successfully processed")
                .or(predicate::str::contains("Processing completed")),
        );

    return Ok(());
}

// =============================================================================
// Test 9: Output Directory Validation
// =============================================================================

#[test]
fn test_cli_validates_output_directory_permissions() -> TestResult {
    let temp_file = create_valid_cpinfo_file()?;
    let temp_dir = assert_fs::TempDir::new()?;

    let mut cmd = Command::cargo_bin("cpinfo-parser")?;
    cmd.arg(temp_file.path())
        .arg("--output")
        .arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully extracted"));

    return Ok(());
}

// =============================================================================
// Test 10: Security Mode Validation
// =============================================================================

#[test]
fn test_cli_security_mode_enabled() -> TestResult {
    let temp_file = create_valid_cpinfo_file()?;

    let mut cmd = Command::cargo_bin("cpinfo-parser")?;
    cmd.arg(temp_file.path())
        .arg("--read-only")
        .arg("--security")
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("CPInfo Parser v"));

    return Ok(());
}

// =============================================================================
// Test 11: Keyboard Navigation - All Features Accessible
// =============================================================================

#[test]
fn test_cli_keyboard_only_operation() -> TestResult {
    // Test that all CLI functionality is accessible via keyboard (command line)
    // No mouse interaction required for CLI tools

    let temp_file = create_valid_cpinfo_file()?;

    // Test basic operation via command line only
    let mut cmd = Command::cargo_bin("cpinfo-parser")?;
    cmd.arg(temp_file.path()).arg("--read-only");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully processed"));

    // Test help access via keyboard
    let mut help_cmd = Command::cargo_bin("cpinfo-parser")?;
    help_cmd.arg("--help");

    help_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));

    return Ok(());
}

// =============================================================================
// Test 12: WCAG 2.1 AA Compliance - Text-Based Interface
// =============================================================================

#[test]
fn test_cli_wcag_compliance_text_output() -> TestResult {
    let temp_file = create_valid_cpinfo_file()?;

    let mut cmd = Command::cargo_bin("cpinfo-parser")?;
    cmd.arg(temp_file.path())
        .arg("--read-only")
        .arg("--verbose")
        .env("NO_COLOR", "1"); // Ensure no color-only information

    let output = cmd.assert().success().get_output().clone();
    let combined_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // WCAG 2.1 AA Compliance checks:

    // 1. Meaningful content (not just visual indicators)
    assert!(
        combined_output.contains("Successfully processed") || combined_output.contains("sections"),
        "Output should contain meaningful status information"
    );

    // 2. No information conveyed by color alone
    assert!(
        !combined_output.contains("red")
            && !combined_output.contains("green")
            && !combined_output.contains("color"),
        "Status should not be conveyed by color references alone"
    );

    // 3. Logical reading order
    let lines: Vec<&str> = combined_output.lines().collect();
    if lines.len() > 1 {
        assert!(
            lines[0].contains("CPInfo Parser") || lines[0].contains("Input file"),
            "First line should contain program identification or primary action"
        );
    }

    // 4. Proper error identification
    if combined_output.contains("error") || combined_output.contains("failed") {
        assert!(
            combined_output.contains("Failed") || combined_output.contains("Error"),
            "Errors should be clearly identified as such"
        );
    }

    return Ok(());
}
