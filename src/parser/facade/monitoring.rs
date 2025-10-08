//! Monitoring operations facade
//!
//! This module contains all monitoring and performance measurement functionality
//! including memory monitoring, speed monitoring, and basic section parsing.

use std::path::Path;

use crate::parser::{
    monitoring,
    stats::{MemoryStats, SpeedStats},
};

/// Monitoring operations facade
#[non_exhaustive]
pub struct MonitoringFacade;

impl MonitoringFacade {
    /// Parse sections from a basic cpinfo file
    ///
    /// Provides simple section counting functionality.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the cpinfo file
    ///
    /// # Returns
    ///
    /// The number of sections found in the file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or read.
    #[inline]
    pub fn parse_sections_basic<P: AsRef<Path>>(path: P) -> crate::error::Result<usize> {
        return monitoring::parse_sections_basic(path);
    }

    /// Parse with memory monitoring
    ///
    /// Delegates to the monitoring module for memory usage tracking.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the cpinfo file to parse
    /// * `config` - Performance configuration including memory limits
    ///
    /// # Returns
    ///
    /// `MemoryStats` containing memory usage information
    ///
    /// # Errors
    ///
    /// Returns an error if memory limits are exceeded or parsing fails.
    #[inline]
    pub fn parse_with_memory_monitoring<P: AsRef<Path>>(
        path: P,
        config: &crate::parser::config::PerformanceConfig,
    ) -> crate::error::Result<MemoryStats> {
        return monitoring::parse_with_memory_monitoring(path, config);
    }

    /// Parse file with speed monitoring
    ///
    /// Delegates to the monitoring module for processing speed measurement.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the cpinfo file to parse
    /// * `config` - Performance configuration including buffer settings
    ///
    /// # Returns
    ///
    /// `SpeedStats` containing processing speed information
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails or speed monitoring encounters issues.
    #[inline]
    pub fn parse_with_speed_monitoring<P: AsRef<Path>>(
        path: P,
        config: &crate::parser::config::PerformanceConfig,
    ) -> crate::error::Result<SpeedStats> {
        return monitoring::parse_with_speed_monitoring(path, config);
    }
}
