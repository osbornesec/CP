//! Exponential backoff calculation for retry operations
//!
//! This module provides precise backoff timing calculations following
//! exponential backoff patterns with configurable parameters.

use std::time::Duration;

/// Calculates exponential backoff delays with configurable parameters
///
/// # Performance
/// Constant memory usage with O(1) calculations per retry attempt
pub(super) struct BackoffCalculator {
    current_delay: Duration,
    max_delay: Duration,
    multiplier: f64,
}

impl BackoffCalculator {
    /// Creates new backoff calculator with initial parameters
    ///
    /// # Arguments
    /// * `initial_delay` - Starting delay duration
    /// * `max_delay` - Maximum allowed delay duration
    /// * `multiplier` - Exponential growth factor (must be > 1.0)
    ///
    /// # Returns
    /// Configured backoff calculator ready for use
    pub(super) fn new(initial_delay: Duration, max_delay: Duration, multiplier: f64) -> Self {
        Self {
            current_delay: initial_delay,
            max_delay,
            multiplier,
        }
    }

    /// Calculates and advances to next delay using exponential backoff
    ///
    /// # Returns
    /// Next delay duration, capped at maximum configured delay
    ///
    /// # Performance
    /// O(1) calculation with saturating arithmetic to prevent overflow
    pub(super) fn next_delay(&mut self) -> Duration {
        let next = Duration::from_millis(
            (self.current_delay.as_millis() as f64 * self.multiplier) as u64
        );
        self.current_delay = std::cmp::min(next, self.max_delay);
        self.current_delay
    }

    /// Gets current delay without advancing the calculator
    ///
    /// # Returns
    /// Current delay duration
    pub(super) fn current(&self) -> Duration {
        self.current_delay
    }
}

