//! Standard classification rules implementations

use super::super::traits::ClassificationRule;
use super::super::ClassificationLevel;

/// Rule for general public information
#[non_exhaustive]
pub struct PublicInfoRule;

impl ClassificationRule for PublicInfoRule {
    #[inline]
    fn applies(&self, name: &str, _content: &str) -> bool {
        return name.to_lowercase().contains("general")
            || name.to_lowercase().contains("public")
            || name.to_lowercase().contains("info");
    }

    #[inline]
    fn description(&self) -> &'static str {
        return "General public information sections";
    }

    #[inline]
    fn level(&self) -> ClassificationLevel {
        return ClassificationLevel::Public;
    }
}

/// Rule for network configuration sections
#[non_exhaustive]
pub struct NetworkConfigRule;

impl ClassificationRule for NetworkConfigRule {
    #[inline]
    fn applies(&self, name: &str, content: &str) -> bool {
        let name_lower = name.to_lowercase();
        return name_lower.contains("network")
            || name_lower.contains("config")
            || content.contains("interface")
            || content.contains("route")
            || content.contains("gateway")
            || content.contains("subnet");
    }

    #[inline]
    fn description(&self) -> &'static str {
        return "Network configuration sections";
    }

    #[inline]
    fn level(&self) -> ClassificationLevel {
        return ClassificationLevel::Internal;
    }
}

/// Rule for security policy sections
#[non_exhaustive]
pub struct SecurityPolicyRule;

impl ClassificationRule for SecurityPolicyRule {
    #[inline]
    fn applies(&self, name: &str, content: &str) -> bool {
        let name_lower = name.to_lowercase();
        let content_lower = content.to_lowercase();

        return name_lower.contains("security")
            || name_lower.contains("policy")
            || content_lower.contains("firewall")
            || content_lower.contains("access control")
            || content_lower.contains("authentication")
            || content_lower.contains("authorization");
    }

    #[inline]
    fn description(&self) -> &'static str {
        return "Security policy and configuration sections";
    }

    #[inline]
    fn level(&self) -> ClassificationLevel {
        return ClassificationLevel::Confidential;
    }
}

/// Rule for system status information
#[non_exhaustive]
pub struct SystemStatusRule;

impl ClassificationRule for SystemStatusRule {
    #[inline]
    fn applies(&self, name: &str, _content: &str) -> bool {
        let name_lower = name.to_lowercase();
        return name_lower.contains("system")
            || name_lower.contains("status")
            || name_lower.contains("health")
            || name_lower.contains("monitor")
            || name_lower.contains("performance");
    }

    #[inline]
    fn description(&self) -> &'static str {
        return "System status and health information";
    }

    #[inline]
    fn level(&self) -> ClassificationLevel {
        return ClassificationLevel::Internal;
    }
}

/// Rule for hardware information
#[non_exhaustive]
pub struct HardwareInfoRule;

impl ClassificationRule for HardwareInfoRule {
    #[inline]
    fn applies(&self, name: &str, content: &str) -> bool {
        let name_lower = name.to_lowercase();
        return name_lower.contains("hardware")
            || name_lower.contains("device")
            || content.contains("CPU")
            || content.contains("memory")
            || content.contains("disk")
            || content.contains("serial");
    }

    #[inline]
    fn description(&self) -> &'static str {
        return "Hardware and device information";
    }

    #[inline]
    fn level(&self) -> ClassificationLevel {
        return ClassificationLevel::Internal;
    }
}

/// Rule for user management sections
#[non_exhaustive]
pub struct UserManagementRule;

impl ClassificationRule for UserManagementRule {
    #[inline]
    fn applies(&self, name: &str, content: &str) -> bool {
        let name_lower = name.to_lowercase();
        let content_lower = content.to_lowercase();

        return name_lower.contains("user")
            || name_lower.contains("account")
            || content_lower.contains("username")
            || content_lower.contains("user id")
            || content_lower.contains("privilege");
    }

    #[inline]
    fn description(&self) -> &'static str {
        return "User management and account information";
    }

    #[inline]
    fn level(&self) -> ClassificationLevel {
        return ClassificationLevel::Confidential;
    }
}

/// Rule for log files and audit trails
#[non_exhaustive]
pub struct LogFileRule;

impl ClassificationRule for LogFileRule {
    #[inline]
    fn applies(&self, name: &str, content: &str) -> bool {
        let name_lower = name.to_lowercase();
        let content_lower = content.to_lowercase();

        return name_lower.contains("log")
            || name_lower.contains("audit")
            || content_lower.contains("timestamp")
            || content_lower.contains("event")
            || content_lower.contains("error");
    }

    #[inline]
    fn description(&self) -> &'static str {
        return "Log files and audit trail information";
    }

    #[inline]
    fn level(&self) -> ClassificationLevel {
        return ClassificationLevel::Internal;
    }
}
