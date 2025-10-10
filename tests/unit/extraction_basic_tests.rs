//\! Unit tests for extraction::basic module
//\! Tests for the SectionExtractor facade

use cpinfo_parser::extraction::basic::SectionExtractor;
use cpinfo_parser::section_parser::SectionFileParser;
use std::fs;
use tempfile::tempdir;

fn create_test_cpinfo_file() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.cpinfo");

    let content = "Check Point Support Information

==============================================
System Information
==============================================
System: Check Point Security Gateway
Version: R80.40
Build: 12345

==============================================
Network Configuration
==============================================
Interfaces: eth0, eth1
Routes: Default gateway configured
DNS: 8.8.8.8, 8.8.4.4

==============================================
";

    fs::write(&file_path, content).expect("Failed to write test file");
    (temp_dir, file_path)
}

#[test]
fn test_section_extractor_new() {
    let extractor = SectionExtractor::new();
    // Verify creation doesn't panic
    let _ = extractor;
}

#[test]
fn test_section_extractor_default() {
    let extractor = SectionExtractor::default();
    // Verify default creation doesn't panic
    let _ = extractor;
}

#[test]
fn test_extract_sections_basic() {
    let (_temp_input_dir, input_file) = create_test_cpinfo_file();
    let temp_output_dir = tempdir().expect("Failed to create temp output directory");

    let result = SectionExtractor::extract_sections(&input_file, temp_output_dir.path());
    assert\!(result.is_ok());

    let extraction_result = result.unwrap();
    assert_eq\!(extraction_result.sections_extracted, 2);
    assert_eq\!(extraction_result.section_files.len(), 2);
    assert_eq\!(extraction_result.output_directory, temp_output_dir.path());
}

#[test]
fn test_extract_sections_organized() {
    let (_temp_input_dir, input_file) = create_test_cpinfo_file();
    let temp_output_dir = tempdir().expect("Failed to create temp output directory");

    let result = SectionExtractor::extract_sections_organized(&input_file, temp_output_dir.path());
    assert\!(result.is_ok());

    let organized_result = result.unwrap();
    assert_eq\!(organized_result.sections_extracted, 2);
    assert_eq\!(organized_result.section_files.len(), 2);
    assert_eq\!(organized_result.output_directory, temp_output_dir.path());
    assert\!(\!organized_result.directories_created.is_empty());
}

#[test]
fn test_extract_sections_with_vsx_detection() {
    let (_temp_input_dir, input_file) = create_test_cpinfo_file();
    let temp_output_dir = tempdir().expect("Failed to create temp output directory");

    let result = SectionExtractor::extract_sections_with_vsx_detection(&input_file, temp_output_dir.path());
    assert\!(result.is_ok());

    let vsx_result = result.unwrap();
    assert_eq\!(vsx_result.sections_extracted, 2);
    assert_eq\!(vsx_result.section_files.len(), 2);
}

#[test]
fn test_extract_sections_empty_file() {
    let temp_input_dir = tempdir().expect("Failed to create temp input directory");
    let temp_output_dir = tempdir().expect("Failed to create temp output directory");
    let input_file = temp_input_dir.path().join("empty.cpinfo");

    fs::write(&input_file, "No valid sections here").expect("Failed to write test file");

    let result = SectionExtractor::extract_sections(&input_file, temp_output_dir.path());
    assert\!(result.is_ok());

    let extraction_result = result.unwrap();
    assert_eq\!(extraction_result.sections_extracted, 0);
    assert_eq\!(extraction_result.section_files.len(), 0);
}

#[test]
fn test_extract_sections_nonexistent_file() {
    let temp_output_dir = tempdir().expect("Failed to create temp output directory");
    let nonexistent_file = std::path::PathBuf::from("/nonexistent/path/file.cpinfo");

    let result = SectionExtractor::extract_sections(&nonexistent_file, temp_output_dir.path());
    assert\!(result.is_err());
}

#[test]
fn test_extract_sections_multiple_sections() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("multi.cpinfo");

    let content = "==============================================
Section 1
==============================================
Content 1

==============================================
Section 2
==============================================
Content 2

==============================================
Section 3
==============================================
Content 3

==============================================
";

    fs::write(&file_path, content).expect("Failed to write test file");
    let temp_output_dir = tempdir().expect("Failed to create temp output directory");

    let result = SectionExtractor::extract_sections(&file_path, temp_output_dir.path());
    assert\!(result.is_ok());

    let extraction_result = result.unwrap();
    assert_eq\!(extraction_result.sections_extracted, 3);
}

/// Verifies that organized extraction handles long file delimiters and preserves file-section content.
///
/// This test writes a CPInfo-style file containing a very long hyphen (`-`) delimiter used to
/// separate a file section for `/var/log/messages`, runs `SectionExtractor::extract_sections_organized`,
/// and asserts the following:
/// - No organized output file is emitted with a name containing `_var_log_messages`.
/// - A `System_Overview.txt` organized section file exists.
/// - The `System_Overview.txt` content includes the long hyphen delimiter.
/// - `SectionFileParser::parse_section_file` successfully parses the system section and
///   yields a file entry for `/var/log/messages` whose content includes the expected log lines.
///
/// # Examples
///
/// ```
/// // This test demonstrates using the extractor on a temporary CPInfo-like file and parsing
/// // the resulting organized section content with SectionFileParser.
/// ```
#[test]
fn test_extract_sections_organized_handles_file_delimiter() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("file_sections.cpinfo");

    let hyphen_delimiter = "-".repeat(67);
    let content = format!(
        "Check Point Support Information\n\n{eq}\nSystem Overview\n{eq}\nSystem ready\n\n{hy}\n/var/log/messages\n{hy}\nlog entry one\nlog entry two\n\n{eq}\nSummary\n{eq}\nDone\n",
        eq = "=".repeat(46),
        hy = hyphen_delimiter
    );

    fs::write(&file_path, content).expect("Failed to write test file");
    let temp_output_dir = tempdir().expect("Failed to create temp output directory");

    let result = SectionExtractor::extract_sections_organized(&file_path, temp_output_dir.path());
    assert!(result.is_ok());

    let organized_result = result.unwrap();
    assert!(!organized_result.section_files.iter().any(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.contains("_var_log_messages"))
            .unwrap_or(false)
    }));

    let system_section_path = organized_result
        .section_files
        .iter()
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name == "System_Overview.txt")
                .unwrap_or(false)
        })
        .expect("System_Overview section should exist");

    let section_content = fs::read_to_string(system_section_path)
        .expect("Expected to read organized section content");
    assert!(section_content.contains(&hyphen_delimiter));

    let parser = SectionFileParser::new();
    let (_commands, files) = parser
        .parse_section_file(&section_content)
        .expect("Section parsing should succeed");

    let file_section = files
        .iter()
        .find(|file| file.path == "/var/log/messages")
        .expect("/var/log/messages file section should be detected");
    assert!(file_section.content.contains("log entry one"));
    assert!(file_section.content.contains("log entry two"));
}