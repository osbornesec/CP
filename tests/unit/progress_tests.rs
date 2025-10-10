//\! Tests for progress formatting utilities

use std::time::Duration;
use cpinfo_parser::progress::format_duration;

#[test]
fn format_duration_human_readable() {
    assert_eq\!(format_duration(Duration::from_secs(30)), "30s");
    assert_eq\!(format_duration(Duration::from_secs(90)), "1m30s");
    assert_eq\!(format_duration(Duration::from_secs(3661)), "1h1m");
}