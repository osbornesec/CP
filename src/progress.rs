//! Progress reporting module
//!
//! Provides accessibility-compliant progress reporting for `Check Point` `cpinfo` file processing.
//! Implements `WCAG 2.1 AA` compliance with screen reader support and color-blind accessibility.

use core::time::Duration;
use std::time::Instant;
use tracing::{debug, info};

/// Display options for progress reporting
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DisplayOptions {
    /// Whether to include percentage information in updates
    pub include_percentage: bool,
    /// Whether to include time estimates in updates
    pub include_time_estimates: bool,
}

/// Accessibility mode for progress reporting
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum AccessibilityMode {
    /// No color mode (color-blind friendly)
    NoColor,
    /// Screen reader mode with verbose descriptions
    ScreenReader,
    /// Standard mode with colors and compact display
    Standard,
}

/// Configuration for progress reporting accessibility features
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct AccessibilityConfig {
    /// Display options for additional information
    pub display_options: DisplayOptions,
    /// Maximum update frequency (Hz) to prevent screen reader overload
    pub max_update_frequency: f64,
    /// Accessibility mode for the display
    pub mode: AccessibilityMode,
}

impl Default for DisplayOptions {
    #[inline]
    fn default() -> Self {
        return Self {
            include_percentage: true,
            include_time_estimates: true,
        };
    }
}

impl Default for AccessibilityConfig {
    #[inline]
    fn default() -> Self {
        let mode = if std::env::var("SCREENREADER").is_ok() {
            AccessibilityMode::ScreenReader
        } else if std::env::var("NO_COLOR").is_ok()
            || std::env::var("TERM").unwrap_or_default() == "dumb"
        {
            AccessibilityMode::NoColor
        } else {
            AccessibilityMode::Standard
        };

        return Self {
            display_options: DisplayOptions::default(),
            max_update_frequency: 5.0_f64, // Max 5 updates per second for accessibility
            mode,
        };
    }
}

/// Progress reporter for long-running operations with accessibility compliance
#[non_exhaustive]
pub struct ProgressReporter {
    /// Accessibility configuration
    config: AccessibilityConfig,
    /// Current progress position
    current: u64,
    /// Current operation description
    current_operation: Option<String>,
    /// Last update time for rate limiting
    last_update: Option<Instant>,
    /// Start time for duration calculations
    start_time: Option<Instant>,
    /// Total work to be done
    total: Option<u64>,
}

impl ProgressReporter {
    /// Get estimated time remaining
    #[must_use]
    #[inline]
    pub fn estimated_time_remaining(&self) -> Option<Duration> {
        if let Some(start_time) = self.start_time {
            let elapsed = Instant::now().duration_since(start_time);
            let fraction = self.fraction();

            if fraction > 0.0_f64 && fraction < 1.0_f64 {
                // Use integer arithmetic to avoid floating-point restriction
                let elapsed_millis = elapsed.as_millis();
                if let Some(total_items) = self.total {
                    if self.current > 0_u64 && total_items > 0_u64 {
                        let progress_ratio = elapsed_millis
                            .checked_div(u128::from(self.current))
                            .unwrap_or(0_u128);
                        let estimated_total_millis = progress_ratio
                            .checked_mul(u128::from(total_items))
                            .unwrap_or(0_u128);
                        let remaining_millis =
                            estimated_total_millis.saturating_sub(elapsed_millis);
                        return Some(Duration::from_millis(
                            u64::try_from(remaining_millis).unwrap_or(0_u64),
                        ));
                    }
                }
            }
        }
        return None;
    }

    /// Finish progress reporting
    #[inline]
    pub fn finish(&mut self, success_message: Option<&str>) {
        if let Some(start_time) = self.start_time {
            let duration = Instant::now().duration_since(start_time);

            let message = success_message.unwrap_or("Operation completed");

            if matches!(self.config.mode, AccessibilityMode::ScreenReader) {
                info!("Completed: {}", message);
                info!("Total time: {:.2}s", duration.as_secs_f64());
                if let Some(total_items) = self.total {
                    info!("Processed {} of {} items", self.current, total_items);
                }
            } else {
                info!("{} in {:.2}s", message, duration.as_secs_f64());
            }
        }
    }

    /// Get current progress as a fraction (0.0 to 1.0)
    #[must_use]
    #[inline]
    #[allow(
        clippy::float_arithmetic,
        reason = "Progress calculations require floating-point arithmetic for fraction representation"
    )]
    pub fn fraction(&self) -> f64 {
        return match self.total {
            Some(total_items) if total_items > 0_u64 => {
                // Use integer division scaled by 1000 for precision, then convert to f64
                let progress_thousandths = (self.current.saturating_mul(1000_u64))
                    .checked_div(total_items)
                    .unwrap_or(0_u64);
                // Convert to f64 and scale back to [0.0, 1.0] range without division
                let clamped_thousandths =
                    u32::try_from(progress_thousandths.min(1000_u64)).unwrap_or(0_u32);
                f64::from(clamped_thousandths) * 0.001_f64
            }
            _ => return 0.0_f64,
        };
    }

    /// Increment progress by specified amount
    #[inline]
    pub fn increment(&mut self, amount: u64) {
        self.update(self.current + amount);
    }

    /// Check if progress reporting is enabled
    #[must_use]
    #[inline]
    pub const fn is_enabled(&self) -> bool {
        // Progress is always enabled, but format depends on accessibility settings
        return true;
    }

    /// Create new progress reporter with default accessibility settings
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        return Self {
            config: AccessibilityConfig::default(),
            current: 0_u64,
            current_operation: None,
            last_update: None,
            start_time: None,
            total: None,
        };
    }

    /// Internal method to report current progress
    #[inline]
    fn report_progress(&self) {
        let operation = self.current_operation.as_deref().unwrap_or("Processing");

        if matches!(self.config.mode, AccessibilityMode::ScreenReader) {
            // Screen reader friendly format
            if let Some(total_items) = self.total {
                let mut message = format!(
                    "Progress: {current} of {total_items} items",
                    current = self.current
                );

                if self.config.display_options.include_percentage {
                    // Safe conversion: fraction is always 0.0-1.0, so percentage is 0.0-100.0
                    // Calculate percentage using integer arithmetic to avoid floating-point operations
                    let percentage = self.total.map_or(0_u32, |total_count| {
                        if total_count > 0_u64 {
                            return u32::try_from(
                                (self.current.saturating_mul(100_u64))
                                    .checked_div(total_count)
                                    .unwrap_or(0_u64),
                            )
                            .unwrap_or(0_u32);
                        } else {
                            return 0_u32;
                        }
                    });
                    let percentage_str = format!(" ({percentage}%)");
                    message.push_str(&percentage_str);
                }

                if self.config.display_options.include_time_estimates {
                    if let Some(remaining_time) = self.estimated_time_remaining() {
                        let duration_str = format_duration(remaining_time);
                        let remaining_str = format!(" - {duration_str} remaining");
                        message.push_str(&remaining_str);
                    }
                }

                info!("{}: {}", operation, message);
            } else {
                info!("{}: {} items processed", operation, self.current);
            }
        } else {
            // Standard format
            if let Some(total_items) = self.total {
                // Safe conversion: fraction is always 0.0-1.0, so percentage is 0.0-100.0
                // Calculate percentage using integer arithmetic to avoid floating-point operations
                let percentage = self.total.map_or(0_u32, |total_count| {
                    if total_count > 0_u64 {
                        return u32::try_from(
                            (self.current.saturating_mul(100_u64))
                                .checked_div(total_count)
                                .unwrap_or(0_u64),
                        )
                        .unwrap_or(0_u32);
                    } else {
                        return 0_u32;
                    }
                });
                debug!(
                    "Progress: {}/{} ({}%)",
                    self.current, total_items, percentage
                );
            } else {
                debug!("Progress: {} items processed", self.current);
            }
        }
    }

    /// Set current operation description
    #[inline]
    pub fn set_operation(&mut self, operation: &str) {
        self.current_operation = Some(operation.to_owned());
        if matches!(self.config.mode, AccessibilityMode::ScreenReader) {
            info!("Current operation: {}", operation);
        }
    }

    /// Start progress tracking with optional total work count
    #[inline]
    pub fn start(&mut self, operation: &str, total: Option<u64>) {
        self.start_time = Some(Instant::now());
        self.last_update = None;
        self.total = total;
        self.current = 0_u64;
        self.current_operation = Some(operation.to_owned());

        // Initial status message for screen readers
        if matches!(self.config.mode, AccessibilityMode::ScreenReader) {
            info!("Started: {}", operation);
            if let Some(total_items) = total {
                info!("Total work: {} items", total_items);
            }
        } else {
            info!("Starting {}", operation);
        }
    }

    /// Update progress with current position
    #[inline]
    pub fn update(&mut self, current: u64) {
        self.current = current;

        // Rate limiting for accessibility
        let now = Instant::now();
        if let Some(last_update_time) = self.last_update {
            let elapsed = now.duration_since(last_update_time).as_secs_f64();
            let min_interval = self.config.max_update_frequency.recip();
            if elapsed < min_interval {
                return; // Skip update to prevent screen reader overload
            }
        }

        self.last_update = Some(now);
        self.report_progress();
    }

    /// Create progress reporter with custom accessibility configuration
    #[must_use]
    #[inline]
    pub const fn with_config(config: AccessibilityConfig) -> Self {
        return Self {
            config,
            current: 0_u64,
            current_operation: None,
            last_update: None,
            start_time: None,
            total: None,
        };
    }
}

impl Default for ProgressReporter {
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}

/// Format duration in human-readable format
#[must_use]
#[inline]
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();

    if total_seconds < 60_u64 {
        return format!("{total_seconds}s");
    }
    if total_seconds < 3600_u64 {
        let minutes = total_seconds.checked_div(60_u64).unwrap_or(0_u64);
        let seconds = total_seconds.checked_rem(60_u64).unwrap_or(0_u64);
        return format!("{minutes}m{seconds}s");
    }
    let hours = total_seconds.checked_div(3600_u64).unwrap_or(0_u64);
    let remaining_seconds = total_seconds.checked_rem(3600_u64).unwrap_or(0_u64);
    let minutes = remaining_seconds.checked_div(60_u64).unwrap_or(0_u64);
    return format!("{hours}h{minutes}m");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_reporter_basic_functionality() {
        let mut reporter = ProgressReporter::new();

        reporter.start("Test operation", Some(100));
        assert_eq!(reporter.current, 0);
        assert_eq!(reporter.total, Some(100));

        reporter.update(50);
        assert_eq!(reporter.current, 50);
        assert_eq!(reporter.fraction(), 0.5);

        reporter.increment(25);
        assert_eq!(reporter.current, 75);
        assert_eq!(reporter.fraction(), 0.75);

        reporter.finish(Some("Test completed"));
    }

    #[test]
    fn test_accessibility_config_from_environment() {
        // Test default behavior
        let config = AccessibilityConfig::default();
        assert!(config.display_options.include_percentage);
        assert!(config.display_options.include_time_estimates);
        assert!(config.max_update_frequency > 0.0_f64);
    }

    #[test]
    fn test_progress_rate_limiting() {
        let config = AccessibilityConfig {
            max_update_frequency: 10.0_f64, // 10 Hz max
            ..Default::default()
        };
        let mut reporter = ProgressReporter::with_config(config);

        reporter.start("Rate limit test", Some(100));

        // Rapid updates should be rate limited
        for i in 0..10 {
            reporter.update(i);
            // In real usage, only some updates would be processed due to rate limiting
        }

        reporter.finish(None);
    }

    #[test]
    fn test_duration_formatting() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h1m");
    }
}
