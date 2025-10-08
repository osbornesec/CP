use core::time::Duration;

/// Retry configuration for I/O operations
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RetryConfig {
    pub backoff_multiplier: f64,
    pub initial_delay: Duration,
    pub max_attempts: usize,
    pub max_delay: Duration,
    pub retry_on_io_errors: bool,
}

impl Default for RetryConfig {
    #[inline]
    fn default() -> Self {
        return Self {
            backoff_multiplier: 2.0_f64,
            initial_delay: Duration::from_millis(100),
            max_attempts: 3,
            max_delay: Duration::from_secs(30),
            retry_on_io_errors: true,
        };
    }
}

/// Network configuration for distributed processing
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct NetworkConfig {
    pub connection_timeout: Duration,
    pub enable_distributed_mode: bool,
    pub max_retries: usize,
    pub node_health_check_interval: Duration,
    pub read_timeout: Duration,
}

impl Default for NetworkConfig {
    #[inline]
    fn default() -> Self {
        return Self {
            connection_timeout: Duration::from_secs(30),
            enable_distributed_mode: false,
            max_retries: 3,
            node_health_check_interval: Duration::from_secs(5),
            read_timeout: Duration::from_secs(60),
        };
    }
}

/// Partial recovery configuration
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PartialRecoveryConfig {
    pub checkpoint_interval_sections: usize,
    pub enable_section_checkpointing: bool,
    pub max_section_errors: usize,
    pub preserve_partial_sections: bool,
    pub recovery_strategy: RecoveryStrategy,
}

/// Recovery strategy for section-level failures
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum RecoveryStrategy {
    ContinueOnError,
    CreateCheckpoint,
    ResumeFromCheckpoint,
    SkipFailedSection,
}

impl Default for PartialRecoveryConfig {
    #[inline]
    fn default() -> Self {
        return Self {
            checkpoint_interval_sections: 5,
            enable_section_checkpointing: true,
            max_section_errors: 3,
            preserve_partial_sections: true,
            recovery_strategy: RecoveryStrategy::ContinueOnError,
        };
    }
}

/// Performance configuration for parsing operations
///
/// Uses feature flags to manage complexity as an alternative to splitting into separate structs.
/// Boolean flags are grouped by functionality area for better maintainability.
#[derive(Debug, Clone)]
#[non_exhaustive]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Feature flags grouped by functionality area for better maintainability"
)]
pub struct PerformanceConfig {
    pub buffer_size: usize,
    pub cache_size_mb: usize,
    pub cleanup_threshold_percent: f64,
    pub cpu_throttle_threshold: f64,
    pub enable_adaptive_processing: bool,
    pub enable_bottleneck_detection: bool,
    pub enable_caching: bool,
    pub enable_cleanup_on_pressure: bool,
    pub enable_concurrent_processing: bool,
    pub enable_cpu_monitoring: bool,
    pub enable_disk_monitoring: bool,
    pub enable_failure_recovery: bool,
    pub enable_graceful_degradation: bool,
    pub enable_load_balancing: bool,
    pub enable_memory_monitoring: bool,
    pub enable_profiling: bool,
    pub enable_resource_monitoring: bool,
    pub enable_scalability_optimization: bool,
    pub enable_speed_monitoring: bool,
    pub max_concurrent_files: usize,
    pub max_disk_space_mb: usize,
    pub max_memory_mb: usize,
    pub max_processing_threads: usize,
    pub memory_pressure_threshold: f64,
    pub monitoring_interval_ms: u64,
    pub profiling_granularity: String,
    pub worker_count: usize,
}

impl Default for PerformanceConfig {
    #[inline]
    fn default() -> Self {
        return Self {
            buffer_size: 8192,
            cache_size_mb: 50,
            cleanup_threshold_percent: 80.0_f64,
            cpu_throttle_threshold: 80.0_f64,
            enable_adaptive_processing: false,
            enable_bottleneck_detection: false,
            enable_caching: false,
            enable_cleanup_on_pressure: false,
            enable_concurrent_processing: false,
            enable_cpu_monitoring: false,
            enable_disk_monitoring: false,
            enable_failure_recovery: false,
            enable_graceful_degradation: false,
            enable_load_balancing: false,
            enable_memory_monitoring: false,
            enable_profiling: false,
            enable_resource_monitoring: false,
            enable_scalability_optimization: false,
            enable_speed_monitoring: false,
            max_concurrent_files: 1,
            max_disk_space_mb: 1024,
            max_memory_mb: 100,
            max_processing_threads: num_cpus::get(),
            memory_pressure_threshold: 0.8_f64,
            monitoring_interval_ms: 1000,
            profiling_granularity: "standard".to_owned(),
            worker_count: 4,
        };
    }
}

/// Configuration for monitoring and observability features
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct MonitoringConfig {
    pub enable_alerting: bool,
    pub enable_health_checks: bool,
    pub enable_metrics_export: bool,
    pub export_format: String,
    pub log_level: String,
}

impl Default for MonitoringConfig {
    #[inline]
    fn default() -> Self {
        return Self {
            enable_alerting: false,
            enable_health_checks: true,
            enable_metrics_export: false,
            export_format: "json".to_owned(),
            log_level: "info".to_owned(),
        };
    }
}
