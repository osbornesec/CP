//! Safe type conversion utilities for high-performance parsing systems
//!
//! This module provides zero-cost and fallible conversion utilities that replace
//! dangerous `as` casts while maintaining >400MB/s streaming performance.

use crate::error::{CpinfoError, Result};
use core::convert::TryFrom as _;

/// Converts integer to floating point for calculations (safe for all integer types)
///
/// # Examples
/// ```rust
/// let file_size: u64 = 1_048_576;
/// let size_mb = to_f64_safe(file_size) / 1_048_576.0;
/// ```
#[inline]
pub fn to_f64_safe<T: Into<f64>>(value: T) -> f64 {
    return value.into();
}

/// Safe conversion from small integers to f64 (guaranteed lossless)
#[inline]
pub fn to_f64<T>(value: T) -> f64
where
    f64: From<T>,
{
    return f64::from(value);
}

/// Safe conversion from u64 to f64 for calculations
/// Note: This may lose precision for very large values (>2^53)
#[must_use]
#[inline]
pub const fn u64_to_f64(value: u64) -> f64 {
    // Note: This may lose precision for values > 2^53
    // Allow the cast for documented precision-loss conversion
    #[expect(
        clippy::cast_precision_loss,
        reason = "u64 to f64 conversion may lose precision for values > 2^53, but acceptable for performance metrics"
    )]
    {
        return value as f64;
    }
}

/// Safe conversion from usize to f64 for calculations
/// Note: This may lose precision for very large values (>2^53)
///
/// # Errors
/// Returns an error if usize cannot fit in u64 (should never happen on current platforms)
#[inline]
pub fn usize_to_f64(value: usize) -> Result<f64> {
    // Convert via u64 for consistent handling across platforms
    let value_u64 = match u64::try_from(value) {
        Ok(converted_value) => converted_value,
        Err(_conversion_error) => {
            return Err(CpinfoError::validation_error(format!(
                "Platform error: usize value {value} cannot fit in u64"
            )));
        }
    };
    return Ok(u64_to_f64(value_u64));
}

/// Converts integer to f32 for calculations (zero-cost widening where possible)
#[inline]
pub fn to_f32<T>(value: T) -> f32
where
    f32: From<T>,
{
    return f32::from(value);
}

/// Safe conversion from usize to f32 for calculations
/// Note: This may lose precision for large values (>2^24)
///
/// # Errors
/// Returns an error if usize cannot fit in u64 (should never happen on current platforms)
#[inline]
pub fn usize_to_f32(value: usize) -> Result<f32> {
    // Note: This may lose precision for values > 2^24
    // Prefer u64_to_f64 for better precision when possible
    // Safe: documented precision loss is acceptable for this use case
    #[expect(
        clippy::cast_precision_loss,
        reason = "Documented precision loss acceptable for performance calculations"
    )]
    {
        // Convert to u64 first for consistency, then to f32
        let value_u64 = match u64::try_from(value) {
            Ok(converted_value) => converted_value,
            Err(_conversion_error) => {
                return Err(CpinfoError::validation_error(format!(
                    "Platform error: usize value {value} cannot fit in u64"
                )));
            }
        };
        return Ok(value_u64 as f32);
    }
}

/// Safely converts floating point percentage to integer percentage
///
/// # Errors
/// Returns an error if the percentage is outside the valid range [0.0, 100.0]
#[inline]
pub fn percentage_to_u32(percentage: f64) -> Result<u32> {
    if !(0.0..=100.0).contains(&percentage) {
        return Err(CpinfoError::validation_error(format!(
            "Percentage {percentage} is out of range [0.0, 100.0]"
        )));
    }
    // Safe: validated range [0.0, 100.0], round() ensures integer range
    let rounded = percentage.round();
    if rounded < 0.0 || rounded > f64::from(u32::MAX) {
        return Err(CpinfoError::validation_error(format!(
            "Rounded percentage {rounded} is out of u32 range"
        )));
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "Range validated above to ensure safe conversion"
    )]
    return Ok(rounded as u32);
}

/// Safely converts collection size to u32 for counts
///
/// # Errors
/// Returns an error if the collection size is too large to fit in u32
#[inline]
pub fn collection_len_to_u32(len: usize) -> Result<u32> {
    return u32::try_from(len).map_err(|_conversion_error| {
        return CpinfoError::validation_error(format!("Collection too large for u32: {len} items"));
    });
}

/// Safely converts collection size to u64 for byte counts
///
/// This is typically zero-cost on 64-bit platforms where usize == u64
///
/// # Errors
/// Returns an error if usize cannot fit in u64 (should never happen on current platforms)
#[inline]
pub fn collection_len_to_u64(len: usize) -> Result<u64> {
    // Safe: u64 can always hold usize values on all platforms
    return u64::try_from(len).map_err(|_conversion_error| {
        return CpinfoError::validation_error(format!(
            "Platform error: collection length {len} cannot fit in u64"
        ));
    });
}

/// Safely converts duration milliseconds to u64 for storage
///
/// # Errors
/// Returns an error if the duration is too large to fit in u64 milliseconds
#[inline]
pub fn duration_millis_to_u64(duration: core::time::Duration) -> Result<u64> {
    let millis = duration.as_millis();
    return u64::try_from(millis).map_err(|_conversion_error| {
        return CpinfoError::validation_error(format!(
            "Duration too large for u64 milliseconds: {millis} ms"
        ));
    });
}

/// Safely converts buffer size to usize for array indexing
///
/// # Errors
/// Returns an error if the buffer size is too large for the current platform
#[inline]
pub fn buffer_size_to_usize(size: u64) -> Result<usize> {
    return usize::try_from(size).map_err(|_conversion_error| {
        return CpinfoError::validation_error(format!("Buffer size {size} too large for platform"));
    });
}

/// Converts bytes to megabytes for human-readable output
#[must_use]
#[inline]
pub fn bytes_to_mb_f64(bytes: u64) -> f64 {
    // Allow floating-point arithmetic for human-readable conversion
    #[expect(
        clippy::float_arithmetic,
        reason = "Safe division by constant for unit conversion to megabytes"
    )]
    {
        return u64_to_f64(bytes) / 1_048_576.0;
    }
}

/// Safely converts bytes per second to MB/s throughput
#[must_use]
#[inline]
pub fn bytes_to_mbps(bytes_per_sec: u64) -> f64 {
    // Allow floating-point arithmetic for throughput calculation
    #[expect(
        clippy::float_arithmetic,
        reason = "Safe division by constant for MB/s conversion"
    )]
    {
        return u64_to_f64(bytes_per_sec) / 1_000_000.0;
    }
}

/// Calculates throughput maintaining precision for performance metrics
#[must_use]
#[inline]
pub fn calculate_throughput_mbps(bytes_processed: u64, duration: core::time::Duration) -> f64 {
    if duration.is_zero() {
        return 0.0;
    }

    let bytes_f64 = u64_to_f64(bytes_processed);
    let duration_secs = duration.as_secs_f64();
    // Allow floating-point arithmetic for throughput calculation
    #[expect(
        clippy::float_arithmetic,
        reason = "Safe division for throughput calculation with validated inputs"
    )]
    {
        return bytes_f64 / duration_secs / 1_000_000.0;
    }
}

/// Calculates processing rate (items per second)
#[must_use]
#[inline]
pub fn calculate_rate(items: usize, duration: core::time::Duration) -> f64 {
    if duration.is_zero() {
        return 0.0;
    }

    // Allow floating-point arithmetic for rate calculation
    #[expect(
        clippy::float_arithmetic,
        reason = "Safe division for rate calculation with validated inputs"
    )]
    {
        let items_f64 = match usize_to_f64(items) {
            Ok(converted_items) => converted_items,
            Err(_conversion_error) => {
                // Fallback for platform error - should never happen
                return 0.0;
            }
        };
        return items_f64 / duration.as_secs_f64();
    }
}

/// Safely converts exponential backoff multiplier
///
/// # Errors
/// Returns an error if the multiplier is invalid or would cause overflow
#[inline]
pub fn apply_backoff_multiplier(current_delay_ms: u64, multiplier: f64) -> Result<u64> {
    if multiplier <= 0.0 || multiplier > 100.0 {
        return Err(CpinfoError::validation_error(format!(
            "Invalid backoff multiplier: {multiplier}"
        )));
    }

    // Allow floating-point arithmetic for backoff calculation
    #[expect(
        clippy::float_arithmetic,
        reason = "Safe multiplication and rounding for backoff delay calculation"
    )]
    let new_delay = (u64_to_f64(current_delay_ms) * multiplier).round();

    if new_delay > u64_to_f64(u64::MAX) {
        return Err(CpinfoError::validation_error(
            "Backoff delay would overflow u64".to_owned(),
        ));
    }

    // Safe: validated above to be within u64 range
    if new_delay < 0.0 {
        return Err(CpinfoError::validation_error(
            "Backoff delay cannot be negative".to_owned(),
        ));
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "Value validated above to be within u64 range and non-negative"
    )]
    return Ok(new_delay as u64);
}
