//! Parser module providing high-performance `cpinfo` file parsing capabilities.
//!
//! This module offers a comprehensive suite of parsing tools including:
//! - Memory-mapped file parsing for large files
//! - Concurrent processing for multiple files
//! - Binary content detection
//! - Memory and performance monitoring
//! - Section extraction and organization

pub mod binary_extraction;
pub mod concurrency;
pub mod config;
pub mod core;
pub mod facade;
pub mod monitoring;
pub mod network;
pub mod recovery;
pub mod stats;
pub mod utils;

// Re-export configuration types
pub use config::{
    MonitoringConfig, NetworkConfig, PartialRecoveryConfig, PerformanceConfig, RecoveryStrategy,
    RetryConfig,
};

// Re-export statistics and result types
pub use stats::{
    CacheStats, CheckpointRecoveryResult, CheckpointingResult, ConcurrentStats,
    ConnectionPoolingResult, CpuThrottleResult, DiagnosticInfo, DiskConstraintResult,
    EnterpriseStats, HealthStatus, LoadBalanceStats, MemoryStats, MonitoringResult, NetworkResult,
    NodeRecoveryResult, ParseResult, PerformanceBottleneck, PerformanceDiagnostics, ProfilingStats,
    ReliabilityStats, ResourceConstraintResult, ResourceStats, RetryResult, SectionRecoveryResult,
    SpeedStats, SystemInfo,
};

// Re-export the main parser from facade
pub use facade::CpinfoParser;
