//\! Unit tests for extraction::basic_extraction module

use cpinfo_parser::extraction::basic_extraction::extract_sections;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_extract_sections_with_content() {
    let temp_input_dir = tempdir().expect("Failed to create temp input directory");
    let temp_output_dir = tempdir().expect("Failed to create temp output directory");
    let input_file = temp_input_dir.path().join("test.cpinfo");

    let content = "Check Point Support Information\n\n==============================================\nSystem Information\n==============================================\nSystem: Check Point Security Gateway\nVersion: R80.40\nBuild: 12345\n\n==============================================\nNetwork Configuration\n==============================================\nInterfaces: eth0, eth1\nRoutes: Default gateway configured\nDNS: 8.8.8.8, 8.8.4.4\n\n==============================================\n";

    fs::write(&input_file, content).expect("Failed to write test file");

    let result = extract_sections(&input_file, temp_output_dir.path());
    assert\!(result.is_ok());

    let extraction_result = result.unwrap();
    assert_eq\!(extraction_result.sections_extracted, 2);
    assert_eq\!(extraction_result.section_files.len(), 2);

    // Verify files were created
    let system_file = temp_output_dir.path().join("System_Information.txt");
    let network_file = temp_output_dir.path().join("Network_Configuration.txt");

    assert\!(system_file.exists());
    assert\!(network_file.exists());

    // Verify content
    let system_content = fs::read_to_string(&system_file).expect("Failed to read system file");
    assert\!(system_content.contains("Check Point Security Gateway"));
    assert\!(system_content.contains("Version: R80.40"));

    let network_content = fs::read_to_string(&network_file).expect("Failed to read network file");
    assert\!(network_content.contains("Interfaces: eth0, eth1"));
    assert\!(network_content.contains("DNS: 8.8.8.8, 8.8.4.4"));
}

#[test]
fn test_extract_sections_empty_sections() {
    let temp_input_dir = tempdir().expect("Failed to create temp input directory");
    let temp_output_dir = tempdir().expect("Failed to create temp output directory");
    let input_file = temp_input_dir.path().join("test.cpinfo");

    let content = "==============================================\nEmpty Section\n==============================================\n\n==============================================\n";

    fs::write(&input_file, content).expect("Failed to write test file");

    let result = extract_sections(&input_file, temp_output_dir.path());
    assert\!(result.is_ok());

    let extraction_result = result.unwrap();
    // Empty sections should not be extracted
    assert_eq\!(extraction_result.sections_extracted, 0);
}

#[test]
fn test_extract_sections_creates_output_directory() {
    let temp_input_dir = tempdir().expect("Failed to create temp input directory");
    let temp_base = tempdir().expect("Failed to create temp base directory");
    let input_file = temp_input_dir.path().join("test.cpinfo");
    let output_dir = temp_base.path().join("new_dir");

    let content = "==============================================\nTest Section\n==============================================\nTest content\n==============================================\n";

    fs::write(&input_file, content).expect("Failed to write test file");

    assert\!(\!output_dir.exists());

    let result = extract_sections(&input_file, &output_dir);
    assert\!(result.is_ok());
    assert\!(output_dir.exists());
}

#[test]
fn test_extract_sections_with_special_characters() {
    let temp_input_dir = tempdir().expect("Failed to create temp input directory");
    let temp_output_dir = tempdir().expect("Failed to create temp output directory");
    let input_file = temp_input_dir.path().join("test.cpinfo");

    let content = "==============================================\nSection/With\\Special:Chars*?\n==============================================\nContent\n==============================================\n";

    fs::write(&input_file, content).expect("Failed to write test file");

    let result = extract_sections(&input_file, temp_output_dir.path());
    assert\!(result.is_ok());

    let extraction_result = result.unwrap();
    assert_eq\!(extraction_result.sections_extracted, 1);

    // Verify sanitized filename
    let sanitized_file = temp_output_dir.path().join("Section_With_Special_Chars__.txt");
    assert\!(sanitized_file.exists());
}

#[test]
fn test_extract_sections_preserves_formatting() {
    let temp_input_dir = tempdir().expect("Failed to create temp input directory");
    let temp_output_dir = tempdir().expect("Failed to create temp output directory");
    let input_file = temp_input_dir.path().join("test.cpinfo");

    let content = "==============================================\nFormatted Section\n==============================================\nLine 1\n  Indented Line 2\n    More indented Line 3\n\nLine after blank\n==============================================\n";

    fs::write(&input_file, content).expect("Failed to write test file");

    let result = extract_sections(&input_file, temp_output_dir.path());
    assert\!(result.is_ok());

    let output_file = temp_output_dir.path().join("Formatted_Section.txt");
    let content = fs::read_to_string(&output_file).expect("Failed to read output file");

    assert\!(content.contains("Line 1"));
    assert\!(content.contains("  Indented Line 2"));
    assert\!(content.contains("    More indented Line 3"));
}