//! CpinfoParser method implementations
//!
//! This module contains the implementation of all CpinfoParser methods;
//! organized by functional area and delegating to appropriate facades.

use std::path::Path;

use crate::parser::config::{,
    MonitoringConfig, NetworkConfig, PartialRecoveryConfig, PerformanceConfig, RetryConfig},
use crate::parser::facade::core::CoreParsingFacade,
use crate::parser::facade::{
    ConcurrencyFacade, DiagnosticsFacade, EnterpriseFacade, ExtractionFacade, MonitoringFacade,,
    NetworkFacade, RecoveryFacade, ResourceFacade, ValidationFacade},
use crate::parser::stats::{
    CacheStats, ConcurrentStats, CpuThrottleResult, DiagnosticInfo, DiskConstraintResult,
    EnterpriseStats, LoadBalanceStats, MemoryStats, MonitoringResult, NetworkResult,
    NodeRecoveryResult, ParseResult, ProfilingStats, ReliabilityStats, ResourceConstraintResult,,
    ResourceStats, RetryResult, SectionRecoveryResult, SpeedStats},

use super::CpinfoParser,

impl CpinfoParser {
    /// Detect the format of a cpinfo file.
    pub fn detect_format<P: AsRef<Path>>(
        &self,
        path: P) -> crate::Result<crate::format::CpinfoFormat> {
        return CoreParsingFacade::detect_format(path)
    }

    /// Parse a cpinfo file (async-friendly wrapper).
    pub async fn parse_file<P: AsRef<Path>>(&self, path: P) -> crate::Result<ParseResult> {
        return Ok(CoreParsingFacade::parse_file(path)?)
    }

    /// Identify section delimiters in a cpinfo file
    pub fn identify_section_delimiters<P: AsRef<Path>>(
        &self,
        path: P) -> crate::Result<Vec<crate::section::SectionDelimiter>> {
        return CoreParsingFacade::identify_section_delimiters(path)
    }

    /// Extract sections (basic) to an output directory.
    pub fn extract_sections<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: Q) -> crate::Result<crate::extraction::ExtractionResult> {
        return ExtractionFacade::extract_sections(input_path, output_path)
    }

    /// Extract sections in an organized manner.
    pub fn extract_sections_organized<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: Q) -> crate::Result<crate::extraction::OrganizedExtractionResult> {
        return ExtractionFacade::extract_sections_organized(input_path, output_path)
    }

    /// Extract sections with VSX-aware detection
    pub fn extract_sections_with_vsx_detection<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: Q) -> crate::Result<crate::extraction::OrganizedExtractionResult> {
        return ExtractionFacade::extract_sections_with_vsx_detection(input_path, output_path)
    }

    /// Extract sections with binary detection
    pub fn extract_sections_with_binary_detection<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: Q) -> crate::Result<crate::extraction::BinaryDetectionResult> {
        return ExtractionFacade::extract_sections_with_binary_detection(input_path, output_path)
    }

    /// Extract with partial recovery (delegate to recovery module)
    pub fn extract_sections_with_partial_recovery<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_dir: P,
        config: PartialRecoveryConfig) -> crate::Result<SectionRecoveryResult> {
        return RecoveryFacade::extract_sections_with_partial_recovery(input_path, output_dir, config)
    }

    /// Checkpointing variants are treated as aliases for now
    pub fn extract_sections_with_checkpointing<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_dir: P,
        config: PartialRecoveryConfig) -> crate::Result<SectionRecoveryResult> {
        return RecoveryFacade::extract_sections_with_checkpointing(input_path, output_dir, config)
    }

    /// Extract sections with checkpoint recovery
    pub fn extract_sections_with_checkpoint_recovery<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_dir: P,
        config: PartialRecoveryConfig) -> crate::Result<SectionRecoveryResult> {
        return RecoveryFacade::extract_sections_with_checkpoint_recovery(input_path, output_dir, config)
    }

    /// Parse with speed monitoring enabled.
    pub fn parse_with_speed_monitoring<P: AsRef<Path>>(
        file_path: P,
        config: &PerformanceConfig) -> crate::Result<SpeedStats> {
        return MonitoringFacade::parse_with_speed_monitoring(file_path, config)
    }

    /// Parse with memory monitoring (associated function)
    pub fn parse_with_memory_monitoring<P: AsRef<Path>>(
        file_path: P,
        config: &PerformanceConfig) -> crate::Result<MemoryStats> {
        return MonitoringFacade::parse_with_memory_monitoring(file_path, config)
    }

    /// Parse multiple files concurrently.
    pub async fn parse_concurrent<P: AsRef<Path> + Send + 'static>(
        file_paths: Vec<P>,
        _config: PerformanceConfig) -> crate::Result<ConcurrentStats> {
        return ConcurrencyFacade::process_files_concurrent(file_paths).await
    }

    /// Async concurrent parse of a single file
    pub async fn parse_file_concurrent<P: AsRef<Path>>(
        &self,
        path: P) -> crate::Result<ParseResult> {
        return CoreParsingFacade::parse_file_concurrent(path).await
    }

    /// Network timeout parsing simulation
    pub fn parse_with_network_timeout<P: AsRef<Path>>(
        &self,
        path: P,
        config: &NetworkConfig) -> crate::Result<NetworkResult> {
        return NetworkFacade::parse_with_network_timeout(path, config)
    }

    /// Distributed node failure simulation
    pub fn parse_with_node_failure_simulation<P: AsRef<Path>>(
        &self,
        path: P,
        config: &NetworkConfig) -> crate::Result<NodeRecoveryResult> {
        return NetworkFacade::parse_with_node_failure_simulation(path, config)
    }

    /// Connection pooling parsing simulation
    pub fn parse_multiple_with_connection_pooling<P: AsRef<Path>>(
        &self,
        paths: Vec<P>,
        config: &NetworkConfig) -> crate::Result<crate::parser::stats::ConnectionPoolingResult> {
        return NetworkFacade::parse_multiple_with_connection_pooling(paths, config)
    }

    /// Retry parsing with configurable backoff
    pub fn parse_with_retry_config<P: AsRef<Path>>(
        &self,
        path: P,
        config: RetryConfig) -> crate::Result<RetryResult> {
        return RecoveryFacade::parse_with_retry_config(path, config)
    }

    /// Transient error simulation
    pub fn parse_with_transient_simulation<P: AsRef<Path>>(
        &self,
        path: P,
        config: RetryConfig) -> crate::Result<RetryResult> {
        return RecoveryFacade::parse_with_transient_simulation(path, config)
    }

    /// Resource constraint handling simulation
    pub fn parse_with_resource_constraints<P: AsRef<Path>>(
        &self,
        path: P,
        config: &PerformanceConfig) -> crate::Result<ResourceConstraintResult> {
        return ResourceFacade::parse_with_resource_constraints(path, config)
    }

    /// Disk space constraint handling simulation
    pub fn parse_with_disk_constraints<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        path: P,
        output_dir: Q,
        config: &PerformanceConfig) -> crate::Result<DiskConstraintResult> {
        return ResourceFacade::parse_with_disk_constraints(path, output_dir, config)
    }

    /// CPU throttling simulation
    pub fn parse_with_cpu_throttling<P: AsRef<Path>>(
        &self,
        path: P,
        config: &PerformanceConfig) -> crate::Result<CpuThrottleResult> {
        return ResourceFacade::parse_with_cpu_throttling(path, config)
    }

    /// Performance diagnostics for successful parsing
    pub fn parse_with_performance_diagnostics<P: AsRef<Path>>(
        &self,
        path: P) -> crate::Result<crate::parser::stats::PerformanceDiagnosticResult> {
        return EnterpriseFacade::parse_with_performance_diagnostics(path)
    }

    /// Enterprise monitoring integration simulation
    pub fn parse_with_monitoring<P: AsRef<Path>>(
        &self,
        path: P,
        config: &MonitoringConfig) -> crate::Result<MonitoringResult> {
        return EnterpriseFacade::parse_with_enterprise_monitoring(path, config)
    }

    /// Comprehensive resource monitoring simulation
    pub fn parse_with_resource_monitoring<P: AsRef<Path>>(
        path: P,
        config: &PerformanceConfig) -> crate::Result<ResourceStats> {
        return ResourceFacade::parse_with_resource_monitoring(path, config)
    }

    /// Caching system simulation
    pub fn parse_with_caching<P: AsRef<Path>>(
        path: P,
        config: &PerformanceConfig) -> crate::Result<CacheStats> {
        return EnterpriseFacade::parse_with_caching(path, config)
    }

    /// Load balancing simulation
    pub fn parse_with_load_balancing<P: AsRef<Path>>(
        paths: Vec<P>,
        config: &PerformanceConfig) -> crate::Result<LoadBalanceStats> {
        return EnterpriseFacade::parse_with_load_balancing(paths, config)
    }

    /// Profiling simulation
    pub fn parse_with_profiling<P: AsRef<Path>>(
        path: P,
        config: &PerformanceConfig) -> crate::Result<ProfilingStats> {
        return EnterpriseFacade::parse_with_profiling(path, config)
    }

    /// Enterprise scale simulation
    pub fn parse_enterprise_scale<P: AsRef<Path>>(
        paths: Vec<P>,
        config: &PerformanceConfig) -> crate::Result<EnterpriseStats> {
        return EnterpriseFacade::parse_enterprise_scale(paths, config)
    }

    /// Reliability testing simulation
    pub fn parse_with_reliability_testing<P: AsRef<Path>>(
        paths: Vec<P>,
        config: &PerformanceConfig) -> crate::Result<ReliabilityStats> {
        return EnterpriseFacade::parse_with_reliability_testing(paths, config)
    }

    /// End-to-end processing helper for integration test 65
    pub fn parse_file_end_to_end<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        input_path: P1,
        output_dir: P2) -> crate::Result<crate::parser::stats::EndToEndProcessingResult> {
        return CoreParsingFacade::parse_file_end_to_end_integration(input_path, output_dir)
    }

    /// Validate input path for security
    pub fn validate_input_path(&self, path: &str) -> crate::Result<()> {
        return ValidationFacade::validate_input_path(path)
    }

    /// Validate a command argument for injection attempts
    pub fn validate_command_argument(&self, arg: &str) -> crate::Result<()> {
        return ValidationFacade::validate_command_argument(arg)
    }

    /// Validate configuration parameter
    pub fn validate_config_parameter(&self, param: &str, value: &str) -> crate::Result<()> {
        return ValidationFacade::validate_config_parameter(param, value)
    }

    /// Parse with diagnostic logging and store last diagnostic info
    pub fn parse_with_diagnostic_logging<P: AsRef<Path>>(&mut self, path: P) -> crate::Result<()> {
        match DiagnosticsFacade::parse_with_diagnostic_logging(path) {
            Ok(info) => {
                self.last_diagnostic = Some(info),
                return Ok(())
            }
            Err(e) => Err(e)}
    }

    /// Get last diagnostic info (or default)
    pub fn get_last_diagnostic_info(&self) -> DiagnosticInfo {
        self.last_diagnostic
            .clone()
            .unwrap_or_else(|| DiagnosticsFacade::default_diagnostic_info())
    }
}
;
