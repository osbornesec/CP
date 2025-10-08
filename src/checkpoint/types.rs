use bitflags::bitflags;

use serde::{Deserialize, Serialize};

/// Version information parsed from cpinfo files
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct VersionInfo {
    pub build: String,
    pub kernel_build: Option<String>,
    pub kernel_version: Option<String>,
    pub version: String,
}

bitflags! {
    /// Security blade configuration flags
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SecurityBlades: u8 {
        const FIREWALL = 0b0000_0001;
        const VPN = 0b0000_0010;
        const URL_FILTERING = 0b0000_0100;
        const APPLICATION_CONTROL = 0b0000_1000;
        const IPS = 0b0001_0000;
        const IDENTITY_SERVER = 0b0010_0000;
        const MONITORING = 0b0100_0000;
    }
}

impl SecurityBlades {
    /// Check if firewall blade is enabled
    #[inline]
    #[must_use]
    pub const fn firewall_enabled(&self) -> bool {
        return self.contains(Self::FIREWALL);
    }
}

/// Core `CheckPoint` parser structure.
///
/// This represents the main parser context for processing `CheckPoint`
/// cpinfo diagnostic files. Currently implemented as a unit struct
/// for future extensibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Checkpoint;

/// Results from streaming parser operations.
///
/// Contains metrics and outcomes from processing a cpinfo file
/// using the streaming parser, including performance statistics
/// and extraction results.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct StreamingResult {
    /// Total size of the processed cpinfo file in bytes
    pub file_size: u64,
    /// Peak memory usage during parsing in megabytes
    pub memory_peak_mb: usize,
    /// Number of distinct sections found in the cpinfo file
    pub sections_found: usize,
}

/// Memory monitoring statistics for parser operations.
///
/// Tracks memory usage patterns during cpinfo file processing
/// to ensure streaming performance characteristics and detect
/// potential memory leaks or excessive allocations.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct MemoryStats {
    /// Final memory usage at completion in megabytes
    pub final_memory_mb: usize,
    /// Number of potential memory leaks detected
    pub memory_leaks_detected: usize,
    /// Peak memory usage during processing in megabytes
    pub peak_memory_mb: usize,
}

/// VSX (Virtual System Extension) deployment information.
///
/// Contains details about `CheckPoint` VSX virtualization setup,
/// including the deployment model and all configured virtual systems.
/// VSX allows multiple virtual firewalls to run on a single platform.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct VsxDeployment {
    /// Type of VSX deployment (e.g., "VSX Gateway", "VSX Management")
    pub deployment_type: String,
    /// List of all virtual systems configured in this deployment
    pub virtual_systems: Vec<VirtualSystem>,
}

/// Individual virtual system within a VSX deployment.
///
/// Represents a single virtual firewall instance with its own
/// security policies, interfaces, and operational state within
/// the VSX virtualization framework.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct VirtualSystem {
    /// Context type of the virtual system (e.g., "firewall", "router")
    pub context_type: String,
    /// Unique identifier for this virtual system
    pub id: u32,
    /// List of network interfaces assigned to this virtual system
    pub interfaces: Vec<String>,
    /// Display name of the virtual system
    pub name: String,
    /// Current operational state (e.g., "Active", "Inactive", "Error")
    pub state: String,
}

/// Cluster configuration information from `CheckPoint` cpinfo files.
///
/// Contains details about high-availability cluster setup including
/// cluster type, member information, and synchronization state.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ClusterConfiguration {
    /// Type of cluster configuration (e.g., "`ClusterXL`", "`VRRP`")
    pub cluster_type: String,
    /// Information about the local cluster member
    pub local_member: ClusterMember,
    /// Total number of cluster members
    pub member_count: u32,
    /// List of remote cluster members
    pub remote_members: Vec<ClusterMember>,
}

/// Individual cluster member information.
///
/// Represents a single node in a `CheckPoint` cluster configuration,
/// including its network details and current operational state.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ClusterMember {
    /// IP address of the cluster member
    pub ip: String,
    /// Hostname or display name of the cluster member
    pub name: String,
    /// Cluster priority value for failover ordering
    pub priority: u32,
    /// Current operational state (e.g., "Active", "Standby", "Down")
    pub state: String,
}

/// Security policy configuration and statistics.
///
/// Contains summary information about the `CheckPoint` security
/// policy rules, including counts by action type and detailed
/// rule definitions for analysis and auditing.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SecurityPolicies {
    /// Number of rules with "allow" or "accept" actions
    pub allow_rules: u32,
    /// Number of rules with "drop" or "reject" actions
    pub drop_rules: u32,
    /// Detailed list of individual policy rules
    pub rules: Vec<PolicyRule>,
    /// Total count of all security policy rules
    pub total_rules: u32,
}

/// Individual security policy rule definition.
///
/// Represents a single rule in the `CheckPoint` security policy,
/// defining traffic matching criteria and the action to take
/// when traffic matches the rule conditions.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct PolicyRule {
    /// Action to take when rule matches (e.g., "Accept", "Drop", "Reject")
    pub action: String,
    /// Destination network objects or addresses
    pub destination: String,
    /// Human-readable name or identifier for the rule
    pub name: String,
    /// Service or port specifications for the rule
    pub service: String,
    /// Source network objects or addresses
    pub source: String,
}

/// Network interface configuration summary.
///
/// Contains information about all network interfaces configured
/// on the `CheckPoint` system, including their addresses, states,
/// and configuration parameters.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct NetworkConfiguration {
    /// Detailed information for each network interface
    pub interfaces: Vec<NetworkInterface>,
    /// Total count of configured network interfaces
    pub total_interfaces: u32,
}

/// Individual network interface configuration.
///
/// Represents a single network interface on the `CheckPoint` system
/// with its addressing information, operational parameters,
/// and current status.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct NetworkInterface {
    /// IP address assigned to the interface
    pub ip_address: String,
    /// Maximum Transmission Unit size in bytes
    pub mtu: u32,
    /// Interface name or identifier (e.g., "eth0", "bond0")
    pub name: String,
    /// Current operational state (e.g., "Up", "Down", "Admin Down")
    pub state: String,
    /// Subnet mask for the interface's network
    pub subnet_mask: String,
}

/// VPN (Virtual Private Network) configuration information.
///
/// Contains details about `CheckPoint` VPN setup including
/// tunnel configurations, connection statistics, and
/// remote access capabilities.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct VpnConfiguration {
    /// Number of currently active VPN tunnels
    pub active_tunnels: u32,
    /// Whether remote access VPN is enabled
    pub remote_access_enabled: bool,
    /// Total number of configured VPN tunnels
    pub total_tunnels: u32,
    /// Detailed information for each VPN tunnel
    pub tunnels: Vec<VpnTunnel>,
}

/// Individual VPN tunnel configuration.
///
/// Represents a single VPN tunnel with its security parameters,
/// peer information, and current connection status.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct VpnTunnel {
    /// Authentication method used for the tunnel (e.g., "PSK", "Certificates")
    pub authentication: String,
    /// Encryption algorithm and key size (e.g., "AES-256", "3DES")
    pub encryption: String,
    /// Human-readable name for the tunnel
    pub name: String,
    /// Remote peer IP address or hostname
    pub remote_peer: String,
    /// Current tunnel status (e.g., "Up", "Down", "Negotiating")
    pub status: String,
}

/// High Availability (HA) status and configuration.
///
/// Contains information about `CheckPoint` HA cluster status,
/// including failover configuration, member states, and
/// synchronization status between cluster members.
#[derive(Default, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct HaStatus {
    /// Failover mode configuration (e.g., "Active/Standby", "Load Sharing")
    pub failover_mode: String,
    /// Whether High Availability is enabled
    pub ha_enabled: bool,
    /// State of the local cluster member (e.g., "Active", "Standby")
    pub local_state: String,
    /// State of the peer cluster member (e.g., "Active", "Standby", "Down")
    pub peer_state: String,
    /// Synchronization status between cluster members (e.g., "In Sync", "Out of Sync")
    pub sync_status: String,
}

/// Log file information and statistics.
///
/// Contains summary information about `CheckPoint` log files
/// including available log types, retention periods, and
/// storage utilization metrics.
#[derive(Default, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct LogInformation {
    /// List of available log types (e.g., "fw", "vpn", "audit")
    pub log_types: Vec<String>,
    /// Timestamp of the oldest log entry
    pub oldest_entry: String,
    /// Total number of distinct log types
    pub total_log_types: u32,
    /// Total size of all log files in megabytes
    pub total_size_mb: u32,
}

/// Certificate store information and validation status.
///
/// Contains details about PKI certificates used by the `CheckPoint`
/// system, including certificate validity status and expiration
/// tracking for security and compliance monitoring.
#[derive(Default, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CertificateInformation {
    /// Detailed information for each certificate
    pub certificates: Vec<Certificate>,
    /// Number of certificates that have expired
    pub expired_certificates: u32,
    /// Total number of certificates in the store
    pub total_certificates: u32,
    /// Number of currently valid certificates
    pub valid_certificates: u32,
}

/// Individual PKI certificate information.
///
/// Represents a single certificate with its identifying information,
/// validity period, and current status for security and compliance
/// tracking purposes.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Certificate {
    /// Certificate expiration date and time
    pub expires: String,
    /// Certificate Authority that issued this certificate
    pub issuer: String,
    /// Common name or identifier for the certificate
    pub name: String,
    /// Current validity status (e.g., "Valid", "Expired", "Revoked")
    pub status: String,
    /// Certificate subject distinguished name
    pub subject: String,
}

/// System performance metrics and resource utilization.
///
/// Contains real-time and historical performance data from the
/// `CheckPoint` system, including resource utilization and
/// throughput statistics for capacity planning and monitoring.
#[derive(Default, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct PerformanceMetrics {
    /// Number of new connections established per second
    pub connections_per_second: u32,
    /// CPU utilization as a percentage (0.0 to 100.0)
    pub cpu_usage_percent: f64,
    /// Disk space utilization as a percentage (0.0 to 100.0)
    pub disk_usage_percent: f64,
    /// Memory utilization as a percentage (0.0 to 100.0)
    pub memory_usage_percent: f64,
    /// Network throughput in megabits per second
    pub throughput_mbps: f64,
}
