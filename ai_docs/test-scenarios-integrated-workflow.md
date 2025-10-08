# CPInfo Parser - Integrated Workflow Test Scenarios

## Executive Summary

This document provides comprehensive test scenarios for the **integrated workflow** where the cpinfo parser automatically processes section files after extracting them from the main cpinfo file, making this the **default behavior**. The test scenarios follow Canon Test-Driven Development (TDD) principles as defined by Kent Beck, building confidence incrementally through systematic testing of the complete end-to-end workflow.

## Test Strategy Overview

### Integrated Workflow Testing Approach
- **Framework**: Rust's `cargo test` with `tokio-test` for async operations and streaming I/O
- **Test Types**: Integration, End-to-End, Performance, Error Recovery, User Experience
- **TDD Cycle**: Red → Green → Refactor → Repeat following Kent Beck's Canon TDD
- **Confidence Building**: Start with simple integrated scenarios, add complexity incrementally
- **Discovery Process**: Expand test list as edge cases emerge during two-phase processing
- **Coverage Target**: >90% integration test coverage with 100% critical workflow coverage

### Latest Rust CLI Integration Testing Best Practices (2025)
Based on comprehensive research of modern Rust CLI testing approaches:

#### Integration Testing Framework
```toml
[dev-dependencies]
# CLI and integration testing
assert_cmd = "2.0"           # Command line application testing
predicates = "3.0"           # Flexible assertion predicates
tempfile = "3.8"             # Temporary directories for integration tests
assert_fs = "1.1"            # File system state assertions

# Async integration testing
tokio-test = "0.4"           # Async test utilities
tokio = { version = "1.40", features = ["test-util", "rt-multi-thread", "process"] }

# Process and system integration
wait-timeout = "0.2"         # Process timeout handling
nix = "0.27"                 # Unix system calls for advanced testing
sysinfo = "0.29"             # System resource monitoring

# Performance testing for integration
criterion = { version = "0.5", features = ["html_reports", "async_tokio"] }
```

## Integrated Workflow Test Scenarios

### Phase 1: Foundation Integration Tests (Must Have - Build Core Confidence)

#### Test Suite 1: Basic Integrated Workflow (Default Behavior)

**IW-001: Default integrated processing of single cpinfo file**
- **Input**: `cpinfo-parser my_file.cpinfo output/`
- **Expected Workflow**:
  1. **Phase 1**: Extract sections from cpinfo → `output/results/general/`, `output/results/security/`, etc.
  2. **Phase 2**: Automatically parse each section file → `output/results/general/cmd_*.txt`, `output/results/general/file_*.txt`
  3. **Output Structure**: Hierarchical with both section files AND parsed command/file outputs
- **Purpose**: Validate core integrated workflow as default behavior

```rust
#[test]
fn test_default_integrated_processing() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.cpinfo");
    let output_dir = temp_dir.path().join("output");
    
    // Create test cpinfo with multiple sections
    create_test_cpinfo_with_sections(&input_file);
    
    let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
    cmd.arg(&input_file)
       .arg(&output_dir);
    
    cmd.assert().success();
    
    // Verify Phase 1: Section extraction occurred
    assert!(output_dir.join("results/general").exists());
    assert!(output_dir.join("results/security").exists());
    
    // Verify Phase 2: Section files were automatically parsed
    assert!(output_dir.join("results/general/cmd_CP_Status_FW.txt").exists());
    assert!(output_dir.join("results/security/cmd_fw_ctl_pstat.txt").exists());
    
    // Verify both section files AND parsed outputs exist
    assert!(output_dir.join("results/general/CP_Status.txt").exists());  // Section file
    assert!(output_dir.join("results/general/cmd_CP_Status_FW.txt").exists());  // Parsed command
}
```

**IW-002: Progress reporting for two-phase integrated processing**
- **Input**: Large cpinfo file with `--progress` flag
- **Expected**: Progress updates for both phases with clear phase identification
- **CLI**: `cpinfo-parser large_file.cpinfo output/ --progress`
- **Purpose**: Ensure users understand two-phase processing progress

**IW-003: Integrated processing maintains directory structure**
- **Input**: cpinfo file with multiple section types
- **Expected**: Output structure preserves section organization with parsed content
- **Structure**: `output/results/general/{section_files + cmd_*.txt + file_*.txt}`
- **Purpose**: Validate organizational consistency through integrated workflow

**IW-004: Memory efficiency during integrated processing**
- **Input**: 1GB cpinfo file processed with integrated workflow
- **Expected**: Memory usage stays under 500MB throughout both phases
- **Purpose**: Ensure streaming architecture works end-to-end

**IW-005: Error recovery between phases**
- **Input**: cpinfo file where Phase 1 succeeds but Phase 2 encounters errors
- **Expected**: Phase 1 results preserved, Phase 2 errors reported, partial results available
- **Purpose**: Validate graceful degradation in integrated workflow

#### Test Suite 2: CLI Integration and Backward Compatibility

**IW-006: Backward compatibility with existing CLI usage**
- **Input**: Existing CLI commands should continue working unchanged
- **Test Cases**:
  - `cpinfo-parser file.cpinfo output/` (now does integrated workflow by default)
  - `cpinfo-parser --section-file section.txt output/` (section-only parsing unchanged)
  - `cpinfo-parser --read-only file.cpinfo` (analysis-only mode unchanged)
- **Purpose**: Ensure no breaking changes to existing functionality

**IW-007: Override integrated workflow with phase control flags**
- **Input**: `cpinfo-parser file.cpinfo output/ --extract-only`
- **Expected**: Only Phase 1 (section extraction) performed, Phase 2 (parsing) skipped
- **Purpose**: Provide user control over workflow phases

**IW-008: Force section parsing phase with existing extracted sections**
- **Input**: `cpinfo-parser output/results/ --parse-sections-only`
- **Expected**: Skip Phase 1, only perform Phase 2 on existing section files
- **Purpose**: Allow reprocessing of previously extracted sections

**IW-009: Selective section parsing in integrated workflow**
- **Input**: `cpinfo-parser file.cpinfo output/ --sections=general,security`
- **Expected**: Integrated workflow only processes specified section types
- **Purpose**: Optimize processing for specific use cases

**IW-010: Help system explains integrated workflow clearly**
- **Input**: `cpinfo-parser --help`
- **Expected**: Help clearly explains default integrated behavior and phase control options
- **Purpose**: User education about new default behavior

#### Test Suite 3: Performance and Scalability Integration

**IW-011: Large file integrated processing performance**
- **Input**: Multi-GB cpinfo file with hundreds of sections
- **Expected**: Complete integrated workflow within performance targets
- **Metrics**: 
  - Phase 1: >50MB/s extraction rate
  - Phase 2: >20MB/s section parsing rate
  - Total time < 2x Phase 1 alone
- **Purpose**: Validate performance scalability of integrated workflow

**IW-012: Concurrent section parsing during integrated workflow**
- **Input**: cpinfo file with many section files
- **Expected**: Phase 2 processes section files in parallel when beneficial
- **Performance**: Near-linear speedup with available CPU cores
- **Purpose**: Optimize integrated workflow throughput

**IW-013: Memory pressure handling during integrated workflow**
- **Input**: Large cpinfo processing with limited memory
- **Expected**: Adaptive behavior, graceful degradation, clear memory warnings
- **Purpose**: Resource constraint handling in integrated workflow

**IW-014: Streaming integration validation**
- **Input**: Very large cpinfo file (>5GB)
- **Expected**: Both phases use streaming architecture, constant memory usage
- **Purpose**: Validate streaming works end-to-end through integrated workflow

**IW-015: Batch integrated processing**
- **Input**: Multiple cpinfo files processed with integrated workflow
- **Expected**: Each file gets complete two-phase processing, results organized separately
- **Purpose**: Multi-file integrated workflow validation

### Phase 2: Error Handling and Recovery Integration (Must Have - Reliability)

#### Test Suite 4: Cross-Phase Error Scenarios

**IW-016: Corrupted cpinfo file error handling**
- **Input**: Partially corrupted cpinfo file
- **Expected**: Phase 1 extracts valid sections, Phase 2 processes extractable content
- **Recovery**: Comprehensive error report with actionable guidance
- **Purpose**: Validate error isolation between phases

**IW-017: Disk space exhaustion during integrated processing**
- **Input**: Large cpinfo file processed with limited disk space
- **Expected**: Graceful failure with cleanup, partial results preserved if useful
- **Recovery**: Clear disk space error message with space requirements
- **Purpose**: Resource exhaustion handling across phases

**IW-018: Permission errors during Phase 2 processing**
- **Input**: Phase 1 succeeds but Phase 2 encounters write permission errors
- **Expected**: Phase 1 results preserved, Phase 2 permission errors clearly reported
- **Recovery**: Specific guidance on fixing permission issues
- **Purpose**: Permission handling in integrated workflow

**IW-019: Malformed section files during Phase 2**
- **Input**: Phase 1 extracts sections with malformed delimiter patterns
- **Expected**: Phase 2 processes valid sections, reports malformed ones clearly
- **Recovery**: Continue processing other sections, provide format guidance
- **Purpose**: Validate Phase 2 error recovery with Phase 1 output

**IW-020: Interrupted processing recovery**
- **Input**: Integrated workflow interrupted partway through (Ctrl+C, system restart)
- **Expected**: Resume capability or clear restart guidance with progress preservation
- **Recovery**: Detect partial processing state, offer continuation options
- **Purpose**: Interruption handling in long-running integrated workflows

#### Test Suite 5: Output Validation and Verification

**IW-021: Output completeness verification**
- **Input**: cpinfo file with known section count and content
- **Expected**: All expected outputs present in correct locations with verification
- **Validation**: Automated output audit with missing file detection
- **Purpose**: Ensure integrated workflow completeness

**IW-022: Content integrity verification across phases**
- **Input**: cpinfo file processed through integrated workflow
- **Expected**: Original content preserved exactly through both phases
- **Validation**: Checksum comparison between source and final extracted content
- **Purpose**: Validate data integrity through complete workflow

**IW-023: File naming consistency validation**
- **Input**: Complex section names with special characters
- **Expected**: Consistent naming between section files and parsed outputs
- **Validation**: Predictable, reversible naming scheme throughout workflow
- **Purpose**: File organization consistency validation

**IW-024: Large output directory organization**
- **Input**: cpinfo file producing thousands of output files
- **Expected**: Logical organization, no filesystem limits exceeded
- **Validation**: Directory structure remains navigable and performant
- **Purpose**: Large-scale output organization validation

**IW-025: Output metadata and indexing**
- **Input**: Integrated workflow processing
- **Expected**: Metadata files documenting workflow phases and outputs
- **Validation**: Machine-readable processing index with workflow details
- **Purpose**: Workflow documentation and traceability

### Phase 3: User Experience and Professional Workflows (Should Have - Usability)

#### Test Suite 6: Network Security Professional Workflows

**IW-026: Network Administrator quick analysis workflow**
- **Input**: `cpinfo-parser incident.cpinfo output/ --priority=commands --progress`
- **Expected**: Integrated workflow prioritizes command extraction for rapid analysis
- **UX**: Progress shows command extraction priority, ETA for full workflow
- **Purpose**: Optimize workflow for incident response scenarios

**IW-027: Security Engineer comprehensive analysis workflow**
- **Input**: `cpinfo-parser gateway.cpinfo output/ --security --verbose`
- **Expected**: Integrated workflow with enhanced security controls and detailed logging
- **UX**: Security-focused progress reporting, sensitive data handling alerts
- **Purpose**: Security-focused integrated workflow validation

**IW-028: Support Engineer TAC submission workflow**
- **Input**: `cpinfo-parser case.cpinfo output/ --tac-format`
- **Expected**: Integrated workflow produces TAC-compatible output organization
- **UX**: TAC submission guidance, compatible file naming and structure
- **Purpose**: TAC workflow integration validation

**IW-029: Batch analysis workflow for fleet management**
- **Input**: Multiple cpinfo files from gateway fleet
- **Expected**: Integrated workflow processes each with fleet-wide result organization
- **UX**: Fleet-level progress reporting, comparative analysis preparation
- **Purpose**: Enterprise fleet management workflow

**IW-030: Forensic analysis workflow with chain of custody**
- **Input**: `cpinfo-parser evidence.cpinfo output/ --forensic --audit`
- **Expected**: Integrated workflow with comprehensive audit trail and integrity verification
- **UX**: Forensic-grade documentation, chain of custody maintenance
- **Purpose**: Legal and forensic workflow compliance

#### Test Suite 7: Accessibility and Documentation

**IW-031: Screen reader compatible integrated workflow progress**
- **Input**: Integrated processing with screen reader simulation
- **Expected**: Text-based progress updates suitable for accessibility tools
- **Compliance**: WCAG 2.1 AA compliance throughout workflow
- **Purpose**: Accessibility validation for integrated workflow

**IW-032: Comprehensive workflow documentation in help system**
- **Input**: `cpinfo-parser --help`, `cpinfo-parser --examples`
- **Expected**: Clear explanation of integrated workflow with practical examples
- **Documentation**: Step-by-step workflow guidance, phase explanations
- **Purpose**: Self-documenting integrated workflow

**IW-033: Error message clarity for integrated workflow failures**
- **Input**: Various failure scenarios in integrated workflow
- **Expected**: Clear phase identification in errors, specific remediation guidance
- **UX**: Non-technical users can understand which phase failed and why
- **Purpose**: Error usability in integrated workflow

**IW-034: Progress estimation accuracy for integrated workflow**
- **Input**: Large cpinfo files with progress monitoring
- **Expected**: Accurate time estimates accounting for both phases
- **UX**: Progressive refinement of estimates as processing advances
- **Purpose**: Progress reporting accuracy validation

**IW-035: Workflow customization and preferences**
- **Input**: User preferences for integrated workflow behavior
- **Expected**: Configuration files and environment variables control workflow
- **UX**: Persistent preferences, per-project customization options
- **Purpose**: Workflow personalization validation

### Phase 4: Advanced Integration and Edge Cases (Could Have - Comprehensive Coverage)

#### Test Suite 8: Complex Integration Scenarios

**IW-036: VSX cluster integrated processing**
- **Input**: Large VSX cluster cpinfo with multiple virtual systems
- **Expected**: Integrated workflow organizes by VS context, processes VS-specific sections
- **Complexity**: Thousands of VS-specific files with cross-VS analysis preparation
- **Purpose**: Complex enterprise environment workflow validation

**IW-037: Multi-version cpinfo compatibility in integrated workflow**
- **Input**: cpinfo files from R80.10, R81.20, R82 processed with same workflow
- **Expected**: Version-adaptive integrated processing with consistent output
- **Compatibility**: Backward and forward compatibility validation
- **Purpose**: Version compatibility in integrated workflow

**IW-038: Malformed input resilience in integrated workflow**
- **Input**: cpinfo files with various corruption types and malformation
- **Expected**: Maximum extraction with graceful handling of malformed sections
- **Resilience**: Robust error recovery, useful partial results
- **Purpose**: Real-world input handling validation

**IW-039: Resource-constrained environment integrated processing**
- **Input**: Large cpinfo processing on low-resource systems
- **Expected**: Adaptive workflow with resource-aware optimization
- **Adaptation**: Memory limits, CPU throttling, disk space management
- **Purpose**: Resource constraint adaptation validation

**IW-040: Security-sensitive integrated workflow**
- **Input**: cpinfo files with sensitive data requiring special handling
- **Expected**: Integrated workflow with data sanitization and security controls
- **Security**: GDPR compliance, sensitive data filtering, audit trails
- **Purpose**: Security compliance in integrated workflow

#### Test Suite 9: Property-Based Integration Testing

**IW-041: Workflow determinism property**
```rust
proptest! {
    #[test]
    fn integrated_workflow_deterministic(
        cpinfo_content in valid_cpinfo_strategy(),
        output_path in valid_path_strategy()
    ) {
        let result1 = run_integrated_workflow(&cpinfo_content, &output_path)?;
        let result2 = run_integrated_workflow(&cpinfo_content, &output_path)?;
        
        prop_assert_eq!(result1, result2);
        prop_assert!(outputs_identical(&output_path, &result1, &result2));
    }
}
```
- **Purpose**: Validate integrated workflow produces consistent results

**IW-042: Content preservation property across integrated workflow**
```rust
proptest! {
    #[test]
    fn content_preservation_through_workflow(
        sections in prop::collection::vec(valid_section_strategy(), 1..50)
    ) {
        let cpinfo = create_cpinfo_from_sections(&sections);
        let output = run_integrated_workflow(&cpinfo)?;
        
        // Verify all original content is preserved somewhere in output
        for section in sections {
            prop_assert!(content_exists_in_output(&output, &section.content));
        }
    }
}
```
- **Purpose**: Ensure no content loss through integrated workflow

**IW-043: Performance scaling property for integrated workflow**
```rust
proptest! {
    #[test]
    fn integrated_workflow_scales_linearly(
        base_size in 1..10_usize,
        scaling_factor in 1..5_usize
    ) {
        let small_file = create_test_cpinfo(base_size * 1024 * 1024);
        let large_file = create_test_cpinfo(base_size * scaling_factor * 1024 * 1024);
        
        let small_time = time_integrated_workflow(&small_file);
        let large_time = time_integrated_workflow(&large_file);
        
        // Processing time should scale sub-linearly or linearly
        let time_ratio = large_time / small_time;
        let size_ratio = scaling_factor as f64;
        
        prop_assert!(time_ratio <= size_ratio * 1.5); // Allow 50% overhead
    }
}
```
- **Purpose**: Validate performance scaling of integrated workflow

**IW-044: Error recovery completeness property**
```rust
proptest! {
    #[test]
    fn error_recovery_preserves_valid_content(
        valid_sections in prop::collection::vec(valid_section_strategy(), 1..20),
        corrupt_sections in prop::collection::vec(corrupt_section_strategy(), 0..5)
    ) {
        let mixed_cpinfo = create_mixed_cpinfo(&valid_sections, &corrupt_sections);
        let result = run_integrated_workflow(&mixed_cpinfo);
        
        // All valid content should be processed successfully
        for valid_section in valid_sections {
            prop_assert!(section_processed_successfully(&result, &valid_section));
        }
        
        // Corrupt sections should be handled gracefully
        prop_assert!(result.errors.len() == corrupt_sections.len());
    }
}
```
- **Purpose**: Validate error recovery preserves valid content

**IW-045: Memory bound property for integrated workflow**
```rust
proptest! {
    #[test]
    fn integrated_workflow_memory_bounded(
        file_size_mb in 10..1000_usize
    ) {
        let test_file = create_test_cpinfo(file_size_mb * 1024 * 1024);
        let initial_memory = get_memory_usage();
        
        run_integrated_workflow(&test_file)?;
        
        let peak_memory = get_peak_memory_usage();
        let memory_increase = peak_memory - initial_memory;
        
        // Memory usage should be bounded regardless of file size
        prop_assert!(memory_increase < 1024 * 1024 * 1024); // <1GB
    }
}
```
- **Purpose**: Validate memory efficiency of integrated workflow

### Phase 5: End-to-End Workflow Validation (Must Have - System Verification)

#### Test Suite 10: Complete Workflow Integration

**IW-046: Enterprise-scale end-to-end workflow**
- **Input**: Real enterprise cpinfo collection (multi-GB, hundreds of sections)
- **Expected**: Complete integrated workflow within enterprise performance targets
- **Validation**: All sections extracted and parsed, professional output organization
- **Purpose**: Enterprise readiness validation

**IW-047: Check Point ecosystem integration**
- **Input**: cpinfo files from actual Check Point environments
- **Expected**: Output compatible with downstream Check Point analysis tools
- **Integration**: SmartConsole compatibility, TAC submission format validation
- **Purpose**: Ecosystem integration validation

**IW-048: Multi-environment workflow consistency**
- **Input**: cpinfo files from various Check Point deployment types
- **Expected**: Consistent integrated workflow behavior across environments
- **Environments**: Standalone, cluster, VSX, management servers
- **Purpose**: Environmental consistency validation

**IW-049: Long-running workflow stability**
- **Input**: Extended processing session with multiple large files
- **Expected**: Stable performance, no memory leaks, consistent behavior
- **Duration**: 24-hour continuous processing simulation
- **Purpose**: Production stability validation

**IW-050: Complete workflow audit and compliance**
- **Input**: Enterprise integrated workflow with full audit requirements
- **Expected**: Complete audit trail, compliance documentation, security validation
- **Compliance**: SOC2, GDPR, enterprise security requirements
- **Purpose**: Enterprise compliance validation

## Implementation Strategy for Integrated Workflow

### Canon TDD Implementation for Integrated Workflow

#### Phase-by-Phase Confidence Building for Integration

**Phase 1: Basic Integration (Tests IW-001 to IW-015) - Core Workflow Confidence**
- **Goal**: Establish reliable integrated workflow foundation
- **Duration**: 3-4 weeks
- **TDD Focus**: Default integrated behavior, CLI integration, basic performance
- **Success Criteria**: Simple cpinfo files processed end-to-end successfully
- **Implementation Notes**: Build on existing section extraction, add automatic section parsing

**Phase 2: Error Handling Integration (Tests IW-016 to IW-025) - Reliability**
- **Goal**: Robust error handling across workflow phases
- **Duration**: 2-3 weeks
- **TDD Focus**: Cross-phase error recovery, output validation, integrity verification
- **Success Criteria**: Graceful handling of errors in either phase
- **Implementation Notes**: Implement phase isolation, partial result preservation

**Phase 3: User Experience Integration (Tests IW-026 to IW-035) - Professional Usability**
- **Goal**: Professional-grade user experience for integrated workflow
- **Duration**: 2-3 weeks
- **TDD Focus**: Role-based workflows, accessibility, documentation
- **Success Criteria**: Network security professionals can use effectively
- **Implementation Notes**: Focus on workflow guidance, progress reporting

**Phase 4: Advanced Integration (Tests IW-036 to IW-050) - Production Readiness**
- **Goal**: Enterprise-grade integrated workflow
- **Duration**: 3-4 weeks
- **TDD Focus**: Complex scenarios, property-based testing, end-to-end validation
- **Success Criteria**: Production deployment ready with enterprise features
- **Implementation Notes**: Comprehensive edge case handling, performance optimization

### Integration Test Framework Configuration

#### Integrated Workflow Test Utilities
```rust
// tests/common/integrated_workflow_utils.rs
use std::process::Command;
use tempfile::TempDir;
use assert_cmd::Command as AssertCommand;

pub struct IntegratedWorkflowTester {
    temp_dir: TempDir,
    cpinfo_file: PathBuf,
    output_dir: PathBuf,
}

impl IntegratedWorkflowTester {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let cpinfo_file = temp_dir.path().join("test.cpinfo");
        let output_dir = temp_dir.path().join("output");
        
        Self { temp_dir, cpinfo_file, output_dir }
    }
    
    pub fn with_test_cpinfo(&mut self, sections: &[TestSection]) -> &mut Self {
        create_test_cpinfo_with_sections(&self.cpinfo_file, sections);
        self
    }
    
    pub fn run_integrated_workflow(&self) -> IntegratedWorkflowResult {
        let mut cmd = AssertCommand::cargo_bin("cpinfo-parser").unwrap();
        cmd.arg(&self.cpinfo_file)
           .arg(&self.output_dir);
        
        let assert = cmd.assert();
        IntegratedWorkflowResult::from_assert(assert, &self.output_dir)
    }
    
    pub fn run_with_flags(&self, flags: &[&str]) -> IntegratedWorkflowResult {
        let mut cmd = AssertCommand::cargo_bin("cpinfo-parser").unwrap();
        cmd.arg(&self.cpinfo_file)
           .arg(&self.output_dir);
        
        for flag in flags {
            cmd.arg(flag);
        }
        
        let assert = cmd.assert();
        IntegratedWorkflowResult::from_assert(assert, &self.output_dir)
    }
    
    pub fn verify_phase_1_output(&self) -> bool {
        // Verify section extraction occurred
        self.output_dir.join("results").exists() &&
        self.output_dir.join("results/general").exists()
    }
    
    pub fn verify_phase_2_output(&self) -> bool {
        // Verify section parsing occurred
        self.has_command_files() && self.has_file_outputs()
    }
    
    pub fn verify_integrated_output(&self) -> bool {
        self.verify_phase_1_output() && self.verify_phase_2_output()
    }
    
    fn has_command_files(&self) -> bool {
        walkdir::WalkDir::new(&self.output_dir)
            .into_iter()
            .any(|entry| {
                entry.map(|e| e.file_name().to_string_lossy().starts_with("cmd_"))
                     .unwrap_or(false)
            })
    }
    
    fn has_file_outputs(&self) -> bool {
        walkdir::WalkDir::new(&self.output_dir)
            .into_iter()
            .any(|entry| {
                entry.map(|e| e.file_name().to_string_lossy().starts_with("file_"))
                     .unwrap_or(false)
            })
    }
}

pub struct IntegratedWorkflowResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub output_dir: PathBuf,
    pub phase_1_completed: bool,
    pub phase_2_completed: bool,
}

impl IntegratedWorkflowResult {
    pub fn from_assert(assert: assert_cmd::assert::Assert, output_dir: &Path) -> Self {
        let output = assert.get_output();
        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        let phase_1_completed = output_dir.join("results").exists();
        let phase_2_completed = has_parsed_outputs(output_dir);
        
        Self {
            success,
            stdout,
            stderr,
            output_dir: output_dir.to_path_buf(),
            phase_1_completed,
            phase_2_completed,
        }
    }
    
    pub fn verify_complete_integration(&self) -> bool {
        self.success && self.phase_1_completed && self.phase_2_completed
    }
}

// Test data generation for integrated workflow
pub fn create_test_cpinfo_with_sections(file_path: &Path, sections: &[TestSection]) {
    let mut content = String::new();
    content.push_str("Check Point Support Information\n");
    content.push_str("===============================\n\n");
    
    for section in sections {
        content.push_str(&format!("{}\n", section.header));
        content.push_str(&format!("{}\n", "=".repeat(section.header.len())));
        content.push_str(&section.content);
        content.push_str("\n\n");
    }
    
    std::fs::write(file_path, content).unwrap();
}

pub struct TestSection {
    pub header: String,
    pub content: String,
    pub section_type: SectionType,
}

pub enum SectionType {
    General,
    Security,
    Network,
    Misc,
}

impl TestSection {
    pub fn general_with_commands() -> Self {
        Self {
            header: "General Info".to_string(),
            content: format!(
                "------------------------\nCP Status - FW\n------------------------\n{}\n",
                "Product name: Firewall\nPolicy name: firewall-1\nDate: 2024-01-01"
            ),
            section_type: SectionType::General,
        }
    }
    
    pub fn security_with_mixed_content() -> Self {
        Self {
            header: "Security Info".to_string(),
            content: format!(
                "{}\n{}\n",
                "-----------------------\nfw ctl pstat\n-----------------------\nConnections: 57924\nPeak: 99980\n",
                "------------------------------------------------------------------\n/opt/CPsuite-R81.10/conf/objects.C\n------------------------------------------------------------------\n# Configuration file\n: (network_objects\n"
            ),
            section_type: SectionType::Security,
        }
    }
}
```

### CLI Integration for Integrated Workflow

#### Enhanced CLI Specification
```rust
// src/cli.rs - Enhanced for integrated workflow
use clap::{Parser, ArgGroup};

#[derive(Parser)]
#[command(name = "cpinfo-parser")]
#[command(about = "Parse and extract sections from Check Point cpinfo files")]
#[command(long_about = "A high-performance streaming parser for Check Point diagnostic cpinfo files.\n\
                       \n\
                       DEFAULT BEHAVIOR (Integrated Workflow):\n\
                       - Phase 1: Extract sections from cpinfo file\n\
                       - Phase 2: Automatically parse section files into individual commands/files\n\
                       - Result: Complete organized output with both section files and parsed content\n\
                       \n\
                       Examples:\n\
                       \n\
                       Integrated workflow (default):\n\
                         cpinfo-parser gateway.cpinfo output/\n\
                       \n\
                       Phase control:\n\
                         cpinfo-parser gateway.cpinfo output/ --extract-only\n\
                         cpinfo-parser output/results/ --parse-sections-only\n\
                       \n\
                       Professional workflows:\n\
                         cpinfo-parser incident.cpinfo output/ --priority=commands\n\
                         cpinfo-parser forensic.cpinfo output/ --security --audit")]
#[command(group = ArgGroup::new("workflow_control").args(&["extract_only", "parse_sections_only"]))]
pub struct Args {
    /// Input cpinfo file or section directory
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,
    
    /// Output directory for extraction results
    #[arg(short = 'o', long = "output", value_name = "DIR")]
    pub output: Option<PathBuf>,
    
    /// Only perform Phase 1 (section extraction), skip Phase 2 (section parsing)
    #[arg(long, help = "Extract sections only, skip automatic parsing")]
    pub extract_only: bool,
    
    /// Only perform Phase 2 (section parsing) on existing extracted sections
    #[arg(long, help = "Parse sections only, skip extraction phase")]
    pub parse_sections_only: bool,
    
    /// Process only specified section types
    #[arg(long, value_delimiter = ',', 
          help = "Process only specified sections (general,security,network,misc)")]
    pub sections: Option<Vec<String>>,
    
    /// Processing priority for integrated workflow
    #[arg(long, value_enum, help = "Processing priority (commands, files, balanced)")]
    pub priority: Option<ProcessingPriority>,
    
    /// Enable progress reporting for both phases
    #[arg(long, help = "Show progress for integrated workflow")]
    pub progress: bool,
    
    /// Enhanced security controls for sensitive data
    #[arg(long, help = "Enable security controls for sensitive content")]
    pub security: bool,
    
    /// Read-only mode (analysis without extraction)
    #[arg(long, help = "Analyze structure without creating output files")]
    pub read_only: bool,
    
    /// Verbose output with phase details
    #[arg(short, long, help = "Detailed logging for both workflow phases")]
    pub verbose: bool,
    
    /// Enable audit trail for compliance
    #[arg(long, help = "Generate audit trail for compliance requirements")]
    pub audit: bool,
    
    /// TAC-compatible output format
    #[arg(long, help = "Format output for Check Point TAC submission")]
    pub tac_format: bool,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum ProcessingPriority {
    Commands,  // Prioritize command extraction
    Files,     // Prioritize file content extraction
    Balanced,  // Equal priority (default)
}
```

### Performance Benchmarks for Integrated Workflow

#### Integrated Workflow Benchmarks
```rust
// benches/integrated_workflow.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

fn integrated_workflow_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("integrated_workflow");
    group.measurement_time(Duration::from_secs(30));
    
    for size_mb in [1, 10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("complete_workflow", size_mb),
            size_mb,
            |b, &size| {
                let test_data = create_test_cpinfo_mb(size);
                b.iter(|| {
                    run_complete_integrated_workflow(&test_data)
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("phase_1_only", size_mb),
            size_mb,
            |b, &size| {
                let test_data = create_test_cpinfo_mb(size);
                b.iter(|| {
                    run_extraction_phase_only(&test_data)
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("phase_2_only", size_mb),
            size_mb,
            |b, &size| {
                let test_sections = create_test_sections_mb(size);
                b.iter(|| {
                    run_parsing_phase_only(&test_sections)
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, integrated_workflow_benchmarks);
criterion_main!(benches);
```

## Test Execution Strategy for Integrated Workflow

### TDD Implementation Process for Integration

#### Red-Green-Refactor for Integrated Tests
1. **Pick ONE integrated test** from the prioritized list (start with IW-001)
2. **Write integration test** that defines expected end-to-end behavior
3. **Run test** - it should FAIL because integrated workflow doesn't exist yet
4. **Implement minimal integration** to make test pass (combine existing phases)
5. **Refactor** while keeping integration tests green
6. **Repeat** with next integration test
7. **Add discovered scenarios** as edge cases emerge during integration

#### Integration Test Execution Commands
```bash
# Run all integrated workflow tests
cargo test integrated_workflow

# Run specific integration test suites
cargo test basic_integration
cargo test error_recovery_integration  
cargo test user_experience_integration
cargo test advanced_integration

# Run integration tests with progress monitoring
cargo test integrated_workflow -- --nocapture --test-threads=1

# Run integration benchmarks
cargo bench integrated_workflow

# Run integration tests with memory profiling
DHAT_PROFILING=1 cargo test integrated_workflow
```

## Summary for Lead Developer Implementation

### Missing Integrated Workflow Functionality

The current implementation successfully extracts sections from cpinfo files but **lacks the integrated workflow** where section parsing happens automatically as the default behavior.

**Current State:**
- ✅ Phase 1: cpinfo → section files (`results/general/CP_Status.txt`)
- ❌ Phase 2: section files → parsed outputs (`cmd_*.txt`, `file_*.txt`) [MISSING INTEGRATION]

**Required Integration:**
- Default behavior: `cpinfo-parser file.cpinfo output/` performs BOTH phases
- Preserve existing CLI for individual operations
- Add phase control flags for advanced users
- Maintain performance and error handling across phases

### Implementation Roadmap

#### Immediate Integration Tasks

1. **Implement `IntegratedWorkflowOrchestrator`** (`src/integrated_workflow.rs`)
   - Coordinate Phase 1 (existing) → Phase 2 (section parsing)
   - Handle phase errors and partial results
   - Provide integrated progress reporting

2. **Enhance CLI for default integrated behavior** (`src/cli.rs`)
   - Make integrated workflow the default
   - Add `--extract-only` and `--parse-sections-only` flags
   - Maintain backward compatibility

3. **Add integrated workflow tests** (`tests/integration/integrated_workflow.rs`)
   - Start with Test IW-001 (basic integrated processing)
   - Follow TDD red-green-refactor for each test
   - Build confidence incrementally

4. **Performance optimization for integrated workflow**
   - Streaming integration between phases
   - Memory efficiency across phases
   - Parallel section processing where beneficial

### Success Criteria for Integrated Workflow

- ✅ **Default Behavior**: `cpinfo-parser file.cpinfo output/` performs complete workflow
- ✅ **Backward Compatibility**: Existing CLI commands work unchanged
- ✅ **Performance**: Integrated workflow meets enterprise performance targets
- ✅ **Error Recovery**: Graceful handling of errors in either phase
- ✅ **User Experience**: Professional workflows for network security professionals
- ✅ **Documentation**: Clear explanation of integrated workflow in help system

This comprehensive test scenarios document provides the foundation for implementing the integrated workflow through systematic TDD validation, ensuring the cpinfo parser becomes a powerful, user-friendly tool that automatically provides complete processing by default while maintaining the flexibility for advanced users to control individual phases as needed.