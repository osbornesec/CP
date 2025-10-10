//! `CheckPoint` `cpinfo` file parser and detector implementation.
//!
//! This module provides the core `CheckPoint` detection and parsing functionality
//! for various `cpinfo` file formats and configurations.

pub mod monitoring;
pub mod network;
pub(in crate::checkpoint) mod regex_utils;
pub mod security;
pub mod types;
pub mod version;
pub mod vsx;

pub use types::{
    Certificate, CertificateInformation, ClusterConfiguration, ClusterMember, HaStatus,
    LogInformation, MemoryStats, NetworkConfiguration, NetworkInterface, PerformanceMetrics,
    PolicyRule, SecurityBlades, SecurityPolicies, StreamingResult, VersionInfo, VirtualSystem,
    VpnConfiguration, VpnTunnel, VsxDeployment,
};

/// `CheckPoint` file format detector.
#[non_exhaustive]
pub struct CheckPointDetector;

impl CheckPointDetector {
    /// Create a new `CheckPoint` detector instance.
    ///
    /// # Returns
    ///
    /// A new `CheckPointDetector` instance ready for use.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        return Self;
    }
}

impl Default for CheckPointDetector {
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}

/// `CheckPoint` `cpinfo` parser implementation.
#[non_exhaustive]
pub struct CheckPointParser;

impl CheckPointParser {
    /// Create a new `CheckPoint` parser instance.
    ///
    /// # Returns
    ///
    /// A new `CheckPointParser` instance ready for parsing operations.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        return Self;
    }

    /// Parse certificate information from a `cpinfo` file.
    ///
    /// Extracts `SSL`/`TLS` certificate details, validity periods, and trust chain
    /// information from `CheckPoint` security gateways.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `cpinfo` file containing certificate data
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `CertificateInformation` on success.
    ///
    /// # Errors
    ///
    /// Returns a `CpinfoError` if the file cannot be read or certificate parsing fails.
    #[inline]
    pub fn parse_certificate_info<P: AsRef<std::path::Path>>(
        path: P,
    ) -> crate::error::Result<CertificateInformation> {
        return monitoring::parse_certificate_info_impl(path);
    }

    /// Parse cluster configuration from a `cpinfo` file.
    ///
    /// Extracts cluster topology, member node configurations, load balancing
    /// settings, and high availability parameters from `CheckPoint` cluster deployments.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `cpinfo` file containing cluster configuration
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `ClusterConfiguration` on success.
    ///
    /// # Errors
    ///
    /// Returns `CpinfoError` if:
    /// - The file cannot be read or accessed
    /// - Regex compilation fails during cluster pattern matching
    /// - Cluster configuration data parsing fails
    #[inline]
    pub fn parse_cluster_configuration<P: AsRef<std::path::Path>>(
        path: P,
    ) -> crate::error::Result<ClusterConfiguration> {
        return security::parse_cluster_configuration_impl(path);
    }

    /// Parse High Availability (`HA`) status from a `cpinfo` file.
    ///
    /// Extracts cluster state, member status, and synchronization information
    /// from `CheckPoint` `HA`-enabled systems.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `cpinfo` file containing `HA` information
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `HaStatus` on success.
    ///
    /// # Errors
    ///
    /// Returns a `CpinfoError` if the file cannot be read or `HA` parsing fails.
    #[inline]
    pub fn parse_ha_status<P: AsRef<std::path::Path>>(path: P) -> crate::error::Result<HaStatus> {
        return monitoring::parse_ha_status_impl(path);
    }

    /// Parse log sections from a `cpinfo` file.
    ///
    /// Extracts structured log information including audit trails, system events,
    /// and security incidents from `CheckPoint` logging facilities.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `cpinfo` file containing log data
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `LogInformation` on success.
    ///
    /// # Errors
    ///
    /// Returns a `CpinfoError` if the file cannot be read or log parsing fails.
    #[inline]
    pub fn parse_log_sections<P: AsRef<std::path::Path>>(
        path: P,
    ) -> crate::error::Result<LogInformation> {
        return monitoring::parse_log_sections_impl(path);
    }

    /// Parse network interface configuration from a `cpinfo` file.
    ///
    /// Extracts detailed network interface information including `IP` addresses,
    /// routing tables, `VLAN` configurations, and interface statistics.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `cpinfo` file containing network configuration
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `NetworkConfiguration` on success.
    ///
    /// # Errors
    ///
    /// Returns `CpinfoError` if:
    /// - The file cannot be read or accessed
    /// - Regex compilation fails during pattern matching
    /// - Network data parsing encounters malformed content
    #[inline]
    pub fn parse_network_interfaces<P: AsRef<std::path::Path>>(
        path: P,
    ) -> crate::error::Result<NetworkConfiguration> {
        return network::extract_network_interfaces(path);
    }

    /// Parse performance metrics from a `cpinfo` file.
    ///
    /// Extracts system performance data including CPU, memory, and disk usage
    /// statistics from `CheckPoint` diagnostic files.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `cpinfo` file to parse
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `PerformanceMetrics` on success.
    ///
    /// # Errors
    ///
    /// Returns a `CpinfoError` if:
    /// - The file cannot be read or accessed
    /// - The content does not match the expected `cpinfo` format
    /// - Parsing of performance data fails
    #[inline]
    pub fn parse_performance_metrics<P: AsRef<std::path::Path>>(
        path: P,
    ) -> crate::error::Result<PerformanceMetrics> {
        return monitoring::parse_performance_metrics_impl(path);
    }

    /// Parse security blades information from a `cpinfo` file.
    ///
    /// Extracts security blade configurations, licensing status, and feature
    /// enablement details from `CheckPoint` security management systems.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `cpinfo` file containing security blades data
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `SecurityBlades` on success.
    ///
    /// # Errors
    ///
    /// Returns a `CpinfoError` if the file cannot be read or security blades parsing fails.
    #[inline]
    pub fn parse_security_blades<P: AsRef<std::path::Path>>(
        path: P,
    ) -> crate::error::Result<SecurityBlades> {
        return version::parse_security_blades_impl(path);
    }

    /// Parse security policies from a `cpinfo` file.
    ///
    /// Extracts firewall rules, access control policies, threat prevention
    /// configurations, and security blade settings from `CheckPoint` security management.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `cpinfo` file containing security policies
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `SecurityPolicies` on success.
    ///
    /// # Errors
    ///
    /// Returns `CpinfoError` if:
    /// - The file cannot be read or accessed
    /// - Regex compilation fails during policy pattern matching
    /// - Security policy data parsing encounters malformed rules
    #[inline]
    pub fn parse_security_policies<P: AsRef<std::path::Path>>(
        path: P,
    ) -> crate::error::Result<SecurityPolicies> {
        return security::parse_security_policies_impl(path);
    }

    /// Parse file using streaming approach for large files.
    ///
    /// Implements memory-efficient streaming parser for processing large `cpinfo`
    /// files without loading entire contents into memory. Suitable for files
    /// exceeding available system memory.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `cpinfo` file to stream and parse
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `StreamingResult` with incremental data.
    ///
    /// # Errors
    ///
    /// Returns a `CpinfoError` if the file cannot be read or streaming parsing fails.
    #[inline]
    pub fn parse_streaming<P: AsRef<std::path::Path>>(
        path: P,
    ) -> crate::error::Result<StreamingResult> {
        return monitoring::parse_streaming_impl(path);
    }

    /// Parse version information from a `cpinfo` file.
    ///
    /// Extracts `CheckPoint` product version details, build numbers, hotfix levels,
    /// and component version matrices from diagnostic files.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `cpinfo` file containing version information
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `VersionInfo` on success.
    ///
    /// # Errors
    ///
    /// Returns a `CpinfoError` if the file cannot be read or version parsing fails.
    #[inline]
    pub fn parse_version_info<P: AsRef<std::path::Path>>(
        path: P,
    ) -> crate::error::Result<VersionInfo> {
        return version::parse_version_info_impl(path);
    }

    /// Parse `VPN` configuration from a `cpinfo` file.
    ///
    /// Extracts `VPN` tunnel configurations, encryption settings, peer information,
    /// and connection status from `CheckPoint` `VPN` gateways.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `cpinfo` file containing `VPN` configuration
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `VpnConfiguration` on success.
    ///
    /// # Errors
    ///
    /// Returns `CpinfoError` if:
    /// - The file cannot be read or accessed
    /// - Regex compilation fails during `VPN` pattern matching
    /// - `VPN` configuration data parsing fails
    #[inline]
    pub fn parse_vpn_configuration<P: AsRef<std::path::Path>>(
        path: P,
    ) -> crate::error::Result<VpnConfiguration> {
        return network::parse_vpn_configuration_impl(path);
    }

    /// Parse Virtual System Extension (`VSX`) deployment information from a `cpinfo` file.
    ///
    /// Extracts `VSX` virtualization configurations, virtual system mappings,
    /// resource allocations, and virtual firewall deployments.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `cpinfo` file containing `VSX` deployment data
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `VsxDeployment` on success.
    ///
    /// # Errors
    ///
    /// Returns a `CpinfoError` if the file cannot be read or `VSX` parsing fails.
    #[inline]
    pub fn parse_vsx_deployment<P: AsRef<std::path::Path>>(
        path: P,
    ) -> crate::error::Result<VsxDeployment> {
        return vsx::parse_vsx_deployment_impl(path);
    }

    /// Parse file with comprehensive memory usage monitoring.
    ///
    /// Parses `cpinfo` files while actively monitoring memory consumption patterns,
    /// providing detailed statistics on memory usage throughout the parsing process.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `cpinfo` file to parse with memory tracking
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `MemoryStats` with usage metrics.
    ///
    /// # Errors
    ///
    /// Returns a `CpinfoError` if the file cannot be read or memory monitoring fails.
    #[inline]
    pub fn parse_with_memory_monitoring<P: AsRef<std::path::Path>>(
        path: P,
    ) -> crate::error::Result<MemoryStats> {
        return monitoring::parse_with_memory_monitoring_impl(path);
    }
}

impl Default for CheckPointParser {
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}
