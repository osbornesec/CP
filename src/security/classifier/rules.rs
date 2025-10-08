//! Classification rules and pattern matching implementations

pub mod rules_content;
pub mod rules_patterns;
pub mod rules_sensitivity;
pub mod rules_standard;

pub use rules_content::ContentClassifier;
pub use rules_patterns::PatternMatcher;
pub use rules_sensitivity::SensitivityAnalyzer;
pub use rules_standard::{
    HardwareInfoRule, NetworkConfigRule, PublicInfoRule, SecurityPolicyRule, SystemStatusRule,
};
