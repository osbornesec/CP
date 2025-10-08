//! Error handling and recovery tests for `CPInfo` parser
//!
//! Phase 5: Error Handling and Recovery Tests (Tests 46-54)
//! Following Canon TDD principles for robust error handling implementation

use cpinfo_parser::{CpinfoError, CpinfoParser};
use std::io::Write;
use tempfile::NamedTempFile;
mod common;
use common::{create_temp_output_dir, create_test_cpinfo_file};

/// Test 46: Should handle corrupted cpinfo file headers gracefully
/// Purpose: Robustness against corruption
#[test]
fn test_corrupted_cpinfo_file_headers_graceful_handling() {
    // Arrange: Create a file with malformed header
    let mut corrupted_file = NamedTempFile::with_suffix(".info").unwrap();
    writeln!(corrupted_file, "CORRUPTED HEADER - NOT VALID CPINFO").unwrap();
    writeln!(corrupted_file, "Random garbage data").unwrap();
    writeln!(
        corrupted_file,
        "=============================================="
    )
    .unwrap();
    writeln!(corrupted_file, "Some content section").unwrap();
    writeln!(
        corrupted_file,
        "=============================================="
    )
    .unwrap();
    corrupted_file.flush().unwrap();

    let parser = CpinfoParser::new();

    // Act: Attempt to parse the corrupted file
    let result = parser.detect_format(corrupted_file.path());

    // Assert: Should get clear error message without crash
    match result {
        Err(CpinfoError::InvalidFormat { reason }) => {
            assert!(
                reason.contains("corrupted")
                    || reason.contains("invalid")
                    || reason.contains("malformed"),
                "Error message should indicate corruption: {reason}"
            );
        }
        Err(other_error) => panic!("Expected InvalidFormat error, but got: {other_error}"),
        Ok(_) => panic!("Corrupted file should be rejected, but parsing succeeded"),
    }
}

/// Test 47: Should recover from incomplete section delimiters\
/// Purpose: Partial file processing
#[test]
fn test_incomplete_section_delimiters_recovery() {
    // Arrange: Create file with missing end delimiter
    let mut incomplete_file = NamedTempFile::with_suffix(".info").unwrap();
    writeln!(incomplete_file, "Check Point Support Information").unwrap();
    writeln!(
        incomplete_file,
        "=============================================="
    )
    .unwrap();
    writeln!(incomplete_file, "General Information").unwrap();
    writeln!(
        incomplete_file,
        "=============================================="
    )
    .unwrap();
    writeln!(incomplete_file, "Version: R81.10").unwrap();
    writeln!(incomplete_file, "Build: 029").unwrap();
    writeln!(incomplete_file, "Network Configuration").unwrap();
    writeln!(
        incomplete_file,
        "=============================================="
    )
    .unwrap();
    writeln!(incomplete_file, "Interface: eth0").unwrap();
    writeln!(incomplete_file, "IP: 192.168.1.1").unwrap();
    // Missing final delimiter - file ends abruptly
    incomplete_file.flush().unwrap();

    let parser = CpinfoParser::new();
    let output_dir = create_temp_output_dir();

    // Act: Attempt to extract sections from incomplete file
    let recovery_config = cpinfo_parser::PartialRecoveryConfig {
        enable_section_checkpointing: true,
        max_section_errors: 3,
        recovery_strategy: cpinfo_parser::RecoveryStrategy::ContinueOnError,
        checkpoint_interval_sections: 5,
        preserve_partial_sections: true,
    };

    let result = parser.extract_sections_with_partial_recovery(
        incomplete_file.path(),
        output_dir.path(),
        recovery_config,
    );

    // Assert: Should process partial content and handle recovery
    match result {
        Ok(recovery_result) => {
            assert!(
                recovery_result.valid_sections_processed >= 1,
                "Should process at least some valid sections"
            );
            assert!(
                recovery_result.recovery_actions_taken >= 0,
                "Should track recovery actions taken"
            );
        }
        Err(CpinfoError::PartialProcessing { message }) => {
            assert!(
                message.contains("incomplete") || message.contains("delimiter"),
                "Should explain partial processing issue: {message}"
            );
        }
        Err(e) => panic!("Expected partial processing or success, but got: {e}"),
    }
}

/// Test 48: Should handle unexpected binary content in text sections
/// Purpose: Mixed content handling  
#[test]
fn test_binary_content_in_text_sections_handling() {
    // Arrange: Create file with binary data in text section
    let mut mixed_content_file = NamedTempFile::with_suffix(".info").unwrap();
    writeln!(mixed_content_file, "Check Point Support Information").unwrap();
    writeln!(
        mixed_content_file,
        "=============================================="
    )
    .unwrap();
    writeln!(mixed_content_file, "System Information").unwrap();
    writeln!(
        mixed_content_file,
        "=============================================="
    )
    .unwrap();
    writeln!(mixed_content_file, "Normal text content").unwrap();
    // Insert binary data
    mixed_content_file
        .write_all(&[0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD])
        .unwrap();
    writeln!(mixed_content_file, "\nMore normal text").unwrap();
    writeln!(
        mixed_content_file,
        "=============================================="
    )
    .unwrap();
    mixed_content_file.flush().unwrap();

    let parser = CpinfoParser::new();
    let output_dir = create_temp_output_dir();

    // Act: Attempt to process file with binary content
    let result =
        parser.extract_sections_with_binary_detection(mixed_content_file.path(), output_dir.path());

    // Assert: Should detect binary content and handle appropriately
    match result {
        Ok(extraction_result) => {
            assert!(
                extraction_result.binary_sections_detected > 0,
                "Should detect binary content in sections"
            );
            assert!(
                extraction_result.sections_extracted >= 1,
                "Should still extract valid text sections"
            );
            assert!(
                extraction_result
                    .warnings
                    .iter()
                    .any(|w| w.contains("binary")),
                "Should warn about binary content"
            );
        }
        Err(e) => panic!("Should handle binary content gracefully, but got error: {e}"),
    }
}

/// Test 49: Should process files with mixed line endings
/// Purpose: Cross-platform compatibility
#[test]
fn test_mixed_line_endings_processing() {
    // Arrange: Create file with different line ending types
    let mut mixed_endings_file = NamedTempFile::with_suffix(".info").unwrap();

    // Write content with mixed line endings
    mixed_endings_file
        .write_all(b"Check Point Support Information\r\n")
        .unwrap(); // CRLF
    mixed_endings_file
        .write_all(b"==============================================\n")
        .unwrap(); // LF
    mixed_endings_file.write_all(b"Unix Section\r").unwrap(); // CR
    mixed_endings_file
        .write_all(b"==============================================\r\n")
        .unwrap(); // CRLF
    mixed_endings_file.write_all(b"Content line 1\n").unwrap(); // LF
    mixed_endings_file.write_all(b"Content line 2\r\n").unwrap(); // CRLF
    mixed_endings_file.write_all(b"Content line 3\r").unwrap(); // CR
    mixed_endings_file
        .write_all(b"==============================================\n")
        .unwrap(); // LF
    mixed_endings_file.flush().unwrap();

    let parser = CpinfoParser::new();
    let output_dir = create_temp_output_dir();

    // Act: Process file with mixed line endings
    let result = parser.extract_sections(mixed_endings_file.path(), output_dir.path());

    // Assert: Should correctly parse sections regardless of line endings
    match result {
        Ok(extraction_result) => {
            assert_eq!(
                extraction_result.sections_extracted, 1,
                "Should extract the Unix Section correctly"
            );

            // Verify extracted content
            let section_file = output_dir.path().join("Unix_Section.txt");
            assert!(section_file.exists(), "Section file should be created");

            let content = std::fs::read_to_string(section_file).unwrap();
            assert!(
                content.contains("Content line 1"),
                "Should preserve content from LF line"
            );
            assert!(
                content.contains("Content line 2"),
                "Should preserve content from CRLF line"
            );
            assert!(
                content.contains("Content line 3"),
                "Should preserve content from CR line"
            );
        }
        Err(e) => panic!("Should handle mixed line endings gracefully, but got error: {e}"),
    }
}

/// Test 50: Should handle disk full errors during extraction
/// Purpose: Disk space handling
#[test]
fn test_disk_full_errors_during_extraction() {
    // Arrange: Create a large cpinfo file
    let large_content =
        "Large section\n==============================================\n".repeat(100);
    let large_file = create_test_cpinfo_file(&large_content);
    let parser = CpinfoParser::new();
    let output_dir = create_temp_output_dir();

    // Act: Attempt extraction with disk space constraints
    let disk_config = cpinfo_parser::PerformanceConfig {
        max_disk_space_mb: 1, // Very limited disk space
        enable_disk_monitoring: true,
        enable_cleanup_on_pressure: true,
        ..Default::default()
    };

    let result =
        parser.parse_with_disk_constraints(large_file.path(), output_dir.path(), disk_config);

    // Assert: Should gracefully fail with cleanup
    match result {
        Err(CpinfoError::Io(io_error)) => {
            assert_eq!(
                io_error.kind(),
                std::io::ErrorKind::WriteZero,
                "Should detect disk full condition"
            );

            // Verify cleanup - no partial files should remain
            let entries: Vec<_> = std::fs::read_dir(output_dir.path()).unwrap().collect();
            assert!(
                entries.is_empty()
                    || entries
                        .iter()
                        .all(|e| { e.as_ref().unwrap().metadata().unwrap().len() == 0 }),
                "Partial files should be cleaned up"
            );
        }
        Err(CpinfoError::ValidationError { message }) if message.contains("disk") => {
            // Alternative: custom disk space error
        }
        Ok(_) => panic!("Should fail when disk is full"),
        Err(e) => panic!("Expected disk full error, but got: {e}"),
    }
}

/// Test 51: Should recover from network interruptions during file access
/// Purpose: Network reliability
#[test]
fn test_network_interruptions_recovery() {
    // Arrange: Simulate network-mounted file with interruption
    let network_content = "Network test\n==============================================\nContent to test network resilience";
    let network_file = create_test_cpinfo_file(network_content);
    let parser = CpinfoParser::new();

    // Act: Parse with network timeout and retry configuration
    let network_config = cpinfo_parser::NetworkConfig {
        connection_timeout: core::time::Duration::from_secs(5),
        read_timeout: core::time::Duration::from_secs(3),
        max_retries: 3,
        enable_distributed_mode: false,
        node_health_check_interval: core::time::Duration::from_millis(100),
    };

    let result = parser.parse_with_network_timeout(network_file.path(), network_config);

    // Assert: Should handle network conditions gracefully
    match result {
        Ok(network_result) => {
            assert!(
                network_result.successful_connections > 0,
                "Should establish successful connection"
            );
            assert!(
                network_result.connection_attempts > 0,
                "Should record connection attempts"
            );
        }
        Err(CpinfoError::NetworkError { attempts, message }) => {
            assert!(attempts > 0, "Should attempt at least one connection");
            assert!(
                message.contains("timeout") || message.contains("connection"),
                "Should provide clear network error message: {message}"
            );
        }
        Err(CpinfoError::Io(io_error)) => {
            // Network failure after retries is acceptable
            assert!(
                io_error.kind() == std::io::ErrorKind::TimedOut
                    || io_error.kind() == std::io::ErrorKind::Interrupted,
                "Should properly identify network issue: {:?}",
                io_error.kind()
            );
        }
        Err(e) => panic!("Unexpected error type for network test: {e}"),
    }
}

/// Test 52: Should handle permission denied errors on output directory\
/// Purpose: Permission handling
#[test]
fn test_permission_denied_output_directory() {
    // Arrange: Create input file and restricted output directory
    let permission_content = "Permission test\n==============================================\nTest content for permission validation";
    let input_file = create_test_cpinfo_file(permission_content);
    let output_dir = create_temp_output_dir();

    // Simulate permission restriction (Unix-specific)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(output_dir.path()).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        std::fs::set_permissions(output_dir.path(), perms).unwrap();
    }

    let parser = CpinfoParser::new();

    // Act: Attempt extraction to restricted directory
    let result = parser.extract_sections(input_file.path(), output_dir.path());

    // Assert: Should provide clear permission error
    match result {
        Err(CpinfoError::Io(io_error)) => {
            assert_eq!(
                io_error.kind(),
                std::io::ErrorKind::PermissionDenied,
                "Should identify permission denied error"
            );
        }
        Err(CpinfoError::SecurityViolation { reason }) => {
            assert!(
                reason.contains("permission"),
                "Security violation should mention permissions"
            );
        }
        Ok(_) => {
            #[cfg(windows)]
            {
                // Windows may not enforce directory permissions the same way
                println!("Note: Windows may not enforce directory permissions");
            }
            #[cfg(unix)]
            panic!("Should fail with permission denied on Unix systems");
        }
        Err(e) => panic!("Expected permission error, but got: {e}"),
    }
}

/// Test 52: Should validate and sanitize user input for all entry points
/// Purpose: Input security and validation
#[test]
fn test_user_input_validation_and_sanitization() {
    let parser = CpinfoParser::new();

    // Test 1: File path injection attempts
    let malicious_paths = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config\\sam",
        "/dev/null",
        "NUL:",
        "file://localhost/etc/passwd",
        "\\\\server\\share\\file",
    ];

    for malicious_path in malicious_paths {
        let result = parser.validate_input_path(malicious_path);
        match result {
            Err(CpinfoError::SecurityViolation { reason }) => {
                assert!(
                    reason.contains("path")
                        || reason.contains("directory")
                        || reason.contains("invalid"),
                    "Should detect path injection attempt: {reason}"
                );
            }
            Err(CpinfoError::ValidationError { message }) => {
                assert!(
                    message.contains("path") || message.contains("invalid"),
                    "Should validate path format: {message}"
                );
            }
            _ => panic!("Should reject malicious path: {malicious_path}"),
        }
    }

    // Test 2: Command argument sanitization
    let malicious_args = vec![
        "--help; rm -rf /",
        "; cat /etc/passwd",
        "$(whoami)",
        "`id`",
        "${PATH}",
        "& net user",
    ];

    for malicious_arg in malicious_args {
        let result = parser.validate_command_argument(malicious_arg);
        match result {
            Err(CpinfoError::SecurityViolation { reason }) => {
                assert!(
                    reason.contains("command")
                        || reason.contains("injection")
                        || reason.contains("invalid"),
                    "Should detect command injection: {reason}"
                );
            }
            _ => panic!("Should reject malicious argument: {malicious_arg}"),
        }
    }

    // Test 3: Configuration parameter validation
    let test_file = create_test_cpinfo_file(
        "General Information\n==============================================\nTest content",
    );
    let malicious_configs = vec![
        ("max_memory_mb", "-1"),        // Negative values
        ("buffer_size", "0"),           // Zero values
        ("output_dir", "../../../tmp"), // Path traversal
        ("thread_count", "999999"),     // Excessive values
    ];

    for (param, value) in malicious_configs {
        let result = parser.validate_config_parameter(param, value);
        match result {
            Err(CpinfoError::ValidationError { message }) => {
                assert!(
                    message.contains(param)
                        || message.contains("invalid")
                        || message.contains("range"),
                    "Should validate config parameter {param}: {message}"
                );
            }
            _ => panic!("Should reject invalid config parameter: {param} = {value}"),
        }
    }
}

/// Test 53: Should handle system resource exhaustion with graceful degradation
/// Purpose: System stability under resource pressure  
#[test]
fn test_system_resource_exhaustion_graceful_degradation() {
    let parser = CpinfoParser::new();
    let test_file = create_test_cpinfo_file(
        "Large content section\n==============================================\nLarge data here",
    );

    // Test 1: Memory exhaustion handling
    let memory_config = cpinfo_parser::PerformanceConfig {
        max_memory_mb: 1, // Very restrictive limit
        enable_memory_monitoring: true,
        buffer_size: 512, // Small buffer
        enable_graceful_degradation: true,
        ..Default::default()
    };

    let result = parser.parse_with_resource_constraints(test_file.path(), memory_config);
    match result {
        Ok(degraded_result) => {
            assert!(
                degraded_result.degradation_applied,
                "Should apply graceful degradation under memory pressure"
            );
            assert!(
                degraded_result.peak_memory_mb <= 2.0,
                "Should respect memory constraints with degradation"
            );
            assert!(
                degraded_result.processing_successful,
                "Should complete processing despite constraints"
            );
        }
        Err(CpinfoError::ResourceExhaustion {
            resource_type,
            message,
        }) => {
            assert_eq!(resource_type, "memory");
            assert!(
                message.contains("exhausted") || message.contains("exceeded"),
                "Should provide clear resource exhaustion message: {message}"
            );
        }
        Err(e) => panic!("Unexpected error type for memory exhaustion: {e}"),
    }

    // Test 2: Disk space limitations with cleanup
    let disk_config = cpinfo_parser::PerformanceConfig {
        max_disk_space_mb: 5, // Limited disk space
        enable_disk_monitoring: true,
        enable_cleanup_on_pressure: true,
        cleanup_threshold_percent: 80.0,
        ..Default::default()
    };

    let output_dir = create_temp_output_dir();
    let result =
        parser.parse_with_disk_constraints(test_file.path(), output_dir.path(), disk_config);
    match result {
        Ok(disk_result) => {
            assert!(
                disk_result.cleanup_triggered || disk_result.space_management_applied,
                "Should manage disk space under pressure"
            );
            assert!(
                disk_result.final_disk_usage_mb <= 8.0,
                "Should maintain disk usage within reasonable limits"
            );
        }
        Err(CpinfoError::ResourceExhaustion {
            resource_type,
            message,
        }) => {
            assert_eq!(resource_type, "disk");
            assert!(
                message.contains("space") || message.contains("full"),
                "Should provide clear disk space message: {message}"
            );
        }
        Err(e) => panic!("Unexpected error type for disk exhaustion: {e}"),
    }

    // Test 3: CPU throttling under high load
    let cpu_config = cpinfo_parser::PerformanceConfig {
        enable_cpu_monitoring: true,
        cpu_throttle_threshold: 60.0, // 60% CPU threshold
        enable_adaptive_processing: true,
        max_processing_threads: 2,
        ..Default::default()
    };

    let result = parser.parse_with_cpu_throttling(test_file.path(), cpu_config);
    match result {
        Ok(cpu_result) => {
            assert!(
                cpu_result.throttling_applied || cpu_result.adaptive_processing_used,
                "Should apply CPU throttling or adaptive processing"
            );
            assert!(
                cpu_result.average_cpu_percent <= 80.0,
                "Should maintain reasonable CPU usage with throttling"
            );
        }
        Err(e) => panic!("Should handle CPU pressure gracefully, but got: {e}"),
    }
}

/// Test 54: Should provide comprehensive error logging and diagnostic information
/// Purpose: Operational excellence and troubleshooting support
#[test]
fn test_error_logging_and_diagnostic_information_collection() {
    let parser = CpinfoParser::new();

    // Test 1: Structured error logging for different error types

    // Test FileNotFound error
    let result = parser.parse_with_diagnostic_logging("nonexistent_file.info");
    match result {
        Err(_error) => {
            let diagnostic_info = parser.get_last_diagnostic_info();

            // Verify structured logging for FileNotFound
            assert!(
                diagnostic_info.error_type.contains("FileNotFound"),
                "Should log correct error type: expected FileNotFound, got {}",
                diagnostic_info.error_type
            );
            assert!(
                !diagnostic_info.timestamp.is_empty(),
                "Should include timestamp in diagnostic info"
            );
            assert!(
                !diagnostic_info.context.is_empty(),
                "Should include contextual information"
            );
            assert!(
                diagnostic_info.severity_level >= 1,
                "Should assign appropriate severity level"
            );

            // Verify error correlation
            assert!(
                !diagnostic_info.correlation_id.is_empty(),
                "Should provide correlation ID for error tracking"
            );
        }
        Ok(()) => panic!("Expected error for nonexistent file"),
    }

    // Test InvalidExtension error - create a file with wrong extension
    let wrong_extension_file = NamedTempFile::with_suffix(".txt").unwrap();
    let result = parser.parse_with_diagnostic_logging(wrong_extension_file.path());
    match result {
        Err(_error) => {
            let diagnostic_info = parser.get_last_diagnostic_info();

            // Verify structured logging for InvalidExtension
            assert!(
                diagnostic_info.error_type.contains("InvalidExtension"),
                "Should log correct error type: expected InvalidExtension, got {}",
                diagnostic_info.error_type
            );
            assert!(
                !diagnostic_info.timestamp.is_empty(),
                "Should include timestamp in diagnostic info"
            );
            assert!(
                !diagnostic_info.context.is_empty(),
                "Should include contextual information"
            );
            assert!(
                diagnostic_info.severity_level >= 1,
                "Should assign appropriate severity level"
            );

            // Verify error correlation
            assert!(
                !diagnostic_info.correlation_id.is_empty(),
                "Should provide correlation ID for error tracking"
            );
        }
        Ok(()) => panic!("Expected error for wrong extension file"),
    }

    // Test 2: Performance diagnostic collection
    let valid_file = create_test_cpinfo_file("Performance test\n==============================================\nContent for performance analysis");
    let result = parser.parse_with_performance_diagnostics(valid_file.path());

    match result {
        Ok(perf_result) => {
            let diagnostics = perf_result.diagnostic_info;

            // Verify performance metrics collection
            assert!(
                diagnostics.processing_time_ms > 0,
                "Should record processing time"
            );
            assert!(
                diagnostics.memory_usage_mb >= 0.0,
                "Should record memory usage"
            );
            assert!(
                !diagnostics.processing_stages.is_empty(),
                "Should track processing stages"
            );
            assert!(
                diagnostics.throughput_mbps >= 0.0,
                "Should calculate throughput metrics"
            );

            // Verify operational metrics
            assert!(
                diagnostics.system_info.cpu_cores > 0,
                "Should collect system information"
            );
            assert!(
                diagnostics.system_info.available_memory_mb > 0.0,
                "Should collect memory information"
            );
        }
        Err(e) => panic!("Should collect diagnostics for successful parsing: {e}"),
    }

    // Test 3: Enterprise monitoring integration
    let monitoring_config = cpinfo_parser::MonitoringConfig {
        enable_metrics_export: true,
        enable_health_checks: true,
        enable_alerting: true,
        log_level: "INFO".to_owned(),
        export_format: "json".to_owned(),
    };

    let test_file = create_test_cpinfo_file("Monitoring test\n==============================================\nTest content for monitoring");
    let result = parser.parse_with_monitoring(test_file.path(), monitoring_config);

    match result {
        Ok(monitoring_result) => {
            // Verify health check status
            assert!(
                monitoring_result.health_status.is_healthy,
                "Should report healthy status for successful processing"
            );
            assert!(
                !monitoring_result
                    .health_status
                    .component_statuses
                    .is_empty(),
                "Should include component health statuses"
            );

            // Verify metrics export
            assert!(
                !monitoring_result.exported_metrics.is_empty(),
                "Should export operational metrics"
            );
            assert!(
                monitoring_result
                    .exported_metrics
                    .contains_key("processing_time"),
                "Should export processing time metric"
            );
            assert!(
                monitoring_result
                    .exported_metrics
                    .contains_key("memory_usage"),
                "Should export memory usage metric"
            );

            // Verify alerting readiness
            assert!(
                monitoring_result.alert_thresholds_configured,
                "Should configure alert thresholds"
            );
        }
        Err(e) => panic!("Should support enterprise monitoring integration: {e}"),
    }
}
