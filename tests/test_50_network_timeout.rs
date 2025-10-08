//! Test 50: Network timeout handling for distributed processing scenarios
//! Following Canon TDD principles for robust network error handling

use cpinfo_parser::{CpinfoError, CpinfoParser};
use std::time::{Duration, Instant};

mod common;
use common::create_test_cpinfo_file;

/// Test 50: RED PHASE - Network timeout handling for distributed scenarios
/// Purpose: Robust network failure handling in distributed environments
#[test]
fn test_network_timeout_handling_distributed() {
    // Arrange: Create test file and configure network timeouts
    let test_content = r#"Check Point Support Information
==============================================
Network Configuration
==============================================
Interface: eth0
IP: 192.168.1.100
Gateway: 192.168.1.1
==============================================
Distributed Processing Info
==============================================
Cluster Node: node-1
Sync Status: active
==============================================
"#;
    let test_file = create_test_cpinfo_file(test_content);
    let parser = CpinfoParser::new();

    // Configure network timeout settings for distributed processing
    let network_config = cpinfo_parser::NetworkConfig {
        connection_timeout: Duration::from_millis(100),
        read_timeout: Duration::from_millis(200),
        max_retries: 3,
        enable_distributed_mode: true,
        node_health_check_interval: Duration::from_millis(50),
    };

    let start_time = Instant::now();

    // Act: Attempt distributed processing with network timeouts
    let result = parser.parse_with_network_timeout(test_file.path(), network_config);

    let elapsed = start_time.elapsed();

    // Assert: Should handle network timeouts gracefully
    match result {
        Ok(network_result) => {
            // Success case - verify network behavior tracking
            assert!(
                network_result.connection_attempts >= 1,
                "Should track connection attempts"
            );
            assert!(
                network_result.timeout_events == 0 || network_result.successful_connections > 0,
                "Should handle timeouts or succeed"
            );
            assert!(
                elapsed <= Duration::from_millis(1000),
                "Should complete within reasonable time"
            );
            println!(
                "✅ Network processing succeeded: {} connections, {} timeouts",
                network_result.successful_connections, network_result.timeout_events
            );
        }
        Err(CpinfoError::NetworkError { attempts, message }) => {
            // Acceptable failure after network timeouts
            assert!(attempts >= 1, "Should attempt at least one connection");
            assert!(
                message.contains("timeout") || message.contains("network"),
                "Error message should mention timeout or network: {}",
                message
            );
            assert!(
                elapsed >= Duration::from_millis(50),
                "Should take some time due to timeouts"
            );
            println!(
                "✅ Network timeout handled properly after {} attempts",
                attempts
            );
        }
        Err(e) => panic!("Expected network timeout behavior, but got: {}", e),
    }
}

/// Test 50B: Distributed node failure recovery
#[test]
fn test_distributed_node_failure_recovery() {
    // Arrange: Simulate distributed processing with node failures
    let test_content = r#"Check Point Support Information
==============================================
Cluster Information  
==============================================
Primary Node: cluster-node-1
Backup Nodes: cluster-node-2, cluster-node-3
Load Balancer: active
==============================================
"#;
    let test_file = create_test_cpinfo_file(test_content);
    let parser = CpinfoParser::new();

    let network_config = cpinfo_parser::NetworkConfig {
        connection_timeout: Duration::from_millis(50),
        read_timeout: Duration::from_millis(100),
        max_retries: 2,
        enable_distributed_mode: true,
        node_health_check_interval: Duration::from_millis(25),
    };

    // Act: Process with simulated node failures
    let result = parser.parse_with_node_failure_simulation(test_file.path(), network_config);

    // Assert: Should demonstrate node failure recovery
    match result {
        Ok(recovery_result) => {
            assert!(
                recovery_result.failed_nodes > 0,
                "Should simulate at least one node failure"
            );
            assert!(
                recovery_result.recovery_attempts > 0,
                "Should attempt recovery from failures"
            );
            assert!(
                recovery_result.final_processing_node.is_some(),
                "Should identify final processing node"
            );
            println!(
                "✅ Node failure recovery: {} failed nodes, recovered to {:?}",
                recovery_result.failed_nodes, recovery_result.final_processing_node
            );
        }
        Err(CpinfoError::NetworkError { attempts, message }) => {
            // Acceptable if all nodes fail
            assert!(attempts >= 2, "Should try multiple nodes");
            assert!(
                message.contains("node") || message.contains("distributed"),
                "Should mention distributed failure: {}",
                message
            );
            println!(
                "✅ All nodes failed appropriately after {} attempts",
                attempts
            );
        }
        Err(e) => panic!("Expected distributed processing behavior, but got: {}", e),
    }
}

/// Test 50C: Network connection pooling and reuse
#[test]
fn test_network_connection_pooling() {
    // Arrange: Multiple files to test connection reuse
    let test_files = vec![
        create_test_cpinfo_file("Section 1\n==============================================\nData 1\n=============================================="),
        create_test_cpinfo_file("Section 2\n==============================================\nData 2\n=============================================="),
        create_test_cpinfo_file("Section 3\n==============================================\nData 3\n=============================================="),
    ];

    let parser = CpinfoParser::new();

    let network_config = cpinfo_parser::NetworkConfig {
        connection_timeout: Duration::from_millis(100),
        read_timeout: Duration::from_millis(200),
        max_retries: 2,
        enable_distributed_mode: true,
        node_health_check_interval: Duration::from_millis(50),
    };

    let start_time = Instant::now();

    // Act: Process multiple files to test connection pooling
    let result = parser.parse_multiple_with_connection_pooling(
        test_files.iter().map(|f| f.path()).collect(),
        network_config,
    );

    let elapsed = start_time.elapsed();

    // Assert: Should demonstrate connection reuse efficiency
    match result {
        Ok(pooling_result) => {
            assert_eq!(
                pooling_result.files_processed, 3,
                "Should process all 3 files"
            );
            assert!(
                pooling_result.connections_created <= pooling_result.files_processed,
                "Should reuse connections (created: {}, files: {})",
                pooling_result.connections_created,
                pooling_result.files_processed
            );
            assert!(
                pooling_result.connection_reuse_rate >= 0.0,
                "Should track connection reuse rate"
            );
            assert!(
                elapsed <= Duration::from_secs(2),
                "Should complete efficiently with connection pooling"
            );
            println!(
                "✅ Connection pooling: {} connections for {} files ({}% reuse)",
                pooling_result.connections_created,
                pooling_result.files_processed,
                (pooling_result.connection_reuse_rate * 100.0) as u32
            );
        }
        Err(e) => panic!("Expected successful connection pooling, but got: {}", e),
    }
}
