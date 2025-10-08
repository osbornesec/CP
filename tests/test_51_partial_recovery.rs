//! Test 51: Partial file processing with section-level recovery and checkpointing
//! Following Canon TDD principles for resilient processing

use cpinfo_parser::CpinfoParser;

mod common;
use common::{create_temp_output_dir, create_test_cpinfo_file};

/// Test 51: RED PHASE - Partial file processing with section-level recovery
/// Purpose: Resilient processing that can recover from section-level failures
#[test]
fn test_partial_file_processing_section_recovery() {
    // Arrange: Create file with mixed valid and invalid sections
    let test_content = "Check Point Support Information
==============================================
Valid Section 1
==============================================
Version: R81.10
Build: 029
Status: Active
==============================================
Corrupted Section
==============================================
This section has
INVALID BINARY DATA: 
==============================================
Valid Section 2  
==============================================
Hostname: checkpoint-gw
IP: 192.168.1.100
==============================================
Incomplete Section
==============================================
This section is missing its end delimiter
and continues indefinitely without proper closure
causing parsing issues but should be recoverable
through section-level recovery mechanisms...
";

    let test_file = create_test_cpinfo_file(test_content);
    let output_dir = create_temp_output_dir();
    let parser = CpinfoParser::new();

    // Configure partial recovery settings
    let recovery_config = cpinfo_parser::PartialRecoveryConfig {
        enable_section_checkpointing: true,
        max_section_errors: 2,
        recovery_strategy: cpinfo_parser::RecoveryStrategy::ContinueOnError,
        checkpoint_interval_sections: 1,
        preserve_partial_sections: true,
    };

    // Act: Process file with partial recovery enabled
    let result = parser.extract_sections_with_partial_recovery(
        test_file.path(),
        output_dir.path(),
        recovery_config,
    );

    // Assert: Should recover valid sections despite errors
    match result {
        Ok(recovery_result) => {
            assert!(
                recovery_result.valid_sections_processed >= 2,
                "Should process at least 2 valid sections, got: {}",
                recovery_result.valid_sections_processed
            );
            assert!(
                recovery_result.failed_sections >= 1,
                "Should detect at least 1 failed section, got: {}",
                recovery_result.failed_sections
            );
            assert!(
                recovery_result.checkpoints_created > 0,
                "Should create checkpoints during processing"
            );
            assert!(
                recovery_result.recovery_actions_taken > 0,
                "Should take recovery actions for failed sections"
            );

            // Verify valid sections were extracted
            let section1_file = output_dir.path().join("Valid_Section_1.txt");
            let section2_file = output_dir.path().join("Valid_Section_2.txt");
            assert!(
                section1_file.exists(),
                "Valid Section 1 should be extracted"
            );
            assert!(
                section2_file.exists(),
                "Valid Section 2 should be extracted"
            );

            println!(
                "\u{2705} Partial recovery: {} valid sections, {} failures, {} checkpoints",
                recovery_result.valid_sections_processed,
                recovery_result.failed_sections,
                recovery_result.checkpoints_created
            );
        }
        Err(e) => panic!("Expected partial recovery to succeed, but got: {e}"),
    }
}

/// Test 51B: Checkpointing and resume functionality
#[test]
fn test_checkpointing_and_resume_functionality() {
    // Arrange: Large file to test checkpointing
    let mut large_content = String::from(
        "Check Point Support Information\n==============================================\n",
    );

    // Add multiple sections to trigger checkpointing
    for i in 1..=5 {
        large_content.push_str(&format!("Section {i}\n"));
        large_content.push_str("==============================================\n");
        large_content.push_str(&format!(
            "Content for section {i} with detailed information\n"
        ));
        large_content.push_str("Multiple lines of data\n");
        large_content.push_str("Important configuration details\n");
        large_content.push_str("==============================================\n");
    }

    let test_file = create_test_cpinfo_file(&large_content);
    let output_dir = create_temp_output_dir();
    let parser = CpinfoParser::new();

    let recovery_config = cpinfo_parser::PartialRecoveryConfig {
        enable_section_checkpointing: true,
        max_section_errors: 0, // No errors expected in this test
        recovery_strategy: cpinfo_parser::RecoveryStrategy::CreateCheckpoint,
        checkpoint_interval_sections: 2, // Checkpoint every 2 sections
        preserve_partial_sections: false,
    };

    // Act: Process with checkpointing
    let result = parser.extract_sections_with_checkpointing(
        test_file.path(),
        output_dir.path(),
        recovery_config,
    );

    // Assert: Should create appropriate checkpoints
    match result {
        Ok(checkpoint_result) => {
            assert_eq!(
                checkpoint_result.total_sections_processed, 5,
                "Should process all 5 sections"
            );
            assert!(
                checkpoint_result.checkpoints_created >= 2,
                "Should create at least 2 checkpoints (every 2 sections), got: {}",
                checkpoint_result.checkpoints_created
            );
            assert!(
                checkpoint_result.checkpoint_files.len() >= 2,
                "Should have checkpoint files created"
            );

            // Verify checkpoint files exist
            for checkpoint_file in &checkpoint_result.checkpoint_files {
                assert!(
                    checkpoint_file.exists(),
                    "Checkpoint file should exist: {checkpoint_file:?}"
                );
            }

            println!(
                "\u{2705} Checkpointing: {} sections, {} checkpoints created",
                checkpoint_result.total_sections_processed, checkpoint_result.checkpoints_created
            );
        }
        Err(e) => panic!("Expected checkpointing to succeed, but got: {e}"),
    }
}

/// Test 51C: Recovery from specific checkpoint
#[test]
fn test_recovery_from_specific_checkpoint() {
    // Arrange: Create file and simulate interruption during processing
    let test_content = "Check Point Support Information
==============================================
Section A
==============================================
Data A - processed successfully
==============================================
Section B
==============================================
Data B - processed successfully  
==============================================
Section C
==============================================
Data C - this is where failure occurs
SIMULATED_PROCESSING_INTERRUPTION
==============================================
Section D
==============================================
Data D - should be recovered from checkpoint
==============================================
";

    let test_file = create_test_cpinfo_file(test_content);
    let output_dir = create_temp_output_dir();
    let parser = CpinfoParser::new();

    let recovery_config = cpinfo_parser::PartialRecoveryConfig {
        enable_section_checkpointing: true,
        max_section_errors: 1,
        recovery_strategy: cpinfo_parser::RecoveryStrategy::ResumeFromCheckpoint,
        checkpoint_interval_sections: 1, // Checkpoint after each section
        preserve_partial_sections: true,
    };

    // Act: Process with checkpoint recovery
    let result = parser.extract_sections_with_checkpoint_recovery(
        test_file.path(),
        output_dir.path(),
        recovery_config,
    );

    // Assert: Should resume from last valid checkpoint
    match result {
        Ok(recovery_result) => {
            assert!(
                recovery_result.sections_recovered_from_checkpoint > 0,
                "Should recover at least one section from checkpoint"
            );
            assert!(
                recovery_result.last_successful_checkpoint.is_some(),
                "Should identify the last successful checkpoint"
            );
            assert!(
                recovery_result.total_sections_processed >= 3,
                "Should process sections before and after recovery point"
            );

            // Verify Section A and B were processed before failure
            let section_a_file = output_dir.path().join("Section_A.txt");
            let section_b_file = output_dir.path().join("Section_B.txt");
            assert!(
                section_a_file.exists(),
                "Section A should exist from initial processing"
            );
            assert!(
                section_b_file.exists(),
                "Section B should exist from initial processing"
            );

            println!(
                "\u{2705} Checkpoint recovery: {} sections recovered, last checkpoint: {:?}",
                recovery_result.sections_recovered_from_checkpoint,
                recovery_result.last_successful_checkpoint
            );
        }
        Err(e) => panic!("Expected checkpoint recovery to succeed, but got: {e}"),
    }
}
