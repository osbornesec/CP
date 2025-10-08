//! Check Point domain-specific tests
//!
//! These tests cover Check Point specific functionality (Phase 1, Tests 12-15)

mod common;

use cpinfo_parser::checkpoint::CheckPointParser;
use std::io::Write;
use tempfile::NamedTempFile;

/// Test 12: Check Point version detection
///
/// Should parse "Version: R81.10 - Build 029" and similar patterns
/// This test should FAIL initially because `CheckPointParser` is not implemented
#[test]
fn test_should_detect_checkpoint_version_r8110() {
    // Arrange: Create cpinfo file with R81.10 version
    let temp_file = create_r8110_cpinfo_file();
    let file_path = temp_file.path();

    // Act: Parse version information
    let result = CheckPointParser::parse_version(file_path);

    // Assert: Should detect R81.10 version with build number
    match result {
        Ok(version_info) => {
            assert_eq!(version_info.version, "R81.10");
            assert_eq!(version_info.build, "029");
            assert_eq!(version_info.kernel_version, Some("R81.10".to_owned()));
            assert_eq!(version_info.kernel_build, Some("030".to_owned()));
        }
        Err(e) => panic!("R81.10 version should be detected, but got error: {e}"),
    }
}

#[test]
fn test_should_detect_checkpoint_version_r8120() {
    // Arrange: Create cpinfo file with R81.20 version
    let temp_file = create_r8120_cpinfo_file();
    let file_path = temp_file.path();

    // Act: Parse version information
    let result = CheckPointParser::parse_version(file_path);

    // Assert: Should detect R81.20 version with build number
    match result {
        Ok(version_info) => {
            assert_eq!(version_info.version, "R81.20");
            assert_eq!(version_info.build, "045");
            assert_eq!(version_info.kernel_version, Some("R81.20".to_owned()));
            assert_eq!(version_info.kernel_build, Some("052".to_owned()));
        }
        Err(e) => panic!("R81.20 version should be detected, but got error: {e}"),
    }
}

/// Test with actual sample files
#[test]
fn test_real_files_version_detection() {
    // Test R81.10 file
    let result_8110 = CheckPointParser::parse_version("/mnt/d/CP/samples/FW1cpinfo.info");
    match result_8110 {
        Ok(version_info) => {
            assert_eq!(version_info.version, "R81.10");
            assert_eq!(version_info.build, "029");
        }
        Err(e) => println!("R81.10 parsing failed: {e}"),
    }

    // Test R81.20 file
    let result_8120 = CheckPointParser::parse_version("/mnt/d/CP/samples/FW1r8120cpinfo.info");
    match result_8120 {
        Ok(version_info) => {
            assert_eq!(version_info.version, "R81.20");
            assert_eq!(version_info.build, "045");
        }
        Err(e) => println!("R81.20 parsing failed: {e}"),
    }
}

/// Test security blades with real sample files
#[test]
fn test_real_files_security_blades() {
    // Test with R81.20 file
    let result = CheckPointParser::parse_security_blades("/mnt/d/CP/samples/FW1r8120cpinfo.info");
    match result {
        Ok(blades) => {
            assert!(
                blades.firewall_enabled(),
                "Firewall should be enabled in sample file"
            );
            assert!(blades.vpn_enabled(), "VPN should be enabled in sample file");
            assert!(
                blades.blade_count() > 0,
                "Should have at least one blade enabled"
            );
            println!("Sample file blades: {blades:?}");
        }
        Err(e) => println!("Blade parsing failed: {e}"),
    }
}

/// Test 13: Security blade identification
///
/// Should detect "fw vpn urlf appi ips identityServer mon" blade configurations
/// This test should FAIL initially because blade parsing is not implemented
#[test]
fn test_should_identify_security_blades() {
    // Arrange: Create cpinfo file with security blades enabled
    let temp_file = create_cpinfo_with_blades();
    let file_path = temp_file.path();

    // Act: Parse security blade information
    let result = CheckPointParser::parse_security_blades(file_path);

    // Assert: Should identify all enabled blades
    match result {
        Ok(blades) => {
            assert!(
                blades.firewall_enabled(),
                "Firewall blade should be enabled"
            );
            assert!(blades.vpn_enabled(), "VPN blade should be enabled");
            assert!(
                blades.url_filtering_enabled(),
                "URL filtering should be enabled"
            );
            assert!(
                blades.application_control_enabled,
                "Application control should be enabled"
            );
            assert!(blades.ips_enabled, "IPS blade should be enabled");
            assert!(
                blades.identity_server_enabled,
                "Identity Server should be enabled"
            );
            assert!(blades.monitoring_enabled, "Monitoring should be enabled");
            assert_eq!(blades.blade_count, 7);
        }
        Err(e) => panic!("Security blades should be detected, but got error: {e}"),
    }
}

/// Test 14: Large file streaming
///
/// Should handle files >100MB without loading full content into memory
/// This test should FAIL initially because streaming parser is not implemented
#[test]
fn test_should_handle_large_file_streaming() {
    // Arrange: Use one of the large sample files (>100MB)
    let large_file_path = "/mnt/d/CP/samples/FW1r8120cpinfo.info"; // 111MB

    // Act: Parse using streaming approach
    let result = CheckPointParser::parse_streaming(large_file_path);

    // Assert: Should parse successfully without memory errors
    match result {
        Ok(parsing_result) => {
            assert!(
                parsing_result.file_size > 100_000_000,
                "Should handle files >100MB"
            );
            assert!(
                parsing_result.sections_found > 0,
                "Should find sections in large file"
            );
            assert!(
                parsing_result.memory_peak_mb < 100,
                "Should use <100MB memory"
            );
        }
        Err(e) => panic!("Large file streaming should work, but got error: {e}"),
    }
}

/// Test 15: Memory usage validation
///
/// Should maintain <100MB memory usage regardless of file size
/// This test should FAIL initially because memory profiling is not implemented
#[test]
fn test_should_validate_memory_usage() {
    // Arrange: Use a reasonably large file (111MB) for practical testing
    let huge_file_path = "/mnt/d/CP/samples/FW1r8120cpinfo.info";

    // Act: Parse with memory monitoring
    let result = CheckPointParser::parse_with_memory_monitoring(huge_file_path);

    // Assert: Should maintain low memory usage
    match result {
        Ok(memory_stats) => {
            assert!(
                memory_stats.peak_memory_mb < 100,
                "Peak memory should be <100MB, but was {}MB",
                memory_stats.peak_memory_mb
            );
            assert!(
                memory_stats.final_memory_mb < 50,
                "Final memory should be <50MB, but was {}MB",
                memory_stats.final_memory_mb
            );
            assert_eq!(
                memory_stats.memory_leaks_detected, 0,
                "No memory leaks should be detected"
            );
        }
        Err(e) => panic!("Memory usage validation should work, but got error: {e}"),
    }
}

// Test helpers
fn create_r8110_cpinfo_file() -> NamedTempFile {
    let mut temp_file =
        NamedTempFile::with_suffix(".info").expect("Failed to create temporary file");

    write!(
        temp_file,
        "************************************************************************
                       Check Point Support Information                    
                           CPinfo 5.0 Build 914000248                           

                        (Last Mod.: Apr 21 2024 11:25:40)                              

************************************************************************

==============================================
General Info
==============================================
OS: Gaia
Version: R81.10 - Build 029
kernel: R81.10 - Build 030
Type: GW cluster

==============================================
Enabled blades
==============================================
fw vpn urlf appi ips identityServer mon
"
    )
    .expect("Failed to write test content");

    temp_file
}

fn create_r8120_cpinfo_file() -> NamedTempFile {
    let mut temp_file =
        NamedTempFile::with_suffix(".info").expect("Failed to create temporary file");

    write!(
        temp_file,
        "************************************************************************
                       Check Point Support Information                    
                           CPinfo 5.0 Build 914000250                           

                        (Last Mod.: Aug  8 2024 13:14:13)                              

************************************************************************

==============================================
General Info
==============================================
OS: Gaia
Version: R81.20 - Build 045
kernel: R81.20 - Build 052
Type: GW cluster

==============================================
Enabled blades
==============================================
fw vpn urlf appi ips identityServer mon
"
    )
    .expect("Failed to write test content");

    temp_file
}

fn create_cpinfo_with_blades() -> NamedTempFile {
    let mut temp_file =
        NamedTempFile::with_suffix(".info").expect("Failed to create temporary file");

    write!(
        temp_file,
        "************************************************************************
                       Check Point Support Information                    

************************************************************************

==============================================
General Info
==============================================
OS: Gaia
Version: R81.20 - Build 045
Type: GW cluster

==============================================
Enabled blades
==============================================
fw vpn urlf appi ips identityServer mon

==============================================
Security Details
==============================================
Firewall: Enabled
VPN: Enabled
URL Filtering: Enabled
Application Control: Enabled
IPS: Enabled
Identity Server: Enabled
Monitoring: Enabled
"
    )
    .expect("Failed to write test content");

    temp_file
}

/// Test 16: VSX Virtual System context parsing
///
/// Should detect and organize VSX contexts (VS 0, VS 1, VS 2, etc.)
/// This test should FAIL initially because VSX parsing is not implemented
#[test]
fn test_should_detect_vsx_deployment() {
    // Arrange: Create cpinfo file with VSX contexts
    let temp_file = create_vsx_cpinfo_file();
    let file_path = temp_file.path();

    // Act: Parse VSX deployment information
    let result = CheckPointParser::parse_vsx_deployment(file_path);

    // Assert: Should detect VSX deployment with multiple contexts
    match result {
        Ok(vsx_info) => {
            assert_eq!(vsx_info.deployment_type, "VSX");
            assert_eq!(vsx_info.virtual_systems.len(), 3); // VS 0, VS 1, VS 2

            // Verify VS 0 (Management context)
            let vs0 = vsx_info
                .virtual_systems
                .iter()
                .find(|vs| vs.id == 0)
                .unwrap();
            assert_eq!(vs0.context_type, "Management");
            assert_eq!(vs0.name, "VSX Gateway");

            // Verify VS 1 (Customer context)
            let vs1 = vsx_info
                .virtual_systems
                .iter()
                .find(|vs| vs.id == 1)
                .unwrap();
            assert_eq!(vs1.context_type, "Customer");
            assert_eq!(vs1.name, "Customer_A");

            // Verify VS 2 (Customer context)
            let vs2 = vsx_info
                .virtual_systems
                .iter()
                .find(|vs| vs.id == 2)
                .unwrap();
            assert_eq!(vs2.context_type, "Customer");
            assert_eq!(vs2.name, "Customer_B");
        }
        Err(e) => panic!("VSX deployment should be detected, but got error: {e}"),
    }
}

/// Test 17: Cluster configuration detection
///
/// Should detect cluster membership and state information
/// This test should FAIL initially because cluster parsing is not implemented
#[test]
fn test_should_detect_cluster_configuration() {
    // Arrange: Create cpinfo file with cluster information
    let temp_file = create_cluster_cpinfo_file();
    let file_path = temp_file.path();

    // Act: Parse cluster configuration
    let result = CheckPointParser::parse_cluster_configuration(file_path);

    // Assert: Should detect cluster details
    match result {
        Ok(cluster_info) => {
            assert_eq!(cluster_info.cluster_type, "HA");
            assert_eq!(cluster_info.member_count, 2);
            assert_eq!(cluster_info.local_member.name, "member_1");
            assert_eq!(cluster_info.local_member.state, "Active");
            assert_eq!(cluster_info.remote_members.len(), 1);
            assert_eq!(cluster_info.remote_members[0].name, "member_2");
            assert_eq!(cluster_info.remote_members[0].state, "Standby");
        }
        Err(e) => panic!("Cluster configuration should be detected, but got error: {e}"),
    }
}

/// Test 18: Security policy rule extraction
///
/// Should extract and categorize firewall rules
/// This test should FAIL initially because policy parsing is not implemented
#[test]
fn test_should_extract_security_policies() {
    // Arrange: Create cpinfo file with security policies
    let temp_file = create_policy_cpinfo_file();
    let file_path = temp_file.path();

    // Act: Parse security policies
    let result = CheckPointParser::parse_security_policies(file_path);

    // Assert: Should extract policy rules
    match result {
        Ok(policies) => {
            assert_eq!(policies.total_rules, 5);
            assert_eq!(policies.allow_rules, 3);
            assert_eq!(policies.drop_rules, 2);
            assert!(policies.rules.iter().any(|r| r.name == "Allow_HTTP"));
            assert!(policies.rules.iter().any(|r| r.action == "Drop"));
        }
        Err(e) => panic!("Security policies should be extracted, but got error: {e}"),
    }
}

// Test helper functions for Phase 2 tests

fn create_vsx_cpinfo_file() -> NamedTempFile {
    let mut temp_file =
        NamedTempFile::with_suffix(".info").expect("Failed to create temporary file");

    write!(
        temp_file,
        "************************************************************************
                       Check Point Support Information                    

************************************************************************

==============================================
General Info
==============================================
OS: Gaia
Version: R81.20 - Build 045
Type: VSX Gateway
Number of Virtual Systems: 3

==============================================
VS 0 (VSX Gateway)
==============================================
Virtual System ID: 0
Context Type: Management
Name: VSX Gateway
State: Active

==============================================
VS 1 (Customer_A)
==============================================
Virtual System ID: 1
Context Type: Customer
Name: Customer_A
State: Active
Interfaces: eth1, eth2

==============================================
VS 2 (Customer_B)
==============================================
Virtual System ID: 2
Context Type: Customer
Name: Customer_B
State: Active
Interfaces: eth3, eth4

==============================================
VSX Status
==============================================
VSX Enabled: true
Virtual Systems: 3
Management Context: VS 0
"
    )
    .expect("Failed to write test content");

    temp_file
}

fn create_cluster_cpinfo_file() -> NamedTempFile {
    let mut temp_file =
        NamedTempFile::with_suffix(".info").expect("Failed to create temporary file");

    write!(
        temp_file,
        "************************************************************************
                       Check Point Support Information                    

************************************************************************

==============================================
General Info
==============================================
OS: Gaia
Version: R81.20 - Build 045
Type: HA Cluster

==============================================
Cluster Status
==============================================
Cluster Type: HA
Member Count: 2
Local Member: member_1
Local State: Active

==============================================
Cluster Members
==============================================
Member 1:
  Name: member_1
  State: Active
  IP: 192.168.1.10
  Priority: 100

Member 2:
  Name: member_2
  State: Standby
  IP: 192.168.1.11
  Priority: 90

==============================================
HA Configuration
==============================================
Synchronization: Enabled
Failover Mode: New High Availability
"
    )
    .expect("Failed to write test content");

    temp_file
}

fn create_policy_cpinfo_file() -> NamedTempFile {
    let mut temp_file =
        NamedTempFile::with_suffix(".info").expect("Failed to create temporary file");

    write!(
        temp_file,
        "************************************************************************
                       Check Point Support Information                    

************************************************************************

==============================================
Security Policy Rules
==============================================
Total Rules: 5
Allow Rules: 3
Drop Rules: 2

Rule 1:
  Name: Allow_HTTP
  Action: Accept
  Source: Any
  Destination: Web_Servers
  Service: HTTP

Rule 2:
  Name: Allow_HTTPS
  Action: Accept
  Source: Any
  Destination: Web_Servers
  Service: HTTPS

Rule 3:
  Name: Allow_SSH_Admin
  Action: Accept
  Source: Admin_Network
  Destination: Servers
  Service: SSH

Rule 4:
  Name: Drop_P2P
  Action: Drop
  Source: Any
  Destination: Any
  Service: P2P_Applications

Rule 5:
  Name: Drop_Malware
  Action: Drop
  Source: Any
  Destination: Any
  Service: Known_Malware
"
    )
    .expect("Failed to write test content");

    temp_file
}

/// Test 19: Network interface configuration parsing
///
/// Should parse network interface configurations and states
/// This test should FAIL initially because network parsing is not implemented
#[test]
fn test_should_parse_network_interfaces() {
    // Arrange: Create cpinfo file with network interface information
    let temp_file = create_network_cpinfo_file();
    let file_path = temp_file.path();

    // Act: Parse network interface configuration
    let result = CheckPointParser::parse_network_interfaces(file_path);

    // Assert: Should extract interface configurations
    match result {
        Ok(network_config) => {
            assert_eq!(network_config.total_interfaces, 4);
            assert_eq!(network_config.interfaces.len(), 4);

            // Verify eth0 interface
            let eth0 = network_config
                .interfaces
                .iter()
                .find(|i| i.name == "eth0")
                .unwrap();
            assert_eq!(eth0.ip_address, "192.168.1.10");
            assert_eq!(eth0.subnet_mask, "255.255.255.0");
            assert_eq!(eth0.state, "Up");

            // Verify management interface
            let mgmt = network_config
                .interfaces
                .iter()
                .find(|i| i.name == "mgmt")
                .unwrap();
            assert_eq!(mgmt.ip_address, "10.0.0.5");
            assert_eq!(mgmt.state, "Up");
        }
        Err(e) => panic!("Network interfaces should be parsed, but got error: {e}"),
    }
}

/// Test 20: VPN configuration and tunnel analysis
///
/// Should detect VPN tunnels, remote access configurations
/// This test should FAIL initially because VPN parsing is not implemented
#[test]
fn test_should_analyze_vpn_configuration() {
    // Arrange: Create cpinfo file with VPN information
    let temp_file = create_vpn_cpinfo_file();
    let file_path = temp_file.path();

    // Act: Parse VPN configuration
    let result = CheckPointParser::parse_vpn_configuration(file_path);

    // Assert: Should extract VPN details
    match result {
        Ok(vpn_config) => {
            assert_eq!(vpn_config.total_tunnels, 3);
            assert_eq!(vpn_config.active_tunnels, 2);
            assert!(vpn_config.remote_access_enabled);
            assert_eq!(vpn_config.tunnels.len(), 3);

            // Verify specific tunnel
            let tunnel1 = vpn_config
                .tunnels
                .iter()
                .find(|t| t.name == "Site_A_Tunnel")
                .unwrap();
            assert_eq!(tunnel1.remote_peer, "203.0.113.5");
            assert_eq!(tunnel1.status, "Up");
            assert_eq!(tunnel1.encryption, "AES-256");
        }
        Err(e) => panic!("VPN configuration should be parsed, but got error: {e}"),
    }
}

/// Test 21: High Availability status and cluster member detection
///
/// Should detect HA configuration and member states
/// This test should FAIL initially because HA parsing is not implemented
#[test]
fn test_should_detect_ha_status() {
    // Arrange: Create cpinfo file with HA information
    let temp_file = create_ha_cpinfo_file();
    let file_path = temp_file.path();

    // Act: Parse HA status
    let result = CheckPointParser::parse_ha_status(file_path);

    // Assert: Should extract HA details
    match result {
        Ok(ha_status) => {
            assert!(ha_status.ha_enabled);
            assert_eq!(ha_status.local_state, "Active");
            assert_eq!(ha_status.peer_state, "Standby");
            assert_eq!(ha_status.sync_status, "In Sync");
            assert_eq!(ha_status.failover_mode, "New High Availability");
        }
        Err(e) => panic!("HA status should be parsed, but got error: {e}"),
    }
}

/// Test 22: Log file section identification and metadata extraction
///
/// Should identify different log types and extract metadata
/// This test should FAIL initially because log parsing is not implemented
#[test]
fn test_should_identify_log_sections() {
    // Arrange: Create cpinfo file with log sections
    let temp_file = create_log_cpinfo_file();
    let file_path = temp_file.path();

    // Act: Parse log sections
    let result = CheckPointParser::parse_log_sections(file_path);

    // Assert: Should identify log types and metadata
    match result {
        Ok(log_info) => {
            assert_eq!(log_info.total_log_types, 4);
            assert!(log_info.log_types.contains(&"Firewall".to_owned()));
            assert!(log_info.log_types.contains(&"VPN".to_owned()));
            assert!(log_info.log_types.contains(&"System".to_owned()));
            assert!(log_info.log_types.contains(&"Audit".to_owned()));

            // Verify log metadata
            assert!(log_info.total_size_mb > 0);
            assert!(log_info.oldest_entry.contains("2024"));
        }
        Err(e) => panic!("Log sections should be identified, but got error: {e}"),
    }
}

/// Test 23: Certificate and PKI information parsing
///
/// Should extract certificate details and PKI configuration
/// This test should FAIL initially because certificate parsing is not implemented
#[test]
fn test_should_parse_certificate_info() {
    // Arrange: Create cpinfo file with certificate information
    let temp_file = create_certificate_cpinfo_file();
    let file_path = temp_file.path();

    // Act: Parse certificate information
    let result = CheckPointParser::parse_certificate_info(file_path);

    // Assert: Should extract certificate details
    match result {
        Ok(cert_info) => {
            assert_eq!(cert_info.total_certificates, 3);
            assert_eq!(cert_info.valid_certificates, 2);
            assert_eq!(cert_info.expired_certificates, 1);
            assert_eq!(cert_info.certificates.len(), 3);

            // Verify specific certificate
            let server_cert = cert_info
                .certificates
                .iter()
                .find(|c| c.name == "Server_Certificate")
                .unwrap();
            assert_eq!(server_cert.issuer, "Check Point Internal CA");
            assert_eq!(server_cert.status, "Valid");
        }
        Err(e) => panic!("Certificate information should be parsed, but got error: {e}"),
    }
}

/// Test 24: Performance metrics and statistics extraction
///
/// Should extract CPU, memory, network performance data
/// This test should FAIL initially because performance parsing is not implemented
#[test]
fn test_should_extract_performance_metrics() {
    // Arrange: Create cpinfo file with performance data
    let temp_file = create_performance_cpinfo_file();
    let file_path = temp_file.path();

    // Act: Parse performance metrics
    let result = CheckPointParser::parse_performance_metrics(file_path);

    // Assert: Should extract performance data
    match result {
        Ok(perf_metrics) => {
            assert!(
                perf_metrics.cpu_usage_percent >= 0.0 && perf_metrics.cpu_usage_percent <= 100.0
            );
            assert!(
                perf_metrics.memory_usage_percent >= 0.0
                    && perf_metrics.memory_usage_percent <= 100.0
            );
            assert!(
                perf_metrics.disk_usage_percent >= 0.0 && perf_metrics.disk_usage_percent <= 100.0
            );
            assert!(perf_metrics.connections_per_second > 0);
            assert!(perf_metrics.throughput_mbps > 0.0);
        }
        Err(e) => panic!("Performance metrics should be extracted, but got error: {e}"),
    }
}

// Additional test helper functions for Phase 2 tests

fn create_network_cpinfo_file() -> NamedTempFile {
    let mut temp_file =
        NamedTempFile::with_suffix(".info").expect("Failed to create temporary file");

    write!(
        temp_file,
        "************************************************************************
                       Check Point Support Information                    

************************************************************************

==============================================
Network Interfaces
==============================================
Total Interfaces: 4

Interface: eth0
  IP Address: 192.168.1.10
  Subnet Mask: 255.255.255.0
  State: Up
  MTU: 1500

Interface: eth1
  IP Address: 10.10.1.1
  Subnet Mask: 255.255.255.0
  State: Up
  MTU: 1500

Interface: eth2
  IP Address: 172.16.1.1
  Subnet Mask: 255.255.255.0
  State: Down
  MTU: 1500

Interface: mgmt
  IP Address: 10.0.0.5
  Subnet Mask: 255.255.255.0
  State: Up
  MTU: 1500
"
    )
    .expect("Failed to write test content");

    temp_file
}

fn create_vpn_cpinfo_file() -> NamedTempFile {
    let mut temp_file =
        NamedTempFile::with_suffix(".info").expect("Failed to create temporary file");

    write!(
        temp_file,
        "************************************************************************
                       Check Point Support Information                    

************************************************************************

==============================================
VPN Configuration
==============================================
Total Tunnels: 3
Active Tunnels: 2
Remote Access: Enabled

Tunnel: Site_A_Tunnel
  Remote Peer: 203.0.113.5
  Status: Up
  Encryption: AES-256
  Authentication: SHA-256

Tunnel: Site_B_Tunnel
  Remote Peer: 198.51.100.10
  Status: Up
  Encryption: AES-128
  Authentication: SHA-1

Tunnel: Site_C_Tunnel
  Remote Peer: 192.0.2.15
  Status: Down
  Encryption: AES-256
  Authentication: SHA-256
"
    )
    .expect("Failed to write test content");

    temp_file
}

fn create_ha_cpinfo_file() -> NamedTempFile {
    let mut temp_file =
        NamedTempFile::with_suffix(".info").expect("Failed to create temporary file");

    write!(
        temp_file,
        "************************************************************************
                       Check Point Support Information                    

************************************************************************

==============================================
High Availability Status
==============================================
HA Enabled: true
Local State: Active
Peer State: Standby
Sync Status: In Sync
Failover Mode: New High Availability
Last Failover: Never
Sync Interface: eth0-01
"
    )
    .expect("Failed to write test content");

    temp_file
}

fn create_log_cpinfo_file() -> NamedTempFile {
    let mut temp_file =
        NamedTempFile::with_suffix(".info").expect("Failed to create temporary file");

    write!(
        temp_file,
        "************************************************************************
                       Check Point Support Information                    

************************************************************************

==============================================
Log Files Information
==============================================
Total Log Types: 4
Total Size: 2048 MB
Oldest Entry: 2024-01-15 09:30:15

Log Type: Firewall
  Location: /var/log/fw.log
  Size: 1024 MB
  Entries: 50000

Log Type: VPN
  Location: /var/log/vpn.log
  Size: 512 MB
  Entries: 25000

Log Type: System
  Location: /var/log/messages
  Size: 256 MB
  Entries: 15000

Log Type: Audit
  Location: /var/log/audit.log
  Size: 256 MB
  Entries: 10000
"
    )
    .expect("Failed to write test content");

    temp_file
}

fn create_certificate_cpinfo_file() -> NamedTempFile {
    let mut temp_file =
        NamedTempFile::with_suffix(".info").expect("Failed to create temporary file");

    write!(
        temp_file,
        "************************************************************************
                       Check Point Support Information                    

************************************************************************

==============================================
Certificate Information
==============================================
Total Certificates: 3
Valid Certificates: 2
Expired Certificates: 1

Certificate: Server_Certificate
  Issuer: Check Point Internal CA
  Subject: CN=gateway.company.com
  Status: Valid
  Expires: 2025-12-31

Certificate: Client_Certificate
  Issuer: VeriSign
  Subject: CN=client.company.com
  Status: Valid
  Expires: 2025-06-15

Certificate: Old_Certificate
  Issuer: Check Point Internal CA
  Subject: CN=old.company.com
  Status: Expired
  Expires: 2023-01-01
"
    )
    .expect("Failed to write test content");

    temp_file
}

fn create_performance_cpinfo_file() -> NamedTempFile {
    let mut temp_file =
        NamedTempFile::with_suffix(".info").expect("Failed to create temporary file");

    write!(
        temp_file,
        "************************************************************************
                       Check Point Support Information                    

************************************************************************

==============================================
Performance Metrics
==============================================
CPU Usage: 45.2%
Memory Usage: 67.8%
Disk Usage: 23.1%
Connections per Second: 1250
Throughput: 890.5 Mbps
Packet Loss: 0.01%
Average Latency: 2.3ms

==============================================
Resource Statistics
==============================================
Total Memory: 16384 MB
Used Memory: 11116 MB
Free Memory: 5268 MB
Total Disk: 500 GB
Used Disk: 115 GB
Free Disk: 385 GB
"
    )
    .expect("Failed to write test content");

    temp_file
}
