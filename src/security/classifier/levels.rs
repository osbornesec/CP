//! Classification levels for data sensitivity

use core::fmt;

/// Classification levels for data sensitivity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ClassificationLevel {
    /// Confidential data - restricted access required
    Confidential,
    /// Internal data - company internal use only
    Internal,
    /// Public data - no restriction on access
    Public,
    /// Restricted data - highest security level
    Restricted,
}

impl ClassificationLevel {
    /// Get all classification levels in order
    #[must_use]
    #[inline]
    pub const fn all_levels() -> &'static [Self] {
        return &[
            Self::Confidential,
            Self::Internal,
            Self::Public,
            Self::Restricted,
        ];
    }

    /// Get string representation for display
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        return match *self {
            Self::Confidential => "CONFIDENTIAL",
            Self::Internal => "INTERNAL",
            Self::Public => "PUBLIC",
            Self::Restricted => "RESTRICTED",
        };
    }

    /// Get color code for UI display
    #[must_use]
    #[inline]
    pub const fn color_code(&self) -> &'static str {
        return match *self {
            Self::Confidential => "orange",
            Self::Internal => "blue",
            Self::Public => "green",
            Self::Restricted => "red",
        };
    }

    /// Check if level requires special handling
    #[must_use]
    #[inline]
    pub const fn requires_special_handling(&self) -> bool {
        return matches!(*self, Self::Confidential | Self::Restricted);
    }

    /// Get security level as numeric value (higher = more secure)
    #[must_use]
    #[inline]
    pub const fn security_level(&self) -> u8 {
        return match *self {
            Self::Confidential => 2,
            Self::Internal => 1,
            Self::Public => 0,
            Self::Restricted => 3,
        };
    }
}

impl fmt::Display for ClassificationLevel {
    #[inline]
    #[allow(
        clippy::min_ident_chars,
        reason = "fmt trait requires single-letter parameter name"
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.as_str());
    }
}

impl From<u8> for ClassificationLevel {
    #[inline]
    fn from(security_level: u8) -> Self {
        return match security_level {
            0 => Self::Public,
            1 => Self::Internal,
            2 => Self::Confidential,
            _ => Self::Restricted,
        };
    }
}

impl From<&str> for ClassificationLevel {
    #[inline]
    fn from(level_string: &str) -> Self {
        return match level_string.to_uppercase().as_str() {
            "CONFIDENTIAL" => Self::Confidential,
            "INTERNAL" => Self::Internal,
            "RESTRICTED" => Self::Restricted,
            _ => Self::Public, // Default to safest public level
        };
    }
}
