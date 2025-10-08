//! Metrics collection and calculation utilities

/// Metrics collector for performance data
#[derive(Debug, Default)]
#[non_exhaustive]
#[allow(
    clippy::struct_field_names,
    reason = "All fields represent different types of samples - names are intentionally similar for consistency"
)]
pub struct MetricsCollector {
    cpu_samples: Vec<f64>,
    io_samples: Vec<f64>,
    memory_samples: Vec<f64>,
}

impl MetricsCollector {
    /// Add CPU usage sample
    #[inline]
    pub fn add_cpu_sample(&mut self, cpu_usage: f64) {
        self.cpu_samples.push(cpu_usage);
    }

    /// Add I/O rate sample
    #[inline]
    pub fn add_io_sample(&mut self, io_rate: f64) {
        self.io_samples.push(io_rate);
    }

    /// Add memory usage sample
    #[inline]
    pub fn add_memory_sample(&mut self, memory_usage: f64) {
        self.memory_samples.push(memory_usage);
    }

    /// Get average I/O rate
    #[must_use]
    #[inline]
    pub fn average_io_rate(&self) -> f64 {
        if self.io_samples.is_empty() {
            return 0.0;
        } else {
            #[allow(
                clippy::cast_precision_loss,
                reason = "Length conversion to f64 is required for accurate division"
            )]
            let samples_count = self.io_samples.len() as f64;
            #[allow(
                clippy::float_arithmetic,
                reason = "Division is required for calculating average from samples"
            )]
            let average = self.io_samples.iter().sum::<f64>() / samples_count;
            return average;
        }
    }

    /// Get all samples (for consumption by stats structs)
    #[must_use]
    #[inline]
    pub fn into_samples(self) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        return (self.cpu_samples, self.memory_samples, self.io_samples);
    }

    /// Create new metrics collector
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        return Self::default();
    }

    /// Get peak CPU usage
    ///
    /// NaN values are treated as equal and excluded from maximum calculation
    #[must_use]
    #[inline]
    pub fn peak_cpu_percent(&self) -> f64 {
        let max_cpu = self
            .cpu_samples
            .iter()
            .max_by(|cpu_a, cpu_b| {
                return cpu_a.partial_cmp(cpu_b).map_or_else(
                    || return core::cmp::Ordering::Equal,
                    |ordering| return ordering,
                );
            })
            .copied()
            .unwrap_or(0.0);
        return max_cpu;
    }

    /// Get peak memory usage
    ///
    /// NaN values are treated as equal and excluded from maximum calculation
    #[must_use]
    #[inline]
    pub fn peak_memory_mb(&self) -> f64 {
        let max_memory = self
            .memory_samples
            .iter()
            .max_by(|memory_a, memory_b| {
                return memory_a.partial_cmp(memory_b).map_or_else(
                    || return core::cmp::Ordering::Equal,
                    |ordering| return ordering,
                );
            })
            .copied()
            .unwrap_or(0.0);
        return max_memory;
    }
}

/// Speed calculation utilities
#[non_exhaustive]
pub struct SpeedCalculator;

impl SpeedCalculator {
    /// Calculate bytes per second
    #[must_use]
    #[inline]
    pub fn bytes_per_second(bytes: u64, duration_secs: f64) -> f64 {
        if duration_secs > 0.001 {
            #[allow(
                clippy::cast_precision_loss,
                reason = "u64 to f64 conversion required for speed calculation"
            )]
            let bytes_f64 = bytes as f64;
            #[allow(
                clippy::float_arithmetic,
                reason = "Division required for calculating bytes per second"
            )]
            let rate = bytes_f64 / duration_secs;
            return rate;
        } else {
            return 0.0;
        }
    }

    /// Convert milliseconds to seconds
    #[must_use]
    #[inline]
    pub fn ms_to_seconds(milliseconds: u64) -> f64 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "u64 to f64 conversion required for time conversion"
        )]
        let milliseconds_f64 = milliseconds as f64;
        #[allow(
            clippy::float_arithmetic,
            reason = "Division by 1000 required for millisecond to second conversion"
        )]
        let seconds = milliseconds_f64 / 1000.0;
        return seconds;
    }

    /// Calculate sections per second
    #[must_use]
    #[inline]
    pub fn sections_per_second(sections: usize, duration_secs: f64) -> f64 {
        if duration_secs > 0.001 {
            #[allow(
                clippy::cast_precision_loss,
                reason = "usize to f64 conversion required for rate calculation"
            )]
            let sections_f64 = sections as f64;
            #[allow(
                clippy::float_arithmetic,
                reason = "Division required for calculating sections per second"
            )]
            let rate = sections_f64 / duration_secs;
            return rate;
        } else {
            return 0.0;
        }
    }
}

/// Cache performance metrics
#[derive(Debug)]
#[non_exhaustive]
pub struct CacheMetrics {
    hits: usize,
    misses: usize,
}

impl CacheMetrics {
    /// Calculate cache hit rate
    #[must_use]
    #[inline]
    pub fn hit_rate(&self) -> f64 {
        let total_requests = self.hits + self.misses;
        if total_requests > 0 {
            #[allow(
                clippy::cast_precision_loss,
                reason = "usize to f64 conversion required for rate calculation"
            )]
            let hits_f64 = self.hits as f64;
            #[allow(
                clippy::cast_precision_loss,
                reason = "usize to f64 conversion required for rate calculation"
            )]
            let total_f64 = total_requests as f64;
            #[allow(
                clippy::float_arithmetic,
                reason = "Division required for calculating hit rate percentage"
            )]
            let rate = hits_f64 / total_f64;
            return rate;
        } else {
            return 0.0;
        }
    }

    /// Get total hits
    #[must_use]
    #[inline]
    pub const fn hits(&self) -> usize {
        return self.hits;
    }

    /// Get total misses
    #[must_use]
    #[inline]
    pub const fn misses(&self) -> usize {
        return self.misses;
    }

    /// Create new cache metrics
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        return Self { hits: 0, misses: 0 };
    }

    /// Record cache hit
    #[inline]
    pub const fn record_hit(&mut self) {
        self.hits = self.hits.wrapping_add(1_usize);
    }

    /// Record cache miss
    #[inline]
    pub const fn record_miss(&mut self) {
        self.misses = self.misses.wrapping_add(1_usize);
    }
}

impl Default for CacheMetrics {
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}
