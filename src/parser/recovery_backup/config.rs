//! Configuration validation and backoff calculation for recovery operations

use crate::error::{CpinfoError, Result};
use crate::parser::config::RetryConfig;
use std::time::Duration;

/// Configuration validator for retry operations
pub(super) struct RetryConfigValidator;

impl RetryConfigValidator {
    /// Validates retry configuration parameters
    pub fn validate(config: &RetryConfig) -> Result<()> {
        if config.max_attempts == 0 {
            return Err(CpinfoError::validation_error("max_attempts cannot be zero"));
        }
        if config.backoff_multiplier <= 0.0 {
            return Err(CpinfoError::validation_error(
                "backoff_multiplier must be positive",
            ));
        }
        if config.initial_delay > config.max_delay {
            return Err(CpinfoError::validation_error(
                "initial_delay cannot exceed max_delay",
            ));
        }
        Ok(())
    }
}

/// Calculates exponential backoff delays
pub(super) struct BackoffCalculator {
    current_delay: Duration,
    max_delay: Duration,
    multiplier: f64,
}

impl BackoffCalculator {
    /// Creates new backoff calculator with initial delay
    pub fn new(initial_delay: Duration, max_delay: Duration, multiplier: f64) -> Self {
        Self {
            current_delay: initial_delay,
            max_delay,
            multiplier,
        }
    }

    /// Calculates next delay using exponential backoff
    pub fn next_delay(&mut self) -> Duration {
        let next =
            Duration::from_millis((self.current_delay.as_millis() as f64 * self.multiplier) as u64);
        self.current_delay = std::cmp::min(next, self.max_delay);
        self.current_delay
    }

    /// Gets current delay without advancing
    pub fn current(&self) -> Duration {
        self.current_delay
    }
}