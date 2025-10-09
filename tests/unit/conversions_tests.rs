//\! Tests for numeric and time conversion utilities

use std::time::Duration;
use cpinfo_parser::utils::conversions::*;

#[test]
fn percentage_to_u32_ok_and_rounding() {
    assert_eq\!(percentage_to_u32(0.0).unwrap(), 0);
    assert_eq\!(percentage_to_u32(25.4).unwrap(), 25);
    assert_eq\!(percentage_to_u32(25.5).unwrap(), 26);
    assert_eq\!(percentage_to_u32(100.0).unwrap(), 100);
    assert\!(percentage_to_u32(-1.0).is_err());
    assert\!(percentage_to_u32(101.0).is_err());
}

#[test]
fn bytes_conversion_helpers() {
    let mb = bytes_to_mb_f64(1_048_576); // 1 MiB
    assert\!((mb - 1.0).abs() < 1e-9);

    let mbps = bytes_to_mbps(1_048_576);
    assert\!((mbps - 1.0).abs() < 1e-9);
}

#[test]
fn duration_and_buffer_conversions() {
    assert_eq\!(duration_millis_to_u64(Duration::from_millis(1500)).unwrap(), 1500);
    assert_eq\!(buffer_size_to_usize(64).unwrap(), 64usize);
}

#[test]
fn length_conversions() {
    assert_eq\!(collection_len_to_u32(5).unwrap(), 5);
    assert_eq\!(collection_len_to_u64(7).unwrap(), 7);
}