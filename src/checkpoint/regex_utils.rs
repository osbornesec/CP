use crate::error::{CpinfoError, Result};
use regex::{Captures, Regex};

/// Helper to create regex with context-specific error
#[inline]
pub(in crate::checkpoint) fn create_regex(pattern: &str, context: &str) -> Result<Regex> {
    match Regex::new(pattern) {
        Ok(regex) => return Ok(regex),
        Err(error) => {
            return Err(CpinfoError::validation_error(format!(
                "Invalid {context} regex pattern: {error}"
            )))
        }
    }
}

/// Extract float value using regex pattern
#[inline]
pub(in crate::checkpoint) fn extract_float(
    text: &str,
    pattern: &str,
    context: &str,
    not_found: &str,
) -> Result<f64> {
    let regex = match create_regex(pattern, context) {
        Ok(regex) => regex,
        Err(error) => return Err(error),
    };
    let captures = match regex.captures(text) {
        Some(captures) => captures,
        None => return Err(CpinfoError::validation_error(not_found)),
    };
    let Some(value_match) = captures.get(1) else {
        return Err(CpinfoError::validation_error(format!(
            "Invalid {context}: missing capture group"
        )));
    };
    match value_match.as_str().parse() {
        Ok(value) => return Ok(value),
        Err(error) => {
            return Err(CpinfoError::validation_error(format!(
                "Invalid {context}: {error}"
            )))
        }
    }
}

/// Extract u32 value using regex pattern
#[inline]
pub(in crate::checkpoint) fn extract_u32(
    text: &str,
    pattern: &str,
    context: &str,
    not_found: &str,
) -> Result<u32> {
    let regex = match create_regex(pattern, context) {
        Ok(regex) => regex,
        Err(error) => return Err(error),
    };
    let captures = match regex.captures(text) {
        Some(captures) => captures,
        None => return Err(CpinfoError::validation_error(not_found)),
    };
    let Some(value_match) = captures.get(1) else {
        return Err(CpinfoError::validation_error(format!(
            "Invalid {context}: missing capture group"
        )));
    };
    match value_match.as_str().parse() {
        Ok(value) => return Ok(value),
        Err(error) => {
            return Err(CpinfoError::validation_error(format!(
                "Invalid {context}: {error}"
            )))
        }
    }
}

/// Extract string value using regex pattern
#[inline]
pub(in crate::checkpoint) fn extract_string(
    text: &str,
    pattern: &str,
    context: &str,
    not_found: &str,
) -> Result<String> {
    let regex = match create_regex(pattern, context) {
        Ok(regex) => regex,
        Err(error) => return Err(error),
    };
    let captures = match regex.captures(text) {
        Some(captures) => captures,
        None => return Err(CpinfoError::validation_error(not_found)),
    };
    let Some(value_match) = captures.get(1) else {
        return Err(CpinfoError::validation_error(format!(
            "Invalid {context}: missing capture group"
        )));
    };
    return Ok(value_match.as_str().trim().to_owned());
}

/// Require that a capture group exists and return its string slice
#[inline]
pub(in crate::checkpoint) fn require_capture<'captures>(
    captures: &'captures Captures<'captures>,
    index: usize,
    error_message: &str,
) -> Result<&'captures str> {
    let capture = match captures.get(index) {
        Some(value_match) => value_match.as_str(),
        None => {
            return Err(CpinfoError::validation_error(error_message.to_owned()));
        }
    };
    return Ok(capture);
}
