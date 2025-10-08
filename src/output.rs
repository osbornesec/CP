//! Output handling module

/// Output manager for extracted sections
#[non_exhaustive]
pub struct OutputManager;

impl OutputManager {
    /// Create new output manager
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        return Self;
    }
}

impl Default for OutputManager {
    #[inline]
    fn default() -> Self {
        return Self::new();
    }
}
