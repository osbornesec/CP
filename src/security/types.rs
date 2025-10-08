//! Security monitoring types and data structures

use std::collections::HashMap;

/// Security event types that can occur in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum SecurityEventType {
    /// Failed authentication attempt
    AuthenticationFailure,
    /// Successful authentication
    AuthenticationSuccess,
    /// Authorization failure (access denied)
    AuthorizationFailure,
    /// Configuration change event
    ConfigurationChange,
    /// Data access event
    DataAccess,
    /// Data modification event
    DataModification,
    /// Security policy violation
    SecurityViolation,
    /// System anomaly detected
    SystemAnomaly,
}

/// Severity levels for security events
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum EventSeverity {
    /// Critical severity event requiring immediate attention
    Critical,
    /// High severity event
    High,
    /// Low severity event
    Low,
    /// Medium severity event
    Medium,
}

/// A security event with all relevant metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct SecurityEvent {
    /// Human-readable description of the event
    pub description: String,
    /// Unique identifier for this event
    pub event_id: String,
    /// Type of security event
    pub event_type: SecurityEventType,
    /// Additional metadata key-value pairs
    pub metadata: HashMap<String, String>,
    /// Severity level of the event
    pub severity: EventSeverity,
    /// Source IP address (if applicable)
    pub source_ip: Option<String>,
    /// Timestamp when the event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// User ID associated with the event (if applicable)
    pub user_id: Option<String>,
}

/// Result of security anomaly detection
#[derive(Debug)]
#[non_exhaustive]
pub struct SecurityAnomalyResult {
    /// Whether an anomaly was detected
    pub anomaly_detected: bool,
    /// Recommended actions to take
    pub recommended_actions: Vec<String>,
    /// Threat level assessment
    pub threat_level: String,
}

/// Result of alert system testing
#[derive(Debug)]
#[non_exhaustive]
pub struct AlertSystemTestResult {
    /// Number of alerts triggered during test
    pub alerts_triggered: usize,
    /// Number of events generated during test
    pub events_generated: usize,
    /// Whether the test was successful
    pub test_successful: bool,
}

/// A monitoring rule configuration
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct MonitoringRule {
    /// Type of events this rule monitors
    pub event_type: SecurityEventType,
    /// Unique identifier for this rule
    pub rule_id: String,
    /// Severity level for triggered alerts
    pub severity: EventSeverity,
    /// Threshold count that triggers the rule
    pub threshold: u32,
    /// Time window for counting events
    pub time_window: chrono::Duration,
}
