//\! Unit tests for security::event_logger module

use cpinfo_parser::security::event_logger::SecurityEventLogger;
use cpinfo_parser::security::types::{EventSeverity, SecurityEventType};
use tempfile::tempdir;

#[test]
fn test_event_logging_basic() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let audit_path = temp_dir.path().join("test_audit").to_string_lossy().to_string();

    let mut logger = SecurityEventLogger::new(&audit_path)?;

    let event_id = logger.log_event(
        SecurityEventType::AuthenticationFailure,
        EventSeverity::Medium,
        "Test authentication failure",
        Some("test_user"),
    )?;

    assert\!(\!event_id.is_empty());
    assert_eq\!(logger.get_events().len(), 1);

    Ok(())
}

#[test]
fn test_event_counting() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let audit_path = temp_dir.path().join("test_audit").to_string_lossy().to_string();

    let mut logger = SecurityEventLogger::new(&audit_path)?;

    // Log multiple events
    for _ in 0..3 {
        logger.log_event(
            SecurityEventType::AuthenticationFailure,
            EventSeverity::Medium,
            "Test failure",
            Some("test_user"),
        )?;
    }

    let count = logger.count_recent_events(
        &SecurityEventType::AuthenticationFailure,
        chrono::Duration::hours(1),
    );

    assert_eq\!(count, 3);

    Ok(())
}

#[test]
fn test_event_counting_different_types() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let audit_path = temp_dir.path().join("test_audit").to_string_lossy().to_string();

    let mut logger = SecurityEventLogger::new(&audit_path)?;

    logger.log_event(
        SecurityEventType::AuthenticationFailure,
        EventSeverity::Medium,
        "Auth failure",
        Some("user1"),
    )?;

    logger.log_event(
        SecurityEventType::UnauthorizedAccess,
        EventSeverity::High,
        "Unauthorized access",
        Some("user2"),
    )?;

    let auth_count = logger.count_recent_events(
        &SecurityEventType::AuthenticationFailure,
        chrono::Duration::hours(1),
    );
    
    let unauth_count = logger.count_recent_events(
        &SecurityEventType::UnauthorizedAccess,
        chrono::Duration::hours(1),
    );

    assert_eq\!(auth_count, 1);
    assert_eq\!(unauth_count, 1);

    Ok(())
}

#[test]
fn test_find_events_by_criteria() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let audit_path = temp_dir.path().join("test_audit").to_string_lossy().to_string();

    let mut logger = SecurityEventLogger::new(&audit_path)?;

    logger.log_event(
        SecurityEventType::AuthenticationFailure,
        EventSeverity::Medium,
        "Failure for user1",
        Some("user1"),
    )?;

    logger.log_event(
        SecurityEventType::AuthenticationFailure,
        EventSeverity::Medium,
        "Failure for user2",
        Some("user2"),
    )?;

    let user1_events = logger.find_events_by_criteria(
        Some(&SecurityEventType::AuthenticationFailure),
        Some("user1"),
        None,
    );

    assert_eq\!(user1_events.len(), 1);

    Ok(())
}

#[test]
fn test_get_all_events() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let audit_path = temp_dir.path().join("test_audit").to_string_lossy().to_string();

    let mut logger = SecurityEventLogger::new(&audit_path)?;

    for i in 0..5 {
        logger.log_event(
            SecurityEventType::AuthenticationFailure,
            EventSeverity::Medium,
            &format\!("Event {}", i),
            Some("test_user"),
        )?;
    }

    let all_events = logger.get_events();
    assert_eq\!(all_events.len(), 5);

    Ok(())
}

#[test]
fn test_event_logger_empty() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let audit_path = temp_dir.path().join("test_audit").to_string_lossy().to_string();

    let logger = SecurityEventLogger::new(&audit_path)?;

    assert_eq\!(logger.get_events().len(), 0);

    let count = logger.count_recent_events(
        &SecurityEventType::AuthenticationFailure,
        chrono::Duration::hours(1),
    );
    assert_eq\!(count, 0);

    Ok(())
}