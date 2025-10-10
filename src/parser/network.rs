use crate::error::CpinfoError;

/// Determines if a `CpinfoError` represents a transient error that should be retried.
///
/// This function analyzes error types to distinguish between transient errors
/// (like network timeouts or connection issues) and permanent errors.
///
/// # Arguments
///
/// * `error` - The `CpinfoError` to analyze for transience
///
/// # Returns
///
/// Returns `true` if the error is transient and retry is appropriate,
/// `false` if the error is permanent and retry would not help.
#[must_use]
#[inline]
pub fn is_transient_error(error: &CpinfoError) -> bool {
    // Use string-based error analysis to avoid pattern matching issues
    let error_string = format!("{error}");

    // Check for I/O errors with transient patterns
    if error_string.starts_with("I/O error:") {
        let is_timeout = error_string.contains("timed out") || error_string.contains("TimedOut");
        let is_interrupted =
            error_string.contains("interrupted") || error_string.contains("Interrupted");
        let is_would_block =
            error_string.contains("would block") || error_string.contains("WouldBlock");
        let is_connection_error = error_string.contains("connection aborted")
            || error_string.contains("connection reset")
            || error_string.contains("ConnectionAborted")
            || error_string.contains("ConnectionReset");

        return is_timeout || is_interrupted || is_would_block || is_connection_error;
    }

    // Check for network errors (transient)
    if error_string.starts_with("Network error:") {
        return true;
    }

    // Check for file not found (transient in distributed systems)
    if error_string.starts_with("File not found:") {
        return true;
    }

    // All other errors are considered permanent
    return false;
}
