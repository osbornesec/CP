#![allow(
    clippy::module_name_repetitions,
    reason = "API compatibility requires Stats suffix"
)]

use core::time::Duration;
use std::collections::HashMap;
use std::path::PathBuf;

/// Parse result containing processing statistics
#[derive(Debug)]
#[non_exhaustive]
pub struct ParseResult {
    pub bytes_processed: u64,
    pub duration: Duration,
    pub section_count: usize,
}

/// Retry operation result
#[derive(Debug)]
#[non_exhaustive]
pub struct RetryResult {
    pub attempt_count: usize,
    pub section_count: usize,
    pub total_delay: Duration,
    pub transient_errors_recovered: usize,
}

/// Network operation result  
#[derive(Debug)]
#[non_exhaustive]
pub struct NetworkResult {
    pub connection_attempts: usize,
    pub section_count: usize,
    pub successful_connections: usize,
    pub timeout_events: usize,
}

/// Distributed node failure recovery result
#[derive(Debug)]
#[non_exhaustive]
pub struct NodeRecoveryResult {
    pub failed_nodes: usize,
    pub final_processing_node: Option<String>,
    pub recovery_attempts: usize,
    pub section_count: usize,
}

/// Connection pooling result
#[derive(Debug)]
#[non_exhaustive]
pub struct ConnectionPoolingResult {
    pub connection_reuse_rate: f64,
    pub connections_created: usize,
    pub files_processed: usize,
    pub total_processing_time: Duration,
}

/// Section-level recovery processing result
#[derive(Debug)]
#[non_exhaustive]
pub struct SectionRecoveryResult {
    pub checkpoints_created: usize,
    pub failed_sections: usize,
    pub recovery_actions_taken: usize,
    pub valid_sections_processed: usize,
}

/// Checkpointing processing result
#[derive(Debug)]
#[non_exhaustive]
pub struct CheckpointingResult {
    pub checkpoint_files: Vec<PathBuf>,
    pub checkpoints_created: usize,
    pub total_sections_processed: usize,
}

/// Checkpoint recovery result
#[derive(Debug)]
#[non_exhaustive]
pub struct CheckpointRecoveryResult {
    pub last_successful_checkpoint: Option<String>,
    pub sections_recovered_from_checkpoint: usize,
    pub total_sections_processed: usize,
}

/// Memory usage information during parsing
#[derive(Debug)]
#[non_exhaustive]
pub struct MemoryStats {
    pub bytes_processed: u64,
    pub peak_memory_mb: f64,
    pub processing_duration_ms: u64,
    pub sections_extracted: usize,
}

/// Processing speed information
#[derive(Debug)]
#[non_exhaustive]
pub struct SpeedStats {
    pub bytes_per_second: f64,
    pub processing_duration_ms: u64,
    pub sections_per_second: f64,
    pub total_sections: usize,
}

/// Concurrent processing information
#[derive(Debug)]
#[non_exhaustive]
pub struct ConcurrentStats {
    pub average_files_per_second: f64,
    pub files_processed: usize,
    pub peak_memory_mb: f64,
    pub processing_duration_ms: u64,
    pub total_sections: usize,
}

/// Resource monitoring information with real-time metrics
#[derive(Debug)]
#[non_exhaustive]
pub struct ResourceStats {
    pub average_io_rate_mb_per_sec: f64,
    pub cpu_samples: Vec<f64>,
    pub io_samples: Vec<f64>,
    pub memory_samples: Vec<f64>,
    pub peak_cpu_percent: f64,
    pub peak_memory_mb: f64,
    pub total_bytes_read: u64,
    pub total_bytes_written: u64,
    pub total_monitoring_duration_ms: u64,
}

/// Caching performance information
#[derive(Debug)]
#[non_exhaustive]
pub struct CacheStats {
    pub bytes_processed: u64,
    pub cache_hit_rate: f64,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub cache_size_mb: f64,
    pub peak_memory_mb: f64,
    pub processing_duration_ms: u64,
    pub sections_extracted: usize,
}

/// Load balancing information across workers
#[derive(Debug)]
#[non_exhaustive]
pub struct LoadBalanceStats {
    pub files_processed: usize,
    pub load_balance_efficiency: f64,
    pub peak_memory_mb: f64,
    pub total_processing_duration_ms: u64,
    pub worker_coordination_overhead_ms: u64,
    pub worker_utilization: HashMap<usize, f64>,
}

/// Performance bottleneck information
#[derive(Debug)]
#[non_exhaustive]
pub struct PerformanceBottleneck {
    pub operation_name: String,
    pub severity: String,
    pub suggested_optimization: String,
    pub time_percentage: f64,
}

/// Performance profiling information
#[derive(Debug)]
#[non_exhaustive]
pub struct ProfilingStats {
    pub bottlenecks_identified: Vec<PerformanceBottleneck>,
    pub cpu_usage_profile: Vec<f64>,
    pub memory_usage_profile: Vec<f64>,
    pub operation_timings: HashMap<String, u64>,
    pub performance_insights: Vec<String>,
    pub profiling_overhead_ms: u64,
    pub total_processing_time_ms: u64,
}

/// Enterprise-scale processing information
#[derive(Debug)]
#[non_exhaustive]
pub struct EnterpriseStats {
    pub average_cpu_utilization: f64,
    pub failed_files: usize,
    pub files_processed: usize,
    pub peak_memory_mb: f64,
    pub scalability_efficiency: f64,
    pub throughput_mb_per_sec: f64,
    pub total_processing_duration_ms: u64,
    pub worker_coordination_overhead_ms: u64,
}

/// Reliability testing information
#[derive(Debug)]
#[non_exhaustive]
pub struct ReliabilityStats {
    pub degradation_events: usize,
    pub downtime_ms: u64,
    pub files_processed: usize,
    pub handled_errors: usize,
    pub partial_processing_enabled: bool,
    pub peak_memory_mb: f64,
    pub recovery_events: usize,
    pub successful_recoveries: usize,
    pub system_stability_score: f64,
    pub unhandled_errors: usize,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ResourceConstraintResult {
    pub degradation_applied: bool,
    pub peak_memory_mb: f64,
    pub processing_successful: bool,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DiskConstraintResult {
    pub cleanup_triggered: bool,
    pub final_disk_usage_mb: f64,
    pub processing_time_ms: u64,
    pub space_management_applied: bool,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CpuThrottleResult {
    pub adaptive_processing_used: bool,
    pub average_cpu_percent: f64,
    pub processing_time_ms: u64,
    pub throttling_applied: bool,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DiagnosticInfo {
    pub context: String,
    pub correlation_id: String,
    pub error_type: String,
    pub severity_level: u8,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PerformanceDiagnostics {
    pub memory_usage_mb: f64,
    pub processing_stages: Vec<String>,
    pub processing_time_ms: u64,
    pub system_info: SystemInfo,
    pub throughput_mbps: f64,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SystemInfo {
    pub available_memory_mb: f64,
    pub cpu_cores: usize,
    pub os_type: String,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PerformanceDiagnosticResult {
    pub diagnostic_info: PerformanceDiagnostics,
    pub processing_successful: bool,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct MonitoringResult {
    pub alert_thresholds_configured: bool,
    pub exported_metrics: HashMap<String, String>,
    pub health_status: HealthStatus,
    pub monitoring_active: bool,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct HealthStatus {
    pub component_statuses: Vec<(String, bool)>,
    pub is_healthy: bool,
    pub last_check: chrono::DateTime<chrono::Utc>,
}
