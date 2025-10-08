//! UI and CLI Interface Tests - Phase 6 (Tests 55-64)
//!
//! Tests for user interface and accessibility implementation following Canon TDD

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;

/// Test 55: Command-line interface structure and argument parsing
#[test]
fn test_55_cli_structure_and_argument_parsing() {
    // Test basic command structure
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--help");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify CLI structure elements
    assert!(stdout.contains("cpinfo-parser"), "Should show program name");
    assert!(stdout.contains("Usage:"), "Should show usage syntax");
    assert!(
        stdout.contains("Arguments:"),
        "Should show arguments section"
    );
    assert!(stdout.contains("Options:"), "Should show options section");

    // Verify core arguments are present
    assert!(stdout.contains("<FILE>"), "Should have FILE argument");
    assert!(stdout.contains("--output"), "Should have output option");
    assert!(stdout.contains("--security"), "Should have security option");
    assert!(stdout.contains("--progress"), "Should have progress option");
    assert!(
        stdout.contains("--read-only"),
        "Should have read-only option"
    );
    assert!(stdout.contains("--verbose"), "Should have verbose option");

    // Verify help and version options
    assert!(stdout.contains("--help"), "Should have help option");
    assert!(stdout.contains("--version"), "Should have version option");
}

#[test]
fn test_55_argument_validation_file_required() {
    // Test that FILE argument is required
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("<FILE>"));
}

#[test]
fn test_55_output_directory_option() {
    // Create a temporary valid cpinfo file
    let temp_file = NamedTempFile::with_suffix(".info").unwrap();
    std::fs::write(temp_file.path(), "This is a test cpinfo file\n").unwrap();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--output")
        .arg("/tmp/test-output")
        .arg("--read-only"); // Use read-only to avoid actual file creation

    // Should not fail due to argument parsing
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should not contain argument parsing errors
    assert!(
        !stderr.contains("invalid value"),
        "Should not have argument parsing errors"
    );
    assert!(
        !stderr.contains("required arguments were not provided"),
        "Should have all required arguments"
    );
}

#[test]
fn test_55_boolean_flags_parsing() {
    let temp_file = NamedTempFile::with_suffix(".info").unwrap();
    std::fs::write(temp_file.path(), "Test cpinfo content\n").unwrap();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--security")
        .arg("--progress")
        .arg("--read-only")
        .arg("--verbose");

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should not contain boolean flag parsing errors
    assert!(
        !stderr.contains("invalid value"),
        "Boolean flags should parse correctly"
    );
    assert!(
        !stderr.contains("unexpected argument"),
        "All flags should be recognized"
    );
}

#[test]
fn test_55_short_flag_aliases() {
    let temp_file = NamedTempFile::with_suffix(".info").unwrap();
    std::fs::write(temp_file.path(), "Test cpinfo content\n").unwrap();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("-o")
        .arg("/tmp/test") // Short form of --output
        .arg("-v") // Short form of --verbose
        .arg("--read-only");

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should recognize short flag aliases
    assert!(
        !stderr.contains("unexpected argument"),
        "Short flags should be recognized"
    );
    assert!(!stderr.contains("invalid"), "Short flags should be valid");
}

#[test]
fn test_55_version_display() {
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--version");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should display version information
    assert!(
        !stdout.trim().is_empty(),
        "Version output should not be empty"
    );
    // Should contain some version-like pattern (numbers and dots)
    assert!(
        stdout.contains(char::is_numeric),
        "Version should contain numbers"
    );
}

/// Test helper to create a minimal valid cpinfo file
fn create_test_cpinfo_file() -> NamedTempFile {
    let temp_file = NamedTempFile::with_suffix(".info").unwrap();
    let content = "
cpinfo version check
====================
System Information
==================
Test content for cpinfo file
";
    std::fs::write(temp_file.path(), content).unwrap();
    return temp_file;
}

#[test]
fn test_55_argument_combination_validation() {
    let temp_file = create_test_cpinfo_file();

    // Test valid combination
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--output")
        .arg("/tmp/valid")
        .arg("--security")
        .arg("--read-only");

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should not have combination validation errors
    assert!(
        !stderr.contains("cannot be used with"),
        "Valid combinations should be accepted"
    );
    assert!(
        !stderr.contains("mutually exclusive"),
        "No mutual exclusion should be violated"
    );
}

#[test]
fn test_55_error_message_clarity() {
    // Test error message when file doesn't exist
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("nonexistent_file.info");

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Error messages should be clear and actionable
    // The CLI uses tracing logger which outputs to stdout/terminal
    let error_output = if stderr.is_empty() {
        stdout.clone()
    } else {
        stderr.clone()
    };

    assert!(
        !error_output.is_empty(),
        "Should provide error message. stderr: '{stderr}', stdout: '{stdout}'"
    );

    // Should not just be a panic or cryptic message
    assert!(
        error_output.contains("No such file")
            || error_output.contains("not found")
            || error_output.contains("does not exist")
            || error_output.contains("Failed to")
            || error_output.contains("File not found"),
        "Should provide clear file error message. stderr: '{stderr}', stdout: '{stdout}'"
    );

    // Should exit with non-zero status on error
    assert!(!output.status.success(), "Should exit with error status");
}

/// Test 56: Help system integration and documentation display
#[test]
fn test_56_help_system_integration() {
    // Test comprehensive help documentation
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--help");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should include program description
    assert!(
        stdout.contains("streaming parser") && stdout.contains("Check Point"),
        "Should show program description"
    );
    assert!(
        stdout.contains("high-performance") || stdout.contains("performance"),
        "Should mention performance characteristics"
    );

    // Should include all command line options with descriptions
    assert!(
        stdout.contains("--output") && stdout.contains("Output directory"),
        "Should document output option"
    );
    assert!(
        stdout.contains("--security") && stdout.contains("security controls"),
        "Should document security option"
    );
    assert!(
        stdout.contains("--progress") && stdout.contains("progress reporting"),
        "Should document progress option"
    );
    assert!(
        stdout.contains("--read-only") && stdout.contains("Read-only mode"),
        "Should document read-only option"
    );
    assert!(
        stdout.contains("--verbose") && stdout.contains("Verbose logging"),
        "Should document verbose option"
    );

    // Should show usage examples or patterns
    assert!(
        stdout.contains("Usage:") || stdout.contains("USAGE:"),
        "Should show usage section"
    );
    assert!(
        stdout.contains("<FILE>"),
        "Should show required file argument"
    );

    // Should include version information access
    assert!(
        stdout.contains("--version") || stdout.contains("-V"),
        "Should show version option"
    );

    // Should exit successfully when showing help
    assert!(
        output.status.success(),
        "Help should exit with success status"
    );
}

#[test]
fn test_56_help_system_detailed_documentation() {
    // Test that help includes sufficient detail for enterprise users
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--help");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should mention Check Point specific context
    assert!(
        stdout.contains("Check Point") || stdout.contains("cpinfo"),
        "Should mention Check Point context"
    );

    // Should indicate enterprise/professional nature
    assert!(
        stdout.contains("diagnostic") || stdout.contains("analysis") || stdout.contains("security"),
        "Should indicate professional diagnostic tool nature"
    );

    // Should provide enough information for administrators to understand purpose
    let word_count = stdout.split_whitespace().count();
    assert!(
        word_count > 50,
        "Help should be comprehensive with sufficient detail"
    );

    // Should be formatted for readability
    assert!(stdout.contains('\n'), "Help should be multi-line");
    assert!(
        stdout.len() > 200,
        "Help should provide substantial information"
    );
}

#[test]
fn test_56_help_system_accessibility() {
    // Test that help output is accessible and well-structured
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--help");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should have clear section headers
    let has_sections = stdout.contains("Usage:")
        || stdout.contains("Arguments:")
        || stdout.contains("Options:")
        || stdout.contains("USAGE:");
    assert!(has_sections, "Should have clear section organization");

    // Should not have overly long lines (accessibility guideline)
    let lines = stdout.lines();
    let mut max_line_length = 0;
    for line in lines {
        if line.len() > max_line_length {
            max_line_length = line.len();
        }
    }
    assert!(
        max_line_length <= 120,
        "Help lines should not be excessively long for readability"
    );

    // Should not contain special characters that might cause issues with screen readers
    assert!(
        !stdout.contains("\u{2502}")
            && !stdout.contains("\u{250c}")
            && !stdout.contains("\u{2514}"),
        "Should avoid complex box drawing characters"
    );
}

/// Test 57: Progress reporting and status feedback during processing
#[test]
fn test_57_progress_reporting_flag() {
    // Test that --progress flag is accepted and doesn't cause argument errors
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--progress")
        .arg("--read-only");

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should not have argument parsing errors
    assert!(
        !stderr.contains("unexpected argument"),
        "Progress flag should be recognized"
    );
    assert!(!stderr.contains("invalid"), "Progress flag should be valid");
}

#[test]
fn test_57_progress_reporting_no_crash() {
    // Test that enabling progress reporting doesn't crash the system
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--progress")
        .arg("--read-only")
        .arg("--verbose");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should complete without crashing
    // Exit status may be error due to parsing logic, but should not be due to progress system
    let combined_output = format!("{stdout}{stderr}");
    assert!(
        !combined_output.contains("panic"),
        "Should not panic with progress enabled"
    );
    assert!(
        !combined_output.contains("thread panicked"),
        "Should not have thread panic"
    );
}

#[test]
fn test_57_progress_status_feedback() {
    // Test that progress system provides status feedback appropriately
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--progress")
        .arg("--verbose")
        .arg("--read-only");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    let combined_output = format!("{stdout}{stderr}");

    // Should show processing status information when verbose and progress are enabled
    // This may include file size info, processing steps, or completion status
    assert!(
        combined_output.contains("CPInfo Parser")
            || combined_output.contains("processing")
            || combined_output.contains("Starting")
            || combined_output.contains("Input file"),
        "Should provide status feedback when progress and verbose are enabled"
    );
}

#[test]
fn test_57_progress_accessibility_compliance() {
    // Test that progress reporting is accessible (text-based, not just visual)
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--progress")
        .arg("--read-only");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Progress should be text-based and accessible to screen readers
    // Should not rely solely on visual indicators like progress bars without text
    let combined_output = format!("{stdout}{stderr}");

    // If progress is shown, it should be in text form
    // For a read-only small test file, progress might not be visible, which is acceptable
    // The key is that when progress IS shown, it should be accessible

    // Should not contain non-text progress indicators that would be problematic for screen readers
    assert!(
        !combined_output.contains('\r'),
        "Should not use carriage returns that confuse screen readers"
    );

    // Should not contain progress bar characters without accompanying text
    let progress_chars = [
        '\u{2588}', '\u{2593}', '\u{2592}', '\u{2591}', '\u{25a0}', '\u{25a1}', '\u{25cf}',
        '\u{25cb}',
    ];
    let has_progress_chars = progress_chars
        .iter()
        .any(|&c| return combined_output.contains(c));
    if has_progress_chars {
        // If visual progress indicators exist, there should also be text content
        assert!(
            combined_output.len() > 50,
            "Visual progress should be accompanied by text descriptions"
        );
    }
}

/// Test 58: Interactive prompts and user confirmation dialogs
#[test]
fn test_58_non_interactive_batch_operation() {
    // Test that the CLI operates in non-interactive mode suitable for automation
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--read-only")
        .arg("--verbose");

    // Provide no stdin input to ensure it doesn't hang waiting for user input
    let output = cmd.output().unwrap();

    // Should complete without hanging or requiring user interaction
    // The command should finish within reasonable time without user input
    assert!(
        output.status.code().is_some(),
        "Should exit with a status code, not hang indefinitely"
    );
}

#[test]
fn test_58_overwrite_protection_read_only() {
    // Test that read-only mode prevents file overwrites without prompting
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--read-only") // This should prevent any file creation
        .arg("--output")
        .arg("/tmp/test-cpinfo-output");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should not prompt for confirmation - should handle read-only mode automatically
    let combined_output = format!("{stdout}{stderr}");
    assert!(
        !combined_output.contains("(y/n)"),
        "Should not prompt for yes/no confirmation"
    );
    assert!(
        !combined_output.contains("Continue?"),
        "Should not prompt for continuation"
    );
    assert!(
        !combined_output.contains("Overwrite?"),
        "Should not prompt for overwrite confirmation"
    );
}

#[test]
fn test_58_dangerous_operation_safeguards() {
    // Test that potentially dangerous operations have appropriate safeguards
    let temp_file = create_test_cpinfo_file();

    // Test with system directory that should be rejected
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--output")
        .arg("/etc/cpinfo-test"); // Should be rejected as dangerous

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should reject dangerous operations without prompting
    let combined_output = format!("{stdout}{stderr}");
    // This might fail due to permissions or be rejected by security controls
    // Either way, it should not prompt the user - it should make the decision automatically
    assert!(
        !combined_output.contains("Are you sure?"),
        "Should not prompt for dangerous operations"
    );
    assert!(
        !combined_output.contains("(y/n)"),
        "Should handle safety checks automatically"
    );
}

#[test]
fn test_58_automation_friendly_design() {
    // Test that the CLI is designed for automation and batch processing
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--read-only")
        .arg("--verbose");

    // Simulate automation environment by providing no stdin
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    let combined_output = format!("{stdout}{stderr}");

    // Should provide clear, parseable output suitable for automation
    assert!(
        !combined_output.contains("Press any key"),
        "Should not wait for key presses"
    );
    assert!(
        !combined_output.contains("Enter to continue"),
        "Should not wait for enter key"
    );

    // Should complete deterministically
    assert!(
        output.status.code().is_some(),
        "Should have deterministic exit code"
    );
}

/// Test 59: Output formatting and result presentation
#[test]
fn test_59_output_directory_structure() {
    // Test that output directory structure is logical and organized
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--output")
        .arg("/tmp/cpinfo-test-output")
        .arg("--read-only") // Prevent actual file creation for this test
        .arg("--verbose");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    let combined_output = format!("{stdout}{stderr}");

    // Should show intended output organization in verbose mode
    assert!(
        combined_output.contains("Output directory:") || combined_output.contains("output"),
        "Should reference output directory"
    );
}

#[test]
fn test_59_verbose_logging_format() {
    // Test that verbose output is well-formatted and informative
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--verbose")
        .arg("--read-only");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    let combined_output = format!("{stdout}{stderr}");

    // Should include structured logging information
    assert!(
        combined_output.contains("CPInfo Parser") || combined_output.contains("INFO"),
        "Should include informational logging"
    );

    // Should include version information
    assert!(
        combined_output.contains("v0.1.0") || combined_output.contains("starting"),
        "Should show version or startup information"
    );

    // Should be structured and readable
    let lines: Vec<&str> = combined_output.lines().collect();
    assert!(lines.len() > 1, "Verbose output should be multi-line");

    // Should have consistent formatting
    let has_timestamps = lines
        .iter()
        .any(|line| return line.contains("INFO") || line.contains("ERROR"));
    if has_timestamps {
        // If using structured logging, should have consistent format
        assert!(
            lines.len() >= 3,
            "Structured logging should provide substantial information"
        );
    }
}

#[test]
fn test_59_error_message_formatting() {
    // Test that error messages are well-formatted and actionable
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("definitely-nonexistent-file.info").arg("--verbose");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    let combined_output = format!("{stdout}{stderr}");

    // Should have clear error indication
    assert!(
        combined_output.contains("ERROR")
            || combined_output.contains("Failed")
            || combined_output.contains("not found"),
        "Should have clear error indication"
    );

    // Should provide specific error information
    assert!(
        combined_output.contains("File not found")
            || combined_output.contains("nonexistent-file")
            || combined_output.contains("definitely-nonexistent-file.info"),
        "Should reference the specific problematic file"
    );

    // Should not be just a stack trace or cryptic message
    assert!(
        !combined_output.contains("panic"),
        "Should not contain panic information"
    );
    assert!(
        !combined_output.contains("unwrap"),
        "Should not contain unwrap errors"
    );
}

#[test]
fn test_59_machine_readable_aspects() {
    // Test that output has machine-readable aspects for automation
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path()).arg("--read-only");

    let output = cmd.output().unwrap();

    // Should have predictable exit codes
    assert!(output.status.code().is_some(), "Should have exit code");

    // Should not mix different types of output inappropriately
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // In non-verbose mode, output should be minimal and parseable
    if !stdout.is_empty() {
        // If there's stdout output, it should be structured
        assert!(
            !stdout.contains("DEBUG"),
            "Non-verbose stdout should not contain debug info"
        );
    }

    // Error output should go to appropriate stream
    if !stderr.is_empty() {
        // If using stderr, it should be for actual errors
        assert!(
            !stderr.contains("INFO:"),
            "INFO messages should not go to stderr"
        );
    }
}

#[test]
fn test_59_professional_presentation() {
    // Test that output presentation is professional and enterprise-appropriate
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--help");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should have professional terminology and presentation
    assert!(
        stdout.contains("Check Point")
            || stdout.contains("cpinfo")
            || stdout.contains("diagnostic"),
        "Should reference Check Point professional context"
    );

    // Should not contain informal language
    assert!(
        !stdout.contains("stuff"),
        "Should not contain informal language"
    );
    assert!(
        !stdout.contains("things"),
        "Should not contain vague terminology"
    );

    // Should use proper capitalization and grammar
    assert!(
        stdout.contains("Output directory"),
        "Should use proper capitalization"
    );

    // Should be appropriate for enterprise documentation
    let word_count = stdout.split_whitespace().count();
    assert!(
        word_count > 30,
        "Should provide sufficient professional information"
    );
}

/// Test 60: Keyboard navigation and accessibility compliance (WCAG 2.1 AA)
#[test]
fn test_60_screen_reader_compatible_progress() {
    // Test that progress reporting is compatible with screen readers
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--progress")
        .arg("--verbose")
        .arg("--read-only");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    let combined_output = format!("{stdout}{stderr}");

    // Should not use cursor manipulation that confuses screen readers
    assert!(
        !combined_output.contains("\x1b["),
        "Should not contain ANSI escape sequences for cursor control"
    );
    assert!(
        !combined_output.contains('\r'),
        "Should not use carriage returns that interfere with screen readers"
    );

    // Should not use visual-only progress indicators without text
    let problematic_chars = [
        '\u{2588}', '\u{2593}', '\u{2592}', '\u{2591}', '\u{25a0}', '\u{25a1}', '\u{25cf}',
        '\u{25cb}', '\u{25d0}', '\u{25d1}', '\u{25d2}', '\u{25d3}',
    ];
    let has_visual_only = problematic_chars
        .iter()
        .any(|&c| return combined_output.contains(c));

    if has_visual_only {
        // If visual indicators are used, they must be accompanied by text descriptions
        assert!(
            combined_output.contains("progress")
                || combined_output.contains("processing")
                || combined_output.contains('%'),
            "Visual progress indicators must be accompanied by text descriptions"
        );
    }

    // Should provide text-based status information suitable for screen readers
    if combined_output.len() > 50 {
        // If there's substantial output, it should include screen reader friendly text
        assert!(
            combined_output.contains("INFO")
                || combined_output.contains("Starting")
                || combined_output.contains("Processing")
                || combined_output.contains("CPInfo"),
            "Should provide text-based status information for screen readers"
        );
    }
}

#[test]
fn test_60_keyboard_only_operation() {
    // Test that all CLI functionality is accessible via keyboard only (inherent design)
    let temp_file = create_test_cpinfo_file();

    // Test that the CLI can be operated entirely through keyboard commands
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--read-only")
        .arg("--verbose");

    let output = cmd.output().unwrap();

    // Should not require mouse interaction or GUI elements
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let combined_output = format!("{stdout}{stderr}");

    // Should not prompt for mouse clicks or GUI interactions
    assert!(
        !combined_output.contains("click"),
        "Should not require mouse clicks"
    );
    assert!(
        !combined_output.contains("mouse"),
        "Should not require mouse interaction"
    );
    assert!(
        !combined_output.contains("GUI"),
        "Should not require GUI interaction"
    );

    // Should complete operation without requiring additional input beyond command line
    assert!(
        output.status.code().is_some(),
        "Should complete with keyboard-only operation"
    );

    // Should not hang waiting for non-keyboard input
    // The fact that cmd.output() completes successfully demonstrates this
}

#[test]
fn test_60_accessible_error_messages() {
    // Test that error messages are descriptive and accessible to screen readers
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("nonexistent_accessibility_test.info");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let combined_output = format!("{stdout}{stderr}");

    // Error messages should be descriptive and screen reader friendly
    assert!(
        !combined_output.is_empty(),
        "Should provide error message output"
    );

    // Should include context about what went wrong
    assert!(
        combined_output.contains("File not found")
            || combined_output.contains("not found")
            || combined_output.contains("Failed to")
            || combined_output.contains("ERROR"),
        "Should provide descriptive error context"
    );

    // Should include specific file information to help user understand the issue
    assert!(
        combined_output.contains("nonexistent_accessibility_test.info")
            || combined_output.contains("nonexistent"),
        "Should reference the specific problematic file"
    );

    // Should not be just technical jargon that's hard for users to understand
    assert!(
        !combined_output.contains("std::fs::File::open"),
        "Should not expose technical implementation details"
    );
    assert!(
        !combined_output.contains("unwrap()"),
        "Should not expose Rust-specific error details"
    );

    // Should have proper error level indication
    assert!(
        !output.status.success(),
        "Should indicate error through exit status"
    );
}

#[test]
fn test_60_structured_help_output() {
    // Test that help output is properly structured for screen readers
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--help");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should have clear hierarchical structure for screen readers
    assert!(
        stdout.contains("Usage:") || stdout.contains("USAGE:"),
        "Should have clear usage section"
    );
    assert!(
        stdout.contains("Arguments:"),
        "Should have clear arguments section"
    );
    assert!(
        stdout.contains("Options:"),
        "Should have clear options section"
    );

    // Should have logical reading order (sections appear in expected sequence)
    let usage_pos = stdout
        .find("Usage:")
        .or_else(|| return stdout.find("USAGE:"));
    let args_pos = stdout.find("Arguments:");
    let opts_pos = stdout.find("Options:");

    if let (Some(usage), Some(args), Some(opts)) = (usage_pos, args_pos, opts_pos) {
        assert!(usage < args, "Usage should come before Arguments section");
        assert!(args < opts, "Arguments should come before Options section");
    }

    // Should not have overly complex formatting that confuses screen readers
    let lines: Vec<&str> = stdout.lines().collect();
    let max_line_length = lines
        .iter()
        .map(|line| return line.len())
        .max()
        .unwrap_or(0);
    assert!(
        max_line_length <= 120,
        "Lines should not be excessively long for screen reader readability"
    );

    // Should avoid ASCII art or complex symbols that don't read well
    assert!(
        !stdout.contains("\u{250c}"),
        "Should avoid complex box drawing characters"
    );
    assert!(
        !stdout.contains("\u{2502}"),
        "Should avoid vertical line characters"
    );
    assert!(
        !stdout.contains("\u{2514}"),
        "Should avoid corner characters"
    );

    // Should have consistent indentation for nested information
    let has_consistent_structure = lines.iter().any(|line| {
        return line.starts_with("  -") || line.starts_with("    ") || line.contains("  --");
    });
    assert!(
        has_consistent_structure,
        "Should have consistent indentation structure"
    );
}

#[test]
fn test_60_no_visual_only_indicators() {
    // Test that all information is available in text form, not just visual
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--progress")
        .arg("--verbose")
        .arg("--read-only");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let combined_output = format!("{stdout}{stderr}");

    // Should not rely on color alone to convey information
    // Since this is a CLI tool, color usage should be tested if present
    assert!(
        !combined_output.contains("\x1b[31m") || combined_output.contains("ERROR"),
        "If red color is used, it should be accompanied by ERROR text"
    );
    assert!(
        !combined_output.contains("\x1b[32m")
            || combined_output.contains("SUCCESS")
            || combined_output.contains("OK"),
        "If green color is used, it should be accompanied by success text"
    );

    // Should not use symbols without text alternatives
    let symbol_chars = [
        '\u{2713}', '\u{2717}', '\u{26a0}', '\u{2192}', '\u{2190}', '\u{2191}', '\u{2193}',
    ];
    let has_symbols = symbol_chars
        .iter()
        .any(|&c| return combined_output.contains(c));

    if has_symbols {
        // If symbols are used, ensure there's accompanying text
        assert!(
            combined_output.len() > 100,
            "Symbols should be accompanied by descriptive text"
        );
    }

    // Should provide status information in text form
    if combined_output.len() > 20 {
        // If there's output, it should be primarily text-based and readable
        // Count alphanumeric, whitespace, and common punctuation as "readable" characters
        let readable_chars: usize = combined_output
            .chars()
            .filter(|c| {
                return c.is_alphanumeric()
                    || c.is_whitespace()
                    || [':', '.', '-', '_', '/', '(', ')', '"', ',', ';'].contains(c);
            })
            .count();
        let total_chars = combined_output.len();

        // At least 90% of characters should be readable text (not special symbols or control chars)
        let readable_ratio = readable_chars as f64 / total_chars as f64;
        assert!(
            readable_ratio >= 0.9,
            "Output should be primarily readable text for accessibility. Ratio: {readable_ratio:.2}"
        );
    }
}

#[test]
fn test_60_wcag_compliance_text_output() {
    // Test that text output meets WCAG 2.1 AA guidelines for accessibility
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--help");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // WCAG 2.1 AA requires meaningful text content
    assert!(
        stdout.len() > 100,
        "Should provide substantial textual content"
    );

    // WCAG guideline: Text should be readable and understandable
    let word_count = stdout.split_whitespace().count();
    assert!(
        word_count > 20,
        "Should provide sufficient textual information"
    );

    // WCAG guideline: Information should not rely solely on sensory characteristics
    // (This is inherently met by CLI text output, but we verify no problematic patterns)
    assert!(
        !stdout.contains("see above"),
        "Should not rely on visual references"
    );
    assert!(
        !stdout.contains("click here"),
        "Should not reference visual interactions"
    );
    assert!(
        !stdout.contains("the red text"),
        "Should not reference color-only information"
    );

    // WCAG guideline: Content should be readable by assistive technology
    // Test that text has proper structure and doesn't use problematic formatting
    assert!(
        !stdout.contains("\t\t\t"),
        "Should not use excessive tabs that confuse screen readers"
    );

    // WCAG guideline: Navigation should be logical and consistent
    let sections = [
        stdout
            .find("Usage:")
            .or_else(|| return stdout.find("USAGE:")),
        stdout.find("Arguments:"),
        stdout.find("Options:"),
    ];

    // Should have logical section ordering
    let defined_sections: Vec<usize> = sections.into_iter().flatten().collect();
    if defined_sections.len() >= 2 {
        // Sections should be in ascending order (logical reading sequence)
        for i in 1..defined_sections.len() {
            assert!(
                defined_sections[i - 1] < defined_sections[i],
                "Sections should appear in logical reading order"
            );
        }
    }

    // WCAG guideline: Error identification and description
    // Test error message accessibility
    let mut error_cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    error_cmd.arg("wcag_test_nonexistent.info");

    let error_output = error_cmd.output().unwrap();
    let error_stdout = String::from_utf8(error_output.stdout).unwrap();
    let error_stderr = String::from_utf8(error_output.stderr).unwrap();
    let error_combined = format!("{error_stdout}{error_stderr}");

    if !error_combined.is_empty() {
        // Error messages should clearly identify the problem
        assert!(
            error_combined.contains("not found")
                || error_combined.contains("ERROR")
                || error_combined.contains("Failed"),
            "Error messages should clearly identify the problem for screen readers"
        );
    }
}

/// Test 61: Screen reader compatibility and ARIA integration for terminal output
#[test]
fn test_61_screen_reader_semantic_output() {
    // RED PHASE: Test screen reader semantic markup for CLI output
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--verbose")
        .arg("--read-only");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let combined_output = format!("{stdout}{stderr}");

    // Should provide semantic structure for screen readers
    assert!(
        combined_output.contains("CPInfo Parser")
            || combined_output.contains("[INFO]")
            || combined_output.contains("Starting"),
        "Should provide semantic application identification"
    );

    // Should use consistent labeling for different types of information
    if combined_output.contains("Input file:") {
        assert!(
            combined_output.contains("Output directory:")
                || combined_output.contains("Processing:"),
            "Should have consistent labeling structure"
        );
    }

    // Should not use ambiguous references that screen readers can't interpret
    assert!(
        !combined_output.contains("this file"),
        "Should not use ambiguous references"
    );
    assert!(
        !combined_output.contains("that option"),
        "Should not use ambiguous pronoun references"
    );
}

#[test]
fn test_61_screen_reader_status_announcements() {
    // Test that status changes are announced clearly for screen readers
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--progress")
        .arg("--verbose")
        .arg("--read-only");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let combined_output = format!("{stdout}{stderr}");

    // Should announce major status transitions
    if combined_output.len() > 50 {
        // Look for clear status announcements
        let has_clear_status = combined_output.contains("Starting")
            || combined_output.contains("Processing")
            || combined_output.contains("Completed")
            || combined_output.contains("Initializing");

        if has_clear_status {
            // Status announcements should be complete sentences or clear phrases
            assert!(
                !combined_output
                    .lines()
                    .any(|line| return line.trim().len() == 1
                        && line.chars().all(|c| return c.is_ascii_punctuation())),
                "Should not use punctuation-only status indicators"
            );
        }
    }

    // Should not use non-text status indicators without text equivalents
    let visual_only_chars = [
        '\u{25cf}', '\u{25cb}', '\u{25d0}', '\u{25d1}', '\u{25d2}', '\u{25d3}', '\u{25b6}',
        '\u{23f8}', '\u{23f9}', '\u{25a0}',
    ];
    let has_visual_status = visual_only_chars
        .iter()
        .any(|&c| return combined_output.contains(c));

    if has_visual_status {
        // Visual indicators must be accompanied by text descriptions
        assert!(
            combined_output.contains("Running")
                || combined_output.contains("Stopped")
                || combined_output.contains("Processing")
                || combined_output.contains("Complete"),
            "Visual status indicators must have text equivalents"
        );
    }
}

#[test]
fn test_61_screen_reader_navigation_structure() {
    // Test that help output has logical navigation structure for screen readers
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--help");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should have clear heading structure
    let headings = ["Usage:", "Arguments:", "Options:"];
    let mut found_headings = Vec::new();

    for heading in &headings {
        if let Some(pos) = stdout.find(heading) {
            found_headings.push((heading, pos));
        }
    }

    // Headings should appear in logical order
    found_headings.sort_by_key(|(_, pos)| return *pos);
    if found_headings.len() >= 2 {
        // Should maintain document structure
        for i in 1..found_headings.len() {
            let (prev_heading, prev_pos) = found_headings[i - 1];
            let (curr_heading, curr_pos) = found_headings[i];

            assert!(
                prev_pos < curr_pos,
                "Headings should appear in logical order: {prev_heading} before {curr_heading}"
            );
        }
    }

    // Should have appropriate content under each heading
    if stdout.contains("Arguments:") {
        let args_pos = stdout.find("Arguments:").unwrap();
        let remaining_text = &stdout[args_pos..];

        // Should have file argument description
        assert!(
            remaining_text.contains("<FILE>") || remaining_text.contains("FILE"),
            "Arguments section should describe file argument"
        );
    }
}

#[test]
fn test_61_screen_reader_error_context() {
    // Test that errors provide sufficient context for screen readers
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("screen_reader_test_missing.info");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let combined_output = format!("{stdout}{stderr}");

    // Should provide complete error context
    assert!(!combined_output.is_empty(), "Should provide error output");

    // Should identify what failed
    assert!(
        combined_output.contains("File not found")
            || combined_output.contains("not found")
            || combined_output.contains("Failed")
            || combined_output.contains("ERROR"),
        "Should clearly identify the type of error"
    );

    // Should identify the specific problematic item
    assert!(
        combined_output.contains("screen_reader_test_missing.info")
            || combined_output.contains("missing"),
        "Should identify the specific file that caused the error"
    );

    // Should provide actionable information
    let lines: Vec<&str> = combined_output.lines().collect();
    let has_substantial_error = lines.iter().any(|line| return line.len() > 20);
    assert!(
        has_substantial_error,
        "Should provide substantial error information for screen readers"
    );
}

/// Test 62: Color-blind friendly interface design with alternative indicators
#[test]
fn test_62_color_independent_information() {
    // RED PHASE: Test that information is not conveyed through color alone
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--verbose")
        .arg("--read-only");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let combined_output = format!("{stdout}{stderr}");

    // Check for ANSI color codes and ensure they're paired with text
    let ansi_patterns = [
        "\x1b[31m", // Red
        "\x1b[32m", // Green
        "\x1b[33m", // Yellow
        "\x1b[34m", // Blue
        "\x1b[35m", // Magenta
        "\x1b[36m", // Cyan
    ];

    for pattern in &ansi_patterns {
        if combined_output.contains(pattern) {
            // If color is used, verify there's accompanying text indication
            match *pattern {
                "\x1b[31m" => assert!(
                    combined_output.contains("ERROR")
                        || combined_output.contains("FAILED")
                        || combined_output.contains("WARNING"),
                    "Red color must be accompanied by error/warning text"
                ),
                "\x1b[32m" => assert!(
                    combined_output.contains("SUCCESS")
                        || combined_output.contains("OK")
                        || combined_output.contains("COMPLETE"),
                    "Green color must be accompanied by success text"
                ),
                "\x1b[33m" => assert!(
                    combined_output.contains("WARNING")
                        || combined_output.contains("CAUTION")
                        || combined_output.contains("NOTE"),
                    "Yellow color must be accompanied by warning/note text"
                ),
                _ => {
                    // Other colors should also have text context
                    assert!(
                        combined_output.split_whitespace().count() > 5,
                        "Color usage must be accompanied by descriptive text"
                    );
                }
            }
        }
    }

    // Should not use color as the sole means of conveying status
    if combined_output.contains("\x1b[") {
        // If colors are used, verify text alternatives exist
        let text_indicators = ["INFO", "ERROR", "WARNING", "SUCCESS", "FAILED", "OK"];
        let has_text_indicators = text_indicators
            .iter()
            .any(|&indicator| return combined_output.contains(indicator));
        assert!(
            has_text_indicators,
            "Color usage must be supplemented with text indicators"
        );
    }
}

#[test]
fn test_62_symbol_based_status_indicators() {
    // Test that status is indicated through symbols and text, not just color
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--progress")
        .arg("--verbose")
        .arg("--read-only");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let combined_output = format!("{stdout}{stderr}");

    // Should use text-based status indicators
    let status_patterns = [
        "[INFO]",
        "[ERROR]",
        "[WARNING]",
        "[DEBUG]",
        "INFO:",
        "ERROR:",
        "WARNING:",
        "DEBUG:",
        "\u{2713}",
        "\u{2717}",
        "\u{2192}",
        "\u{2022}",
    ];

    let has_text_status = status_patterns
        .iter()
        .any(|&pattern| return combined_output.contains(pattern));

    if combined_output.len() > 50 && has_text_status {
        // If status indicators are present, they should be readable
        assert!(
            combined_output.contains("INFO")
                || combined_output.contains("Starting")
                || combined_output.contains("Processing"),
            "Should use readable text-based status indicators"
        );
    }

    // Should not rely on spacing or positioning alone to indicate status
    let lines: Vec<&str> = combined_output.lines().collect();
    for line in &lines {
        if !line.trim().is_empty() {
            // Each non-empty line should have meaningful content, not positioning-dependent info
            assert!(
                !line
                    .chars()
                    .all(|c| return c.is_whitespace() || c == '.' || c == '-'),
                "Should not use whitespace or dots alone to convey information"
            );
        }
    }
}

#[test]
fn test_62_high_contrast_text_patterns() {
    // Test that text patterns work well for users with color vision deficiencies
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--help");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should use clear textual differentiation
    let sections = ["Usage:", "Arguments:", "Options:"];
    for section in &sections {
        if stdout.contains(section) {
            // Section headers should be clearly differentiated from content
            let section_line = stdout
                .lines()
                .find(|line| return line.contains(section))
                .unwrap();

            // Should either be on its own line or clearly formatted
            assert!(
                section_line.trim() == *section
                    || section_line.ends_with(':')
                    || section_line.len() < 50,
                "Section headers should be clearly differentiated: {section_line}"
            );
        }
    }

    // Should use consistent formatting that doesn't depend on color
    let has_consistent_formatting = stdout.contains("--") || stdout.contains('-');
    if has_consistent_formatting {
        // Command line options should be clearly marked with dashes
        assert!(
            stdout.contains("--help") || stdout.contains("--version"),
            "Command line options should use clear textual markers"
        );
    }

    // Should avoid low-contrast text combinations (can't test color directly, but check formatting)
    let lines: Vec<&str> = stdout.lines().collect();
    for line in &lines {
        if line.len() > 80 {
            // Long lines should have clear structure
            assert!(
                line.contains("  ")
                    || line.contains('\t')
                    || line.contains('-')
                    || line.contains(':'),
                "Long lines should have clear structural elements for readability"
            );
        }
    }
}

#[test]
fn test_62_error_indication_without_color() {
    // Test that errors are clearly indicated without relying on color
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("colorblind_test_nonexistent.info");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let combined_output = format!("{stdout}{stderr}");

    // Should clearly indicate error status through text
    assert!(
        combined_output.contains("ERROR")
            || combined_output.contains("Failed")
            || combined_output.contains("not found")
            || combined_output.contains("File not found"),
        "Errors should be clearly indicated through text markers"
    );

    // Should have clear error context
    assert!(
        combined_output.contains("colorblind_test_nonexistent.info")
            || combined_output.contains("nonexistent"),
        "Should identify the specific problematic file"
    );

    // Should use exit status to indicate error (machine-readable)
    assert!(
        !output.status.success(),
        "Should use exit status to indicate error state"
    );

    // Should not rely on visual formatting alone
    let error_lines: Vec<&str> = combined_output
        .lines()
        .filter(|line| {
            return line.to_lowercase().contains("error") || line.to_lowercase().contains("failed");
        })
        .collect();

    for error_line in &error_lines {
        // Error lines should contain descriptive text, not just formatting
        let word_count = error_line.split_whitespace().count();
        assert!(
            word_count >= 2,
            "Error messages should contain descriptive text: {error_line}"
        );
    }
}

/// Test 63: Terminal resize handling and responsive layout for different screen sizes
#[test]
fn test_63_narrow_terminal_width_handling() {
    // RED PHASE: Test behavior with narrow terminal widths

    let temp_file = create_test_cpinfo_file();

    // Simulate narrow terminal by setting COLUMNS environment variable
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.env("COLUMNS", "40") // Narrow terminal width
        .arg(temp_file.path())
        .arg("--help");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should handle narrow width gracefully
    let lines: Vec<&str> = stdout.lines().collect();
    let has_long_lines = lines.iter().any(|line| return line.len() > 80);

    // In narrow terminals, should either wrap or use shorter lines
    if has_long_lines {
        // If long lines exist, they should be properly structured
        for line in &lines {
            if line.len() > 80 {
                // Long lines should contain natural break points
                assert!(
                    line.contains(' ') || line.contains('-') || line.contains(','),
                    "Long lines should have natural break points for text wrapping"
                );
            }
        }
    }

    // Should maintain readability in narrow format
    assert!(
        stdout.contains("Usage:"),
        "Core sections should remain in narrow format"
    );
    assert!(
        stdout.contains("--help"),
        "Essential options should remain visible"
    );
}

#[test]
fn test_63_wide_terminal_width_utilization() {
    // Test behavior with wide terminal widths
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.env("COLUMNS", "120") // Wide terminal width
        .arg(temp_file.path())
        .arg("--help");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should utilize available width appropriately
    let lines: Vec<&str> = stdout.lines().collect();

    // Should not have unnecessarily short lines in wide terminals
    let substantial_lines: Vec<&str> = lines
        .iter()
        .filter(|line| return line.trim().len() > 10)
        .copied()
        .collect();

    if substantial_lines.len() > 3 {
        // Should make reasonable use of available width
        let avg_line_length: f64 = substantial_lines
            .iter()
            .map(|line| return line.len())
            .sum::<usize>() as f64
            / substantial_lines.len() as f64;

        // In wide terminals, average line length should be reasonable
        assert!(
            avg_line_length > 20.0,
            "Should make reasonable use of wide terminal width"
        );
    }

    // Should maintain good readability even with more space
    assert!(
        stdout.contains("Usage:"),
        "Should maintain clear structure in wide format"
    );
    assert!(
        stdout.contains("Options:"),
        "Should maintain section organization"
    );
}

#[test]
fn test_63_terminal_detection_fallback() {
    // Test behavior when terminal size cannot be detected
    let temp_file = create_test_cpinfo_file();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.env_remove("COLUMNS") // Remove column hint
        .env_remove("LINES") // Remove line hint
        .env_remove("TERM") // Remove terminal type
        .arg(temp_file.path())
        .arg("--verbose")
        .arg("--read-only");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let combined_output = format!("{stdout}{stderr}");

    // Should function normally without terminal size information
    assert!(
        !combined_output.contains("panic"),
        "Should not panic without terminal size info"
    );
    assert!(
        !combined_output.contains("terminal"),
        "Should not complain about terminal detection"
    );

    // Should use reasonable default formatting
    if combined_output.len() > 50 {
        let lines: Vec<&str> = combined_output.lines().collect();
        let max_line_length = lines
            .iter()
            .map(|line| return line.len())
            .max()
            .unwrap_or(0);

        // Should use conservative default width (not too wide, not too narrow)
        assert!(
            max_line_length <= 120,
            "Should use reasonable default maximum width"
        );
        assert!(
            max_line_length >= 20,
            "Should use reasonable default minimum width"
        );
    }
}

#[test]
fn test_63_dynamic_content_adjustment() {
    // Test that content adjusts appropriately to available space
    let temp_file = create_test_cpinfo_file();

    // Test with very narrow width
    let mut narrow_cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    narrow_cmd
        .env("COLUMNS", "30")
        .arg(temp_file.path())
        .arg("--help");

    let narrow_output = narrow_cmd.output().unwrap();
    let narrow_stdout = String::from_utf8(narrow_output.stdout).unwrap();

    // Test with normal width
    let mut normal_cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    normal_cmd
        .env("COLUMNS", "80")
        .arg(temp_file.path())
        .arg("--help");

    let normal_output = normal_cmd.output().unwrap();
    let normal_stdout = String::from_utf8(normal_output.stdout).unwrap();

    // Both should contain the same essential information
    assert!(
        narrow_stdout.contains("Usage:") && normal_stdout.contains("Usage:"),
        "Essential sections should appear in both narrow and normal formats"
    );
    assert!(
        narrow_stdout.contains("--help") && normal_stdout.contains("--help"),
        "Essential options should appear in both formats"
    );

    // Should handle different widths gracefully (both should complete successfully)
    assert!(
        narrow_output.status.success() && normal_output.status.success(),
        "Should handle different terminal widths successfully"
    );

    // Content structure should remain logical in both cases
    let narrow_lines: Vec<&str> = narrow_stdout.lines().collect();
    let normal_lines: Vec<&str> = normal_stdout.lines().collect();

    // Both should have multi-line output with structure
    assert!(
        narrow_lines.len() > 3,
        "Narrow format should have structured multi-line output"
    );
    assert!(
        normal_lines.len() > 3,
        "Normal format should have structured multi-line output"
    );
}

/// Test 64: Multi-language support and internationalization for global deployment
#[test]
fn test_64_utf8_character_support() {
    // RED PHASE: Test support for UTF-8 characters in file paths and content

    // Create a file with UTF-8 characters in the name
    let temp_dir = tempfile::tempdir().unwrap();
    let utf8_filename = "\u{6d4b}\u{8bd5}\u{6587}\u{4ef6}.info"; // Chinese characters
    let utf8_path = temp_dir.path().join(utf8_filename);

    std::fs::write(&utf8_path, "Test cpinfo content with UTF-8\n").unwrap();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(&utf8_path).arg("--read-only").arg("--verbose");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let combined_output = format!("{stdout}{stderr}");

    // Should handle UTF-8 filenames without error
    assert!(
        !combined_output.contains("invalid UTF-8"),
        "Should handle UTF-8 filenames"
    );
    assert!(
        !combined_output.contains("encoding error"),
        "Should not have encoding errors"
    );

    // Should either process the file or provide clear error message
    if !output.status.success() {
        // If it fails, should provide informative error message
        assert!(
            !combined_output.is_empty(),
            "Should provide error message for UTF-8 filename"
        );
    }

    // Should not crash or produce garbled output
    assert!(
        !combined_output.contains("\u{fffd}"),
        "Should not produce replacement characters"
    );
}

#[test]
fn test_64_locale_independent_operation() {
    // Test that the application works regardless of system locale
    let temp_file = create_test_cpinfo_file();

    // Test with different locale settings
    let locales = ["C", "en_US.UTF-8", "de_DE.UTF-8", "ja_JP.UTF-8"];

    for locale in &locales {
        let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
        cmd.env("LC_ALL", locale)
            .env("LANG", locale)
            .arg(temp_file.path())
            .arg("--read-only");

        let output = cmd.output().unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();

        // Should function consistently across locales
        assert!(
            !stdout.contains("locale"),
            "Should not have locale-related errors"
        );
        assert!(
            !stderr.contains("locale"),
            "Should not have locale-related errors"
        );

        // Should not crash due to locale settings
        assert!(
            output.status.code().is_some(),
            "Should complete with any locale setting"
        );
    }
}

#[test]
fn test_64_unicode_content_processing() {
    // Test processing of files containing Unicode content
    let temp_file = NamedTempFile::with_suffix(".info").unwrap();
    let unicode_content = r"
cpinfo version check
====================
System Information (\u{7cfb}\u{7edf}\u{4fe1}\u{606f})
==================
German: System\u{fc}berpr\u{fc}fung
French: V\u{e9}rification du syst\u{e8}me
Japanese: \u{30b7}\u{30b9}\u{30c6}\u{30e0}\u{78ba}\u{8a8d}
Arabic: \u{641}\u{62d}\u{635} \u{627}\u{644}\u{646}\u{638}\u{627}\u{645}
Russian: \u{41f}\u{440}\u{43e}\u{432}\u{435}\u{440}\u{43a}\u{430} \u{441}\u{438}\u{441}\u{442}\u{435}\u{43c}\u{44b}
";
    std::fs::write(temp_file.path(), unicode_content).unwrap();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(temp_file.path())
        .arg("--read-only")
        .arg("--verbose");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let combined_output = format!("{stdout}{stderr}");

    // Should handle Unicode content gracefully
    assert!(
        !combined_output.contains("invalid sequence"),
        "Should handle Unicode sequences"
    );
    assert!(
        !combined_output.contains("encoding"),
        "Should not have encoding issues"
    );

    // Should not produce garbled output
    assert!(
        !combined_output.contains("\u{fffd}"),
        "Should not produce Unicode replacement characters"
    );

    // Should complete processing without Unicode-related crashes
    assert!(
        output.status.code().is_some(),
        "Should complete processing Unicode content"
    );
}

#[test]
fn test_64_error_message_consistency() {
    // Test that error messages are consistent and could support localization
    let error_scenarios = [
        "nonexistent_file.info",
        "/invalid/path/file.info",
        "file_without_extension",
    ];

    for scenario in &error_scenarios {
        let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
        cmd.arg(scenario);

        let output = cmd.output().unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        let combined_output = format!("{stdout}{stderr}");

        // Error messages should be structured consistently
        if !combined_output.is_empty() {
            // Should not contain hardcoded paths or system-specific details that would be hard to localize
            assert!(
                !combined_output.contains("/usr/"),
                "Should not contain hardcoded system paths"
            );
            assert!(
                !combined_output.contains("\\Windows\\"),
                "Should not contain hardcoded Windows paths"
            );

            // Should use consistent error message structure
            let has_structured_error = combined_output.contains("ERROR")
                || combined_output.contains("Failed")
                || combined_output.contains("not found");

            if has_structured_error {
                // Error messages should be complete sentences that could be localized
                let lines: Vec<&str> = combined_output
                    .lines()
                    .filter(|line| return !line.trim().is_empty())
                    .collect();

                assert!(!lines.is_empty(), "Should have substantial error content");

                // Should not contain concatenated error strings that would be hard to translate
                for line in &lines {
                    if line.contains("ERROR") || line.contains("Failed") {
                        let word_count = line.split_whitespace().count();
                        assert!(
                            word_count >= 3,
                            "Error messages should be complete phrases: {line}"
                        );
                    }
                }
            }
        }

        // Should have consistent exit behavior
        assert!(
            !output.status.success(),
            "Should exit with error status for invalid input"
        );
    }
}

#[test]
fn test_64_numeric_format_independence() {
    // Test that numeric formatting is not locale-dependent
    let temp_file = create_test_cpinfo_file();

    // Test with locale that uses comma as decimal separator
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.env("LC_NUMERIC", "de_DE.UTF-8") // German locale uses comma for decimals
        .arg(temp_file.path())
        .arg("--verbose")
        .arg("--read-only");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let combined_output = format!("{stdout}{stderr}");

    // Should use consistent numeric formatting regardless of locale
    if combined_output.contains("bytes")
        || combined_output.contains("MB")
        || combined_output.contains("KB")
    {
        // Numeric values should be formatted consistently (likely using dots for decimals)
        // This ensures compatibility with international systems
        let has_numeric_content = combined_output.chars().any(|c| return c.is_ascii_digit());

        if has_numeric_content {
            // Should not crash due to numeric formatting issues
            assert!(
                output.status.code().is_some(),
                "Should handle numeric formatting consistently"
            );

            // Should not contain locale-specific numeric format errors
            assert!(
                !combined_output.contains("parse"),
                "Should not have numeric parsing errors"
            );
            assert!(
                !combined_output.contains("invalid digit"),
                "Should not have digit parsing errors"
            );
        }
    }

    // Should complete successfully regardless of numeric locale
    assert!(
        !combined_output.contains("locale error"),
        "Should not have locale-related errors"
    );
}
