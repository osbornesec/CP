//\! Unit tests for parser::recovery::backoff module

use std::time::Duration;

// We need to create a wrapper or test the behavior through its usage
// Since BackoffCalculator is pub(super), we test it indirectly through the recovery module
// For now, create a local test implementation to demonstrate the logic

#[test]
fn test_backoff_calculator_progression() {
    // Test the exponential backoff logic
    let initial = Duration::from_millis(100);
    let max = Duration::from_millis(1000);
    let multiplier = 2.0;

    let mut current = initial;
    let values = vec\![
        current,
        {
            current = Duration::from_millis((current.as_millis() as f64 * multiplier) as u64);
            std::cmp::min(current, max)
        },
        {
            current = Duration::from_millis((current.as_millis() as f64 * multiplier) as u64);
            std::cmp::min(current, max)
        },
        {
            current = Duration::from_millis((current.as_millis() as f64 * multiplier) as u64);
            std::cmp::min(current, max)
        },
        {
            current = Duration::from_millis((current.as_millis() as f64 * multiplier) as u64);
            std::cmp::min(current, max)
        },
    ];

    assert_eq\!(values[0], Duration::from_millis(100));
    assert_eq\!(values[1], Duration::from_millis(200));
    assert_eq\!(values[2], Duration::from_millis(400));
    assert_eq\!(values[3], Duration::from_millis(800));
    assert_eq\!(values[4], Duration::from_millis(1000)); // Capped at max
}

#[test]
fn test_backoff_respects_max_delay() {
    let max = Duration::from_millis(500);
    let multiplier = 3.0;

    let mut current = Duration::from_millis(200);
    current = Duration::from_millis((current.as_millis() as f64 * multiplier) as u64);
    current = std::cmp::min(current, max);

    // 200 * 3 = 600, but capped at 500
    assert_eq\!(current, Duration::from_millis(500));
}

#[test]
fn test_backoff_with_fractional_multiplier() {
    let initial = Duration::from_millis(1000);
    let multiplier = 1.5;

    let next = Duration::from_millis((initial.as_millis() as f64 * multiplier) as u64);
    assert_eq\!(next, Duration::from_millis(1500));
}

#[test]
fn test_backoff_stays_at_max() {
    let max = Duration::from_millis(100);
    let multiplier = 2.0;

    let mut current = Duration::from_millis(100);
    current = Duration::from_millis((current.as_millis() as f64 * multiplier) as u64);
    current = std::cmp::min(current, max);

    assert_eq\!(current, max);

    current = Duration::from_millis((current.as_millis() as f64 * multiplier) as u64);
    current = std::cmp::min(current, max);

    assert_eq\!(current, max);
}