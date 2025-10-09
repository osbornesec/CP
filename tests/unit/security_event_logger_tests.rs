//\! Minimal tests for security event logger

use tempfile::tempdir;
use cpinfo_parser::security::{SecurityEventLogger, SecurityEventType, EventSeverity};

#[test]
fn log_and_query_events() {
    let dir = tempdir().unwrap();
    let audit_path = dir.path().join("audit");
    // Use directory path as string target
    let mut logger = SecurityEventLogger::new(audit_path.to_string_lossy().as_ref())
        .expect("logger");

    let event_id = logger
        .log_event(
            SecurityEventType::AuthenticationFailure,
            EventSeverity::Medium,
            "Test authentication failure",
            Some("test_user"),
        )
        .expect("log");
    assert\!(\!event_id.is_empty());

    // Count recently logged events within 1 hour
    let count = logger.count_recent_events(
        &SecurityEventType::AuthenticationFailure,
        chrono::Duration::hours(1),
    );
    assert_eq\!(count, 1);

    // Query by criteria
    let matches = logger.find_events_by_criteria(
        Some(&SecurityEventType::AuthenticationFailure),
        Some("test_user"),
        None,
    );
    assert_eq\!(matches.len(), 1);
}