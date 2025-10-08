// Test 65: Real R81.10 cpinfo file end-to-end processing
// Phase 7: Integration and End-to-End Tests
//
// Canon TDD Cycle 65: Should successfully process real R81.10 cpinfo file end-to-end
// Purpose: Real-world validation with complete extraction of all sections

use cpinfo_parser::CpinfoParser;
use std::fs;
use tempfile::TempDir;

mod common;

/// Test 65: Should successfully process real R81.10 cpinfo file end-to-end
///
/// This test validates that the parser can handle a realistic R81.10 cpinfo file
/// from start to finish, extracting all sections and organizing them correctly.
#[test]
fn test_65_should_process_real_r81_end_to_end() {
    // Arrange: Create a realistic R81.10 cpinfo file
    let test_file = create_realistic_r81_cpinfo_file();
    let output_dir = TempDir::new().expect("Failed to create temp output directory");

    let parser = CpinfoParser::new();

    // Act: Process the cpinfo file end-to-end
    let result = parser.parse_file_end_to_end(test_file.path(), output_dir.path());

    // Assert: Complete processing with all expected sections
    match result {
        Ok(processing_result) => {
            // Verify all major Check Point sections were extracted
            assert!(
                processing_result.sections_extracted >= 10,
                "Should extract at least 10 major sections from R81.10 cpinfo, got: {}",
                processing_result.sections_extracted
            );

            // Verify version detection worked correctly
            assert!(
                processing_result.version_info.is_some(),
                "Should detect Check Point version information"
            );
            let version_info = processing_result.version_info.unwrap();
            assert!(
                version_info.version.starts_with("R81"),
                "Should detect R81.x version, got: {}",
                version_info.version
            );

            // Verify output directory structure was created
            assert!(output_dir.path().exists(), "Output directory should exist");

            // Verify key directories were created
            let general_dir = output_dir.path().join("general");
            let network_dir = output_dir.path().join("network");
            let security_dir = output_dir.path().join("security");

            assert!(
                general_dir.exists(),
                "General sections directory should be created"
            );
            assert!(
                network_dir.exists(),
                "Network sections directory should be created"
            );
            assert!(
                security_dir.exists(),
                "Security sections directory should be created"
            );

            // Verify section files were created
            let general_files = fs::read_dir(&general_dir)
                .expect("Should be able to read general directory")
                .count();
            assert!(
                general_files >= 3,
                "Should create at least 3 general section files, got: {general_files}"
            );

            // Verify processing statistics
            assert!(
                processing_result.processing_time_ms > 0,
                "Should record processing time"
            );
            assert!(
                processing_result.total_file_size > 1000,
                "Test file should be substantial size (>1KB)"
            );

            // Verify no critical errors occurred
            assert!(
                processing_result.critical_errors == 0,
                "Should have no critical errors during processing"
            );
        }
        Err(e) => {
            panic!("R81.10 cpinfo file processing should succeed, but got error: {e}");
        }
    }
}

/// Create a realistic R81.10 cpinfo file for end-to-end testing
///
/// This creates a test file that mimics the structure and content of a real
/// Check Point R81.10 diagnostic file with all major sections.
fn create_realistic_r81_cpinfo_file() -> tempfile::NamedTempFile {
    let cpinfo_content = "
CPInfo collection utility for Check Point NG

Products: VPN-1 & Firewall-1

Product: VPN-1 & Firewall-1
Version: R81.10 - Build 016

Checking environment...
[OK]

==============================================
General Info
==============================================

Check Point Information:
Version: R81.10 - Build 016
kernel: R81.10 - Build 016
OS Information: SecurePlatform
Platform: Linux secureplatform 2.6.18-194.el5 #1 SMP Fri Apr 2 13:34:04 EDT 2010 x86_64

Product Installation: Standard 
Product Features: None

Enabled blades
--------------------------
fw vpn urlf appi ips identityServer mon

==============================================
Network Configuration
==============================================

Network Information:
Total Interfaces: 4

Interface: eth0
IP Address: 192.168.1.10
Subnet Mask: 255.255.255.0
State: Up
MTU: 1500

Interface: eth1
IP Address: 10.0.0.5
Subnet Mask: 255.255.0.0
State: Up
MTU: 1500

Interface: eth2
IP Address: 172.16.100.1
Subnet Mask: 255.255.255.0
State: Down
MTU: 1500

Interface: mgmt
IP Address: 192.168.100.50
Subnet Mask: 255.255.255.0
State: Up
MTU: 1500

==============================================
Security Policies
==============================================

Security Policy Information:
Total Rules: 25
Allow Rules: 18
Drop Rules: 7

Rule 1:
Name: Allow HTTPS to Web Servers
Action: Allow
Source: Any
Destination: WebServers_Group
Service: HTTPS

Rule 2:
Name: Allow SSH to Management
Action: Allow
Source: AdminNetwork
Destination: Management_Servers
Service: SSH

Rule 3:
Name: Block suspicious traffic
Action: Drop
Source: External_Untrusted
Destination: Any
Service: Any

==============================================
VPN Configuration
==============================================

VPN Information:
Total Tunnels: 3
Active Tunnels: 2
Remote Access: Enabled

Tunnel: Site-to-Site-Branch1
Remote Peer: 203.0.113.100
Status: Active
Encryption: AES-256
Authentication: SHA-256

Tunnel: Site-to-Site-Branch2
Remote Peer: 198.51.100.50
Status: Active
Encryption: AES-128
Authentication: SHA-1

Tunnel: Partner-Connection
Remote Peer: 192.0.2.10
Status: Inactive
Encryption: 3DES
Authentication: MD5

==============================================
High Availability
==============================================

HA Information:
HA Enabled: true
Local State: Active
Peer State: Standby
Sync Status: Synchronized
Failover Mode: New High Availability

==============================================
Log Information
==============================================

Log File Information:
Total Log Types: 4
Total Size: 2048 MB
Oldest Entry: 2024-01-01 00:00:00

Log Type: fw
Log Type: vpn  
Log Type: audit
Log Type: system

==============================================
Certificate Information
==============================================

Certificate Information:
Total Certificates: 3
Valid Certificates: 2
Expired Certificates: 1

Certificate: Internal_CA_Certificate
Issuer: Check Point Internal CA
Subject: CN=Gateway.company.com
Status: Valid
Expires: 2025-12-31

Certificate: HTTPS_Server_Certificate
Issuer: DigiCert Global Root CA
Subject: CN=*.company.com
Status: Valid
Expires: 2024-08-15

Certificate: Legacy_Certificate
Issuer: VeriSign Class 3 CA
Subject: CN=old.company.com
Status: Expired
Expires: 2023-03-01

==============================================
Performance Information
==============================================

Performance Statistics:
CPU Usage: 45.2%
Memory Usage: 67.8%
Disk Usage: 23.1%
Connections per Second: 1250
Throughput: 890.5 Mbps

==============================================
System Information
==============================================

Hardware Information:
CPU: Intel Xeon E5-2680 v4 @ 2.40GHz
Memory: 64 GB
Disk Space: 2 TB SSD
Network Cards: 4x Intel 82599ES

Operating System:
Distribution: Gaia R81.10
Kernel Version: 3.10.0-957.el7.x86_64
Architecture: x86_64
Uptime: 45 days, 12 hours, 23 minutes

==============================================
Cluster Information
==============================================

Cluster Configuration:
Cluster Type: HA
Member Count: 2

Local Member: gateway-primary
Local State: Active

Member 1:
Name: gateway-primary
State: Active
IP: 192.168.1.10
Priority: 100

Member 2:
Name: gateway-secondary
State: Standby
IP: 192.168.1.11
Priority: 90

==============================================
End of CPInfo Collection
==============================================
";

    common::create_test_cpinfo_file(cpinfo_content)
}
