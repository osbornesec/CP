//! Security event logging functionality

use crate::error::Result;
use crate::security::audit::AuditTrail;
use crate::security::types::{EventSeverity, SecurityEvent, SecurityEventType};
use std::collections::HashMap;

/// Handles logging and persistence of security events
pub struct SecurityEventLogger {
    /// Audit trail for persistent logging
    audit: AuditTrail,
    /// In-memory event log storage
    event_log: Vec<SecurityEvent>,
}

impl SecurityEventLogger {
    /// Count recent events of a specific type within a time window
    #[must_use]
    #[inline]
    pub fn count_recent_events(
        &self,
        event_type: &SecurityEventType,
        duration: chrono::Duration,
    ) -> u32 {
        let cutoff_time = chrono::Utc::now() - duration;

        let count = self
            .event_log
            .iter()
            .filter(|event| {
                return event.event_type == *event_type && event.timestamp > cutoff_time;
            })
            .count();

        #[allow(
            clippy::cast_possible_truncation,
            reason = "count is bounded by event log size, truncation is acceptable"
        )]
        return count as u32;
    }

    /// Find events matching specific criteria
    #[must_use]
    #[inline]
    pub fn find_events_by_criteria(
        &self,
        event_type: Option<&SecurityEventType>,
        user_id: Option<&str>,
        since: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Vec<&SecurityEvent> {
        return self
            .event_log
            .iter()
            .filter(|event| {
                if let Some(event_type_filter) = event_type {
                    if event.event_type != *event_type_filter {
                        return false;
                    }
                }

                if let Some(user_id_filter) = user_id {
                    if event.user_id.as_deref() != Some(user_id_filter) {
                        return false;
                    }
                }

                if let Some(since_time) = since {
                    if event.timestamp < since_time {
                        return false;
                    }
                }

                return true;
            })
            .collect();
    }

    /// Get all events from the log
    #[must_use]
    #[inline]
    pub fn get_events(&self) -> &[SecurityEvent] {
        return &self.event_log;
    }

    /// Log an alert to the audit trail
    ///
    /// # Errors
    ///
    /// Returns an error if the alert cannot be logged to the audit trail
    #[inline]
    pub fn log_alert(&mut self, alert_message: &str) -> Result<()> {
        return self
            .audit
            .log_action("security_monitor", alert_message, "security_alerting")
            .map(drop);
    }

    /// Log a security event and return the event ID
    ///
    /// # Errors
    ///
    /// Returns an error if the event cannot be logged
    #[inline]
    pub fn log_event(
        &mut self,
        event_type: SecurityEventType,
        severity: EventSeverity,
        description: &str,
        user_id: Option<&str>,
    ) -> Result<String> {
        let event_id = uuid::Uuid::new_v4().to_string();
        let mut metadata = HashMap::new();
        metadata.insert("system".to_owned(), "cpinfo_parser".to_owned());
        metadata.insert("version".to_owned(), "1.0.0".to_owned());

        let event = SecurityEvent {
            description: description.to_owned(),
            event_id: event_id.clone(),
            event_type,
            metadata,
            severity,
            source_ip: Some("127.0.0.1".to_owned()),
            timestamp: chrono::Utc::now(),
            user_id: user_id.map(String::from),
        };

        self.store_event(event);
        return Ok(event_id);
    }

    /// Create a new security event logger
    ///
    /// # Errors
    ///
    /// Returns an error if the audit trail cannot be initialized
    #[inline]
    pub fn new(audit_path: &str) -> Result<Self> {
        let audit_trail = match AuditTrail::new(audit_path) {
            Ok(audit) => audit,
            Err(error) => return Err(error),
        };
        return Ok(Self {
            audit: audit_trail,
            event_log: Vec::new(),
        });
    }

    /// Store an event in the log
    fn store_event(&mut self, event: SecurityEvent) {
        self.event_log.push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_event_logging() -> Result<()> {
        let temp_dir = tempdir()?;
        let audit_path = temp_dir
            .path()
            .join("test_audit")
            .to_string_lossy()
            .to_string();

        let mut logger = match SecurityEventLogger::new(&audit_path) {
            Ok(l) => l,
            Err(e) => return Err(e),
        };

        let event_id = match logger.log_event(
            SecurityEventType::AuthenticationFailure,
            EventSeverity::Medium,
            "Test authentication failure",
            Some("test_user"),
        ) {
            Ok(id) => id,
            Err(e) => return Err(e),
        };

        assert!(!event_id.is_empty());
        assert_eq!(logger.get_events().len(), 1);

        Ok(())
    }

    #[test]
    fn test_event_counting() -> Result<()> {
        let temp_dir = tempdir()?;
        let audit_path = temp_dir
            .path()
            .join("test_audit")
            .to_string_lossy()
            .to_string();

        let mut logger = match SecurityEventLogger::new(&audit_path) {
            Ok(l) => l,
            Err(e) => return Err(e),
        };

        // Log multiple events
        for _ in 0..3 {
            match logger.log_event(
                SecurityEventType::AuthenticationFailure,
                EventSeverity::Medium,
                "Test failure",
                Some("test_user"),
            ) {
                Ok(_) => {}
                Err(e) => return Err(e),
            };
        }

        let count = logger.count_recent_events(
            &SecurityEventType::AuthenticationFailure,
            chrono::Duration::hours(1),
        );

        assert_eq!(count, 3);

        Ok(())
    }
}
