//! Memory monitoring utilities

#[cfg(test)]
use crate::parser::monitoring::monitoring_config::DEFAULT_TEST_MEMORY_BASE;

/// Memory monitoring builder for configurable memory tracking
pub struct MemoryMonitorBuilder {
    check_interval: u64,
    enable_monitoring: bool,
    max_memory_mb: Option<usize>,
}

/// Memory monitor for tracking memory usage during processing
pub struct MemoryMonitor {
    check_interval: u64,
    enable_monitoring: bool,
    max_memory_mb: Option<usize>,
}

impl Default for MemoryMonitorBuilder {
    #[inline]
    fn default() -> Self {
        return Self {
            check_interval: 1000,
            enable_monitoring: false,
            max_memory_mb: None,
        };
    }
}

impl MemoryMonitorBuilder {
    /// Build the memory monitor
    #[inline]
    #[must_use]
    pub const fn build(self) -> MemoryMonitor {
        return MemoryMonitor {
            check_interval: self.check_interval,
            enable_monitoring: self.enable_monitoring,
            max_memory_mb: self.max_memory_mb,
        };
    }

    /// Set memory check interval
    #[inline]
    #[must_use]
    pub const fn check_interval(mut self, interval: u64) -> Self {
        self.check_interval = interval;
        return self;
    }

    /// Enable or disable monitoring
    #[inline]
    #[must_use]
    pub const fn enable_monitoring(mut self, enable: bool) -> Self {
        self.enable_monitoring = enable;
        return self;
    }

    /// Set maximum memory limit
    #[inline]
    #[must_use]
    pub const fn max_memory_mb(mut self, max_megabytes: usize) -> Self {
        self.max_memory_mb = Some(max_megabytes);
        self.enable_monitoring = true;
        return self;
    }

    /// Create new memory monitor builder
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        return Self::default();
    }
}

impl MemoryMonitor {
    /// Check if memory usage exceeds the configured limit
    #[inline]
    #[must_use]
    pub fn exceeds_limit(&self, current_memory: f64) -> bool {
        if let Some(max_megabytes) = self.max_memory_mb {
            let current_memory_positive = if current_memory < 0.0 {
                0.0
            } else {
                current_memory
            };
            let current_memory_clamped = current_memory_positive.min(f64::from(u32::MAX));
            // Safe conversion with bounds checking
            let current_memory_u32 =
                if current_memory_clamped.is_finite() && current_memory_clamped >= 0.0 {
                    if current_memory_clamped <= f64::from(u32::MAX) {
                        #[allow(
                            clippy::cast_possible_truncation,
                            clippy::cast_sign_loss,
                            reason = "bounds checked conversion from f64 to u32"
                        )]
                        {
                            current_memory_clamped.trunc() as u32
                        }
                    } else {
                        u32::MAX
                    }
                } else {
                    0_u32
                };
            return current_memory_u32 > u32::try_from(max_megabytes).unwrap_or(u32::MAX);
        } else {
            return false;
        }
    }

    /// Get the maximum memory limit
    #[inline]
    #[must_use]
    pub const fn max_memory_mb(&self) -> Option<usize> {
        return self.max_memory_mb;
    }

    /// Check if memory usage should be monitored at this line count
    #[inline]
    #[must_use]
    pub const fn should_check_memory(&self, line_count: u64) -> bool {
        if self.enable_monitoring {
            let remainder = line_count.wrapping_rem(self.check_interval);
            return remainder == 0;
        } else {
            return false;
        }
    }
}

/// Get current memory usage in megabytes
///
/// In test environments, returns a simulated value.
/// In production, queries the system for actual RSS memory usage.
#[inline]
#[must_use]
#[allow(
    clippy::missing_const_for_fn,
    reason = "Function conditionally performs runtime process inspection"
)]
pub fn get_memory_usage_mb() -> f64 {
    #[cfg(test)]
    {
        return DEFAULT_TEST_MEMORY_BASE;
    }

    #[cfg(not(test))]
    {
        let result = std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .ok()
            .and_then(|output| {
                return String::from_utf8(output.stdout).ok();
            })
            .and_then(|memory_string| {
                return memory_string.trim().parse::<f64>().ok();
            });

        match result {
            Some(kilobytes) => {
                // Convert KB to MB using safe conversion
                if kilobytes.is_finite() && kilobytes >= 0.0 && kilobytes <= (u64::MAX as f64) {
                    #[allow(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        reason = "bounds checked conversion from f64 to u64"
                    )]
                    let kilobytes_u64 = kilobytes.trunc() as u64;
                    let megabytes = kilobytes_u64.saturating_div(1024_u64);
                    return f64::from(u32::try_from(megabytes).unwrap_or(u32::MAX));
                } else {
                    return 0.0;
                }
            }
            None => {
                return 50.0;
            }
        }
    }
}
