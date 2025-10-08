#![allow(
    clippy::pub_use,
    reason = "Library design requires re-exporting types for clean public API"
)]
#![allow(
    clippy::blanket_clippy_restriction_lints,
    reason = "Strict quality standards maintained via selective lint enforcement"
)]

pub mod checkpoint;
pub mod cli;
pub mod error;
pub mod extraction;
pub mod format;
pub mod integrated_workflow;
pub mod output;
pub mod parser;
pub mod progress;
pub mod section;
pub mod section_parser;
pub mod security;
pub mod utils;
pub mod validation;
pub mod workflow;

pub use error::{CpinfoError, Result};
pub use extraction::{
    BinaryDetectionResult, ExtractionResult, OrganizedExtractionResult, PartialRecoveryResult,
    SectionExtractor,
};
pub use format::{CpinfoFormat, FormatDetector};

pub use workflow::{
    IntegratedWorkflowConfig, IntegratedWorkflowOrchestrator, IntegratedWorkflowResult, PhaseStats,
};

pub use parser::{
    CacheStats, CheckpointRecoveryResult, CheckpointingResult, ConcurrentStats,
    ConnectionPoolingResult, CpinfoParser, CpuThrottleResult, DiagnosticInfo, DiskConstraintResult,
    EnterpriseStats, HealthStatus, LoadBalanceStats, MemoryStats, MonitoringConfig,
    MonitoringResult, NetworkConfig, NetworkResult, NodeRecoveryResult, ParseResult,
    PartialRecoveryConfig, PerformanceBottleneck, PerformanceConfig, PerformanceDiagnostics,
    ProfilingStats, RecoveryStrategy, ReliabilityStats, ResourceConstraintResult, ResourceStats,
    RetryConfig, RetryResult, SectionRecoveryResult, SpeedStats, SystemInfo,
};
pub use progress::{AccessibilityConfig, ProgressReporter};
pub use section::{DelimiterDetector, SectionDelimiter};

pub use section_parser::{
    CommandSection, FileSection, SectionDelimiterDetector, SectionDelimiterType, SectionFileParser,
};
pub use security::{
    AccessAuthorization, AlertSystemTestResult, AuditEntry, AuditEvent, AuditLevel, AuditTrail,
    AuthenticationManager, AuthenticationResult, ClassificationLevel, ClassificationResult,
    ClassifiedSection, ComplianceEvidence, ComplianceFramework, ComplianceManager,
    ComplianceReport, ConfigValidationResult, ConfigurationLevel, ConfigurationManager,
    ContainmentAction, ContainmentResult, DataClassifier, EmergencyAccess, EncryptedFileInfo,
    EncryptionResult, EventSeverity, ExportSummary, FileEncryption, GdprRequestResult, GdprRight,
    IncidentManager, IncidentResponseTest, IncidentSeverity, IncidentType, MonitoringRule,
    PermissionSet, PiiClassification, PrivacyManager, RoleBasedAccess, RoleHierarchyResult,
    RoleManager, SecureConfig, SecurityAnomalyResult, SecurityEvent, SecurityEventType,
    SecurityIncident, SecurityMonitor, SensitiveDataFilter, SensitiveMatch, Soc2Criteria,
    TamperReport, ThreatDetector, UserRole, UserSession,
};
pub use utils::safe_mutex_lock;
pub use validation::FileValidator;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default buffer size for streaming operations (64KB)
pub const DEFAULT_BUFFER_SIZE: usize = 64 * 1024;

/// Maximum section name length to prevent excessively long filenames
pub const MAX_SECTION_NAME_LENGTH: usize = 255;

/// Section delimiter pattern used in cpinfo files
pub const SECTION_DELIMITER: &str = "==============================================";
