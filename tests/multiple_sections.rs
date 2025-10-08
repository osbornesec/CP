//! Multiple section extraction tests
//!
//! These tests verify that the parser can handle cpinfo files with multiple sections

mod common;

use cpinfo_parser::parser::CpinfoParser;
use std::fs;
use std::io::Write;

/// Test 8: Multiple section extraction
///
/// This test ensures that cpinfo files with multiple sections are properly
/// parsed and extracted into separate files.
#[test]
fn test_multiple_section_extraction() {
    // Arrange: Create a cpinfo file with multiple sections
    let mut temp_file = tempfile::NamedTempFile::with_suffix(".info").unwrap();
    writeln!(temp_file, "Check Point Support Information").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    // First section: General Information
    writeln!(temp_file, "General Information").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "Version: R81.10").unwrap();
    writeln!(temp_file, "Build: 123456").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    // Second section: Network Information
    writeln!(temp_file, "Network Information").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "Interface eth0: 192.168.1.1").unwrap();
    writeln!(temp_file, "Gateway: 192.168.1.254").unwrap();
    writeln!(temp_file, "DNS: 8.8.8.8").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    // Third section: Security Configuration
    writeln!(temp_file, "Security Configuration").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "Firewall: enabled").unwrap();
    writeln!(temp_file, "IPS: enabled").unwrap();
    writeln!(temp_file, "VPN: configured").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    temp_file.flush().unwrap();
    let file_path = temp_file.path();

    // Create temporary output directory
    let output_dir = tempfile::tempdir().unwrap();
    let output_path = output_dir.path();

    // Act: Extract sections from the file
    let parser = CpinfoParser::new();
    let result = parser.extract_sections(file_path, output_path);

    // Assert: Should extract all 3 sections
    match result {
        Ok(extraction_result) => {
            assert_eq!(
                extraction_result.sections_extracted, 3,
                "Should extract exactly 3 sections"
            );
            assert_eq!(
                extraction_result.section_files.len(),
                3,
                "Should create 3 section files"
            );

            // Check that all expected section files were created
            let general_info_file = output_path.join("General_Information.txt");
            let network_info_file = output_path.join("Network_Information.txt");
            let security_config_file = output_path.join("Security_Configuration.txt");

            assert!(
                general_info_file.exists(),
                "General Information file should exist"
            );
            assert!(
                network_info_file.exists(),
                "Network Information file should exist"
            );
            assert!(
                security_config_file.exists(),
                "Security Configuration file should exist"
            );

            // Verify content of first section
            let general_content = fs::read_to_string(&general_info_file).unwrap();
            assert!(
                general_content.contains("Version: R81.10"),
                "General Information should contain version"
            );
            assert!(
                general_content.contains("Build: 123456"),
                "General Information should contain build"
            );

            // Verify content of second section
            let network_content = fs::read_to_string(&network_info_file).unwrap();
            assert!(
                network_content.contains("Interface eth0: 192.168.1.1"),
                "Network Information should contain interface info"
            );
            assert!(
                network_content.contains("Gateway: 192.168.1.254"),
                "Network Information should contain gateway"
            );
            assert!(
                network_content.contains("DNS: 8.8.8.8"),
                "Network Information should contain DNS"
            );

            // Verify content of third section
            let security_content = fs::read_to_string(&security_config_file).unwrap();
            assert!(
                security_content.contains("Firewall: enabled"),
                "Security Configuration should contain firewall status"
            );
            assert!(
                security_content.contains("IPS: enabled"),
                "Security Configuration should contain IPS status"
            );
            assert!(
                security_content.contains("VPN: configured"),
                "Security Configuration should contain VPN status"
            );

            // Verify no cross-contamination between sections
            assert!(
                !general_content.contains("Interface eth0"),
                "General Information should not contain network info"
            );
            assert!(
                !network_content.contains("Version: R81.10"),
                "Network Information should not contain general info"
            );
            assert!(
                !security_content.contains("Gateway: 192.168.1.254"),
                "Security Configuration should not contain network info"
            );
        }
        Err(e) => panic!("Multiple section extraction should succeed, but got error: {e}"),
    }
}

/// Test 9: Section name sanitization
///
/// This test ensures that section names with special characters are properly
/// sanitized for use as filenames.
#[test]
fn test_section_name_sanitization() {
    // Arrange: Create a cpinfo file with problematic section names
    let mut temp_file = tempfile::NamedTempFile::with_suffix(".info").unwrap();
    writeln!(temp_file, "Check Point Support Information").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    // Section with special characters that need sanitization
    writeln!(temp_file, "Network/Interface: Configuration").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "eth0: 192.168.1.1").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    // Section with multiple special characters
    writeln!(temp_file, "Security: Firewall/IPS Status").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "Status: Active").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    temp_file.flush().unwrap();
    let file_path = temp_file.path();

    // Create temporary output directory
    let output_dir = tempfile::tempdir().unwrap();
    let output_path = output_dir.path();

    // Act: Extract sections from the file
    let parser = CpinfoParser::new();
    let result = parser.extract_sections(file_path, output_path);

    // Assert: Should sanitize section names properly
    match result {
        Ok(extraction_result) => {
            assert_eq!(
                extraction_result.sections_extracted, 2,
                "Should extract exactly 2 sections"
            );

            // Check that files were created with sanitized names
            let sanitized_network_file = output_path.join("Network_Interface__Configuration.txt");
            let sanitized_security_file = output_path.join("Security__Firewall_IPS_Status.txt");

            assert!(
                sanitized_network_file.exists(),
                "Network file with sanitized name should exist: {sanitized_network_file:?}"
            );
            assert!(
                sanitized_security_file.exists(),
                "Security file with sanitized name should exist: {sanitized_security_file:?}"
            );

            // Verify content is still correct despite name sanitization
            let network_content = fs::read_to_string(&sanitized_network_file).unwrap();
            assert!(
                network_content.contains("eth0: 192.168.1.1"),
                "Content should be preserved despite name sanitization"
            );

            let security_content = fs::read_to_string(&sanitized_security_file).unwrap();
            assert!(
                security_content.contains("Status: Active"),
                "Content should be preserved despite name sanitization"
            );
        }
        Err(e) => panic!("Section name sanitization test should succeed, but got error: {e}"),
    }
}
