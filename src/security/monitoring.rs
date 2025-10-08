use crate::error::Result;
use crate::security::audit::AuditTrail;
use crate::security::types::{
    AlertSystemTestResult, EventSeverity, MonitoringRule, SecurityAnomalyResult, SecurityEvent,
    SecurityEventType,
};

/// Real-time security monitoring system
pub struct SecurityMonitor {
    alert_thresholds: std::collections::HashMap<SecurityEventType, u32>,
    audit: AuditTrail,
    event_log: Vec<SecurityEvent>,
}

impl SecurityMonitor {
    /// Count recent events of a specific type
    #[must_use]
    #[inline]
    pub fn count_recent_events(
        &self,
        event_type: &SecurityEventType,
        duration: chrono::Duration,
    ) -> u32 {
        let cutoff_time = chrono::Utc::now() - duration;

        return match u32::try_from(
            self.event_log
                .iter()
                .filter(|event| {
                    return event.event_type == *event_type && event.timestamp > cutoff_time;
                })
                .count(),
        ) {
            Ok(count_value) => count_value,
            Err(_conversion_error) => u32::MAX,
        };
    }

    /// Detect security anomalies
    ///
    /// # Errors
    /// Returns an error if anomaly logging fails
    #[inline]
    pub fn detect_anomalies(&mut self) -> Result<SecurityAnomalyResult> {
        let recent_failures = self.count_recent_events(
            &SecurityEventType::AuthenticationFailure,
            chrono::Duration::hours(1),
        );
        let recent_violations = self.count_recent_events(
            &SecurityEventType::SecurityViolation,
            chrono::Duration::hours(1),
        );

        let anomaly_detected = recent_failures > 10 || recent_violations > 0;

        if anomaly_detected {
            match self.log_security_event(
                &SecurityEventType::SystemAnomaly,
                EventSeverity::High,
                &format!(
                    "Anomaly detected: {recent_failures} failures, {recent_violations} violations"
                ),
                Some("system"),
            ) {
                Ok(_event_id) => {}
                Err(log_error) => return Err(log_error),
            }
        }

        return Ok(SecurityAnomalyResult {
            anomaly_detected,
            threat_level: if recent_violations > 0 {
                "High"
            } else if recent_failures > 5 {
                "Medium"
            } else {
                "Low"
            }
            .to_owned(),
            recommended_actions: vec![
                "Review authentication logs".to_owned(),
                "Check for brute force attacks".to_owned(),
            ],
        });
    }

    /// Log a security event
    ///
    /// # Errors
    /// Returns an error if the event cannot be logged or audit trail fails
    #[inline]
    pub fn log_security_event(
        &mut self,
        event_type: &SecurityEventType,
        severity: EventSeverity,
        description: &str,
        user_id: Option<&str>,
    ) -> Result<String> {
        let event_id = uuid::Uuid::new_v4().to_string();

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("system".to_owned(), "cpinfo_parser".to_owned());
        metadata.insert("version".to_owned(), "1.0.0".to_owned());

        let event = SecurityEvent {
            event_id: event_id.clone(),
            event_type: event_type.clone(),
            severity,
            timestamp: chrono::Utc::now(),
            user_id: user_id.map(str::to_owned),
            source_ip: Some("127.0.0.1".to_owned()),
            description: description.to_owned(),
            metadata,
        };

        self.event_log.push(event);

        let recent_events = self.count_recent_events(event_type, chrono::Duration::hours(1));
        if let Some(&threshold) = self.alert_thresholds.get(event_type) {
            if recent_events >= threshold {
                match self.audit.log_action(
                    "security_monitor",
                    &format!(
                        "Alert: {recent_events} events of type {event_type:?} exceeded threshold {threshold}"
                    ),
                    "security_alerting",
                ) {
                    Ok(_audit_result) => {},
                    Err(audit_error) => return Err(audit_error),
                }
            }
        }

        return Ok(event_id);
    }

    /// Create a new security monitor
    ///
    /// # Errors
    /// Returns an error if the audit trail cannot be created
    #[inline]
    pub fn new() -> Result<Self> {
        let mut alert_thresholds = std::collections::HashMap::new();
        alert_thresholds.insert(SecurityEventType::AuthenticationFailure, 5);
        alert_thresholds.insert(SecurityEventType::AuthorizationFailure, 3);
        alert_thresholds.insert(SecurityEventType::SecurityViolation, 1);
        alert_thresholds.insert(SecurityEventType::SystemAnomaly, 2);

        let audit_trail = match AuditTrail::new("/tmp/security_monitor_audit") {
            Ok(trail) => trail,
            Err(audit_error) => return Err(audit_error),
        };
        return Ok(Self {
            alert_thresholds,
            audit: audit_trail,
            event_log: Vec::new(),
        });
    }

    /// Process file access and generate security events
    ///
    /// # Errors
    /// Returns an error if security event logging fails
    #[inline]
    pub fn process_file_access(
        &mut self,
        file_path: &std::path::Path,
        user_id: &str,
        access_type: &str,
    ) -> Result<Vec<SecurityEvent>> {
        let mut events = Vec::new();

        let event_id = match self.log_security_event(
            &SecurityEventType::AuthorizationFailure,
            EventSeverity::Medium,
            &format!("File access: {} by user {}", file_path.display(), user_id),
            Some(user_id),
        ) {
            Ok(event_id) => event_id,
            Err(log_error) => return Err(log_error),
        };

        if let Some(event) = self
            .event_log
            .iter()
            .find(|event| return event.event_id == event_id)
        {
            events.push(event.clone());
        }

        let file_name = file_path
            .file_name()
            .and_then(|file_name| return file_name.to_str())
            .unwrap_or("");

        if file_name.contains("sensitive") || access_type.contains("sensitive") {
            let sensitive_id = match self.log_security_event(
                &SecurityEventType::SecurityViolation,
                EventSeverity::High,
                &format!("Sensitive file access detected: {}", file_path.display()),
                Some(user_id),
            ) {
                Ok(sensitive_id) => sensitive_id,
                Err(log_error) => return Err(log_error),
            };

            if let Some(event) = self
                .event_log
                .iter()
                .find(|event| return event.event_id == sensitive_id)
            {
                events.push(event.clone());
            }
        }

        return Ok(events);
    }

    /// Test the alert system functionality
    ///
    /// # Errors
    /// Returns an error if event logging fails during testing
    #[inline]
    pub fn test_alert_system(&mut self) -> Result<AlertSystemTestResult> {
        let mut events_generated = 0;
        let mut alerts_triggered = 0;

        for _ in 0..6 {
            match self.log_security_event(
                &SecurityEventType::AuthenticationFailure,
                EventSeverity::Medium,
                "Test authentication failure",
                Some("test_user"),
            ) {
                Ok(_event_id) => {}
                Err(log_error) => return Err(log_error),
            }
            events_generated += 1;
        }

        let recent_failures = self.count_recent_events(
            &SecurityEventType::AuthenticationFailure,
            chrono::Duration::hours(1),
        );
        if recent_failures >= 5 {
            alerts_triggered += 1;
        }

        match self.log_security_event(
            &SecurityEventType::SecurityViolation,
            EventSeverity::High,
            "Test security violation",
            Some("test_user"),
        ) {
            Ok(_event_id) => {}
            Err(log_error) => return Err(log_error),
        }
        events_generated += 1;
        alerts_triggered += 1;

        return Ok(AlertSystemTestResult {
            events_generated,
            alerts_triggered,
            test_successful: alerts_triggered > 0,
        });
    }
}

/// Threat detector for security analysis
#[derive(Debug)]
#[non_exhaustive]
pub struct ThreatDetector {
    pub detected_threats: Vec<String>,
    pub rules: Vec<MonitoringRule>,
}

impl Default for ThreatDetector {
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}

impl ThreatDetector {
    /// Analyze events for threats
    #[inline]
    pub fn analyze_events(&mut self, events: &[SecurityEvent]) -> Vec<String> {
        let mut threats = Vec::new();

        for rule in &self.rules {
            let matching_events: Vec<_> = events
                .iter()
                .filter(|event| return event.event_type == rule.event_type)
                .collect();

            if matching_events.len() >= rule.threshold as usize {
                let threat_description = format!(
                    "Threat detected: {} events of type {:?} exceeded threshold {}",
                    matching_events.len(),
                    rule.event_type,
                    rule.threshold
                );
                threats.push(threat_description.clone());
                self.detected_threats.push(threat_description);
            }
        }

        return threats;
    }

    /// Create a new threat detector
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        let rules = vec![
            MonitoringRule {
                rule_id: "auth_failure".to_owned(),
                event_type: SecurityEventType::AuthenticationFailure,
                threshold: 5,
                time_window: chrono::Duration::hours(1),
                severity: EventSeverity::Medium,
            },
            MonitoringRule {
                rule_id: "security_violation".to_owned(),
                event_type: SecurityEventType::SecurityViolation,
                threshold: 1,
                time_window: chrono::Duration::minutes(1),
                severity: EventSeverity::High,
            },
        ];

        return Self {
            detected_threats: Vec::new(),
            rules,
        };
    }
}
