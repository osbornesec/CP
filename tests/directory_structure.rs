//! Directory structure creation tests
//!
//! These tests verify that sections are organized in proper directory structures

mod common;

use cpinfo_parser::parser::CpinfoParser;
use std::fs;
use std::io::Write;

/// Test 10: Directory structure creation
///
/// This test ensures that extracted sections are organized into proper
/// subdirectories for better file organization.
#[test]
fn test_directory_structure_creation() {
    // Arrange: Create a cpinfo file with sections that should be categorized
    let mut temp_file = tempfile::NamedTempFile::with_suffix(".info").unwrap();
    writeln!(temp_file, "Check Point Support Information").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    // General section (should go in general/ subdirectory)
    writeln!(temp_file, "General Information").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "Version: R81.10").unwrap();
    writeln!(temp_file, "Build: 123456").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    // Network section (should go in network/ subdirectory)
    writeln!(temp_file, "Network Configuration").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "Interface eth0: 192.168.1.1").unwrap();
    writeln!(temp_file, "Gateway: 192.168.1.254").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    // Security section (should go in security/ subdirectory)
    writeln!(temp_file, "Security Policy").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "Firewall rules: enabled").unwrap();
    writeln!(temp_file, "IPS settings: active").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    temp_file.flush().unwrap();
    let file_path = temp_file.path();

    // Create temporary output directory
    let output_dir = tempfile::tempdir().unwrap();
    let output_path = output_dir.path();

    // Act: Extract sections with directory organization
    let parser = CpinfoParser::new();
    let result = parser.extract_sections_organized(file_path, output_path);

    // Assert: Should create organized directory structure
    match result {
        Ok(extraction_result) => {
            assert_eq!(
                extraction_result.sections_extracted, 3,
                "Should extract exactly 3 sections"
            );

            // Check that subdirectories were created
            let general_dir = output_path.join("general");
            let network_dir = output_path.join("network");
            let security_dir = output_path.join("security");

            assert!(
                general_dir.exists(),
                "General subdirectory should be created"
            );
            assert!(
                network_dir.exists(),
                "Network subdirectory should be created"
            );
            assert!(
                security_dir.exists(),
                "Security subdirectory should be created"
            );

            // Check that files are in correct subdirectories
            let general_file = general_dir.join("General_Information.txt");
            let network_file = network_dir.join("Network_Configuration.txt");
            let security_file = security_dir.join("Security_Policy.txt");

            assert!(
                general_file.exists(),
                "General Information file should be in general/ subdirectory"
            );
            assert!(
                network_file.exists(),
                "Network Configuration file should be in network/ subdirectory"
            );
            assert!(
                security_file.exists(),
                "Security Policy file should be in security/ subdirectory"
            );

            // Verify content is still preserved in organized structure
            let general_content = fs::read_to_string(&general_file).unwrap();
            assert!(
                general_content.contains("Version: R81.10"),
                "General Information content should be preserved"
            );

            let network_content = fs::read_to_string(&network_file).unwrap();
            assert!(
                network_content.contains("Interface eth0: 192.168.1.1"),
                "Network Configuration content should be preserved"
            );

            let security_content = fs::read_to_string(&security_file).unwrap();
            assert!(
                security_content.contains("Firewall rules: enabled"),
                "Security Policy content should be preserved"
            );
        }
        Err(e) => panic!("Directory structure creation should succeed, but got error: {e}"),
    }
}

/// Test 11: VSX detection  
///
/// This test ensures that Virtual System Extension (VSX) contexts
/// are properly identified and categorized.
#[test]
fn test_vsx_detection() {
    // Arrange: Create a cpinfo file with VSX-specific content
    let mut temp_file = tempfile::NamedTempFile::with_suffix(".info").unwrap();
    writeln!(temp_file, "Check Point Support Information").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    // VSX status section
    writeln!(temp_file, "VSX Information").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "VSX Status: enabled").unwrap();
    writeln!(temp_file, "Virtual Systems: 5").unwrap();
    writeln!(temp_file, "Management: 192.168.100.1").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    // Virtual System configuration
    writeln!(temp_file, "Virtual System vs1").unwrap();
    writeln!(temp_file, "==============================================").unwrap();
    writeln!(temp_file, "VS ID: 1").unwrap();
    writeln!(temp_file, "VS Name: Production").unwrap();
    writeln!(temp_file, "VS Type: firewall").unwrap();
    writeln!(temp_file, "==============================================").unwrap();

    temp_file.flush().unwrap();
    let file_path = temp_file.path();

    // Create temporary output directory
    let output_dir = tempfile::tempdir().unwrap();
    let output_path = output_dir.path();

    // Act: Extract and detect VSX information
    let parser = CpinfoParser::new();
    let result = parser.extract_sections_with_vsx_detection(file_path, output_path);

    // Assert: Should properly detect and categorize VSX content
    match result {
        Ok(extraction_result) => {
            assert_eq!(
                extraction_result.sections_extracted, 2,
                "Should extract exactly 2 sections"
            );
            assert!(
                extraction_result.vsx_detected,
                "Should detect VSX configuration"
            );
            assert_eq!(
                extraction_result.virtual_systems_count, 5,
                "Should detect 5 virtual systems"
            );

            // Check VSX-specific directory structure
            let vsx_dir = output_path.join("vsx");
            assert!(vsx_dir.exists(), "VSX subdirectory should be created");

            let vsx_info_file = vsx_dir.join("VSX_Information.txt");
            let vs1_file = vsx_dir.join("Virtual_System_vs1.txt");

            assert!(
                vsx_info_file.exists(),
                "VSX Information file should be in vsx/ subdirectory"
            );
            assert!(
                vs1_file.exists(),
                "Virtual System file should be in vsx/ subdirectory"
            );

            // Verify VSX content detection
            let vsx_content = fs::read_to_string(&vsx_info_file).unwrap();
            assert!(
                vsx_content.contains("VSX Status: enabled"),
                "Should preserve VSX status information"
            );
            assert!(
                vsx_content.contains("Virtual Systems: 5"),
                "Should preserve virtual systems count"
            );

            let vs1_content = fs::read_to_string(&vs1_file).unwrap();
            assert!(
                vs1_content.contains("VS ID: 1"),
                "Should preserve virtual system ID"
            );
            assert!(
                vs1_content.contains("VS Type: firewall"),
                "Should preserve virtual system type"
            );
        }
        Err(e) => panic!("VSX detection should succeed, but got error: {e}"),
    }
}
