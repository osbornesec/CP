//! Test 49: I/O error recovery and retry mechanisms with exponential backoff
//! Following Canon TDD principles for robust I/O handling

use cpinfo_parser::{CpinfoError, CpinfoParser};
use std::time::{Duration, Instant};

mod common;
use common::create_test_cpinfo_file;

/// Test 49: RED PHASE - I/O error recovery with exponential backoff
/// Purpose: Robust I/O handling with retry mechanisms
#[test]
fn test_io_error_recovery_exponential_backoff() {
    // Arrange: Create test file and parser with valid cpinfo content
    let test_content = r#"Check Point Support Information
==============================================
General Information
==============================================
Version: R81.10
Build: 029
==============================================
"#;
    let test_file = create_test_cpinfo_file(test_content);
    let parser = CpinfoParser::new();

    // Configure retry settings for testing
    let retry_config = cpinfo_parser::RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        backoff_multiplier: 2.0,
        retry_on_io_errors: true,
    };

    let start_time = Instant::now();

    // Act: Attempt to parse with simulated I/O interruptions
    let result = parser.parse_with_retry_config(test_file.path(), retry_config);

    let elapsed = start_time.elapsed();

    // Assert: Should handle I/O errors with proper retry logic
    match result {
        Ok(retry_result) => {
            // Success case - verify retry behavior
            assert!(
                retry_result.attempt_count >= 1,
                "Should track retry attempts"
            );
            assert!(
                elapsed >= Duration::from_millis(10),
                "Should have some delay from retries"
            );
            assert!(
                retry_result.total_delay <= Duration::from_millis(500),
                "Total delay should be reasonable"
            );
            println!(
                "✅ I/O retry succeeded after {} attempts",
                retry_result.attempt_count
            );
        }
        Err(CpinfoError::NetworkError { attempts, message }) => {
            // Acceptable failure after exhausting retries
            assert_eq!(attempts, 3, "Should exhaust all retry attempts");
            assert!(
                message.contains("I/O") || message.contains("retry"),
                "Error message should mention I/O or retry: {}",
                message
            );
            assert!(
                elapsed >= Duration::from_millis(30), // 10 + 20 + ...
                "Should have exponential backoff delays"
            );
            println!(
                "✅ I/O retry exhausted properly after {} attempts",
                attempts
            );
        }
        Err(e) => panic!("Expected I/O retry behavior, but got: {}", e),
    }
}

/// Test 49B: Verify exponential backoff timing
#[test]
fn test_exponential_backoff_timing() {
    // Arrange: Force I/O failure scenario
    let parser = CpinfoParser::new();
    let non_existent_file = std::path::Path::new("/nonexistent/path/file.info");

    let retry_config = cpinfo_parser::RetryConfig {
        max_attempts: 4,
        initial_delay: Duration::from_millis(50),
        max_delay: Duration::from_millis(500),
        backoff_multiplier: 2.0,
        retry_on_io_errors: true,
    };

    let start_time = Instant::now();

    // Act: Attempt operation that will fail
    let result = parser.parse_with_retry_config(non_existent_file, retry_config);

    let elapsed = start_time.elapsed();

    // Assert: Should demonstrate exponential backoff
    match result {
        Err(CpinfoError::NetworkError { attempts, .. }) => {
            assert_eq!(attempts, 4, "Should use all retry attempts");
            // Expected delays: 50ms + 100ms + 200ms + 400ms = 750ms minimum
            assert!(
                elapsed >= Duration::from_millis(700),
                "Should show exponential backoff delays, but took only {:?}",
                elapsed
            );
            assert!(
                elapsed <= Duration::from_millis(1200),
                "Should not exceed reasonable maximum, but took {:?}",
                elapsed
            );
            println!("✅ Exponential backoff timing verified: {:?}", elapsed);
        }
        other => panic!(
            "Expected network error with retry attempts, got: {:?}",
            other
        ),
    }
}

/// Test 49C: Transient vs persistent error handling
#[test]
fn test_transient_vs_persistent_errors() {
    // Arrange: Create scenario with transient errors
    let test_content = r#"Check Point Support Information
==============================================
System Information
==============================================
Hostname: test-checkpoint
IP: 192.168.1.100
==============================================
"#;
    let test_file = create_test_cpinfo_file(test_content);
    let parser = CpinfoParser::new();

    let retry_config = cpinfo_parser::RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(5),
        max_delay: Duration::from_millis(50),
        backoff_multiplier: 2.0,
        retry_on_io_errors: true,
    };

    // Act: Parse with transient error simulation
    let result = parser.parse_with_transient_simulation(test_file.path(), retry_config);

    // Assert: Should retry transient errors but not persistent ones
    match result {
        Ok(retry_result) => {
            assert!(
                retry_result.transient_errors_recovered > 0,
                "Should recover from some transient errors"
            );
            assert!(
                retry_result.attempt_count > 1,
                "Should make multiple attempts for transient errors"
            );
            println!(
                "✅ Transient error recovery: {} errors recovered",
                retry_result.transient_errors_recovered
            );
        }
        Err(CpinfoError::ValidationError { message }) if message.contains("persistent") => {
            // Acceptable - persistent errors should fail immediately
            println!("✅ Persistent error correctly identified: {}", message);
        }
        Err(e) => panic!("Expected transient error handling, but got: {}", e),
    }
}
