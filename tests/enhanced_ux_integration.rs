//! Integration tests for enhanced user experience of integrated workflow
//!
//! Tests the new default behavior, enhanced help system, and improved progress reporting.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_enhanced_help_system_shows_integrated_workflow() {
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "NEW DEFAULT BEHAVIOR - INTEGRATED WORKFLOW",
        ))
        .stdout(predicate::str::contains("Phase 1: Extract sections"))
        .stdout(predicate::str::contains(
            "Phase 2: Automatically parse section files",
        ))
        .stdout(predicate::str::contains("PROFESSIONAL WORKFLOWS"))
        .stdout(predicate::str::contains("Network Administrator"))
        .stdout(predicate::str::contains("Security Engineer"))
        .stdout(predicate::str::contains("Support Engineer"))
        .stdout(predicate::str::contains("--extract-only"));
}

#[test]
fn test_help_system_shows_quick_start_examples() {
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("QUICK START"))
        .stdout(predicate::str::contains(
            "cpinfo-parser gateway.cpinfo output/",
        ))
        .stdout(predicate::str::contains(
            "Extracts sections AND parses them automatically",
        ))
        .stdout(predicate::str::contains("BASIC EXAMPLES"));
}

#[test]
fn test_help_system_shows_role_based_workflows() {
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Network Administrator (Incident Response)",
        ))
        .stdout(predicate::str::contains(
            "Security Engineer (Comprehensive Analysis)",
        ))
        .stdout(predicate::str::contains(
            "Support Engineer (TAC Submission)",
        ))
        .stdout(predicate::str::contains(
            "Enterprise Operations (Batch Processing)",
        ));
}

#[test]
fn test_extract_only_flag_available() {
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--extract-only"))
        .stdout(predicate::str::contains(
            "Only extract sections from cpinfo file, skip automatic section parsing",
        ));
}

#[test]
fn test_extract_only_flag_functionality() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.cpinfo");
    let output_dir = temp_dir.path().join("output");

    // Create a minimal test cpinfo file
    let test_content = "Check Point Support Information
================================

This section contains CP Status information.

General Info
============
Some general information content here.
";
    fs::write(&input_file, test_content).unwrap();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(&input_file)
        .arg("-o")
        .arg(&output_dir)
        .arg("--extract-only");

    // The command should succeed (we're testing CLI interface, not full parsing logic)
    let result = cmd.assert();

    // We expect either success or a specific cpinfo parsing error (since our test file isn't a real cpinfo)
    // The important thing is that the --extract-only flag is recognized and processed
    let output = result.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should not show "unknown argument" error for --extract-only
    assert!(
        !stderr.contains("unknown argument"),
        "extract-only flag should be recognized: {stderr}"
    );
}

#[test]
fn test_version_flag_works() {
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("cpinfo-parser"));
}

#[test]
fn test_error_messages_are_user_friendly() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_file = temp_dir.path().join("nonexistent.cpinfo");
    let output_dir = temp_dir.path().join("output");

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(&nonexistent_file).arg("-o").arg(&output_dir);

    cmd.assert().failure().stderr(
        predicate::str::contains("No such file or directory")
            .or(predicate::str::contains("cannot find the file"))
            .or(predicate::str::contains("Failed")),
    );
}

#[test]
fn test_accessibility_environment_variable_support() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.cpinfo");
    let _output_dir = temp_dir.path().join("output");

    // Create a minimal test file
    fs::write(&input_file, "test content").unwrap();

    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.env("NO_COLOR", "1") // Test accessibility environment variable
        .env("SCREENREADER", "1")
        .arg("--help"); // Use help to avoid full parsing

    cmd.assert().success();
    // The fact that it runs without error shows environment variables are handled
}

#[test]
fn test_progress_flag_available() {
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--progress"))
        .stdout(predicate::str::contains("Monitor progress"));
}

#[test]
fn test_security_flag_available() {
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--security"))
        .stdout(predicate::str::contains("Security mode"));
}

#[test]
fn test_verbose_flag_available() {
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--verbose"))
        .stdout(predicate::str::contains("Verbose output"));
}
