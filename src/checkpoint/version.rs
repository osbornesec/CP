use crate::checkpoint::types::{SecurityBlades, VersionInfo};
use crate::error::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Implementation function for version info parsing
///
/// # Errors
/// Returns a `CpinfoError` if the file cannot be read, or if the content does not match the expected format.
#[inline]
pub fn parse_version_info_impl<P: AsRef<Path>>(path: P) -> Result<VersionInfo> {
    let file_content = match fs::read_to_string(path) {
        Ok(content_string) => content_string,
        Err(error) => return Err(error.into()),
    };

    let version_regex = match Regex::new(r"Version:\s+(R\d+\.\d+)\s+-\s+Build\s+(\d+)") {
        Ok(regex) => regex,
        Err(regex_error) => {
            return Err(crate::error::CpinfoError::validation_error(format!(
                "Invalid version regex pattern: {regex_error}"
            )))
        }
    };
    let version_captures = match version_regex.captures(&file_content) {
        Some(captures) => captures,
        None => {
            return Err(crate::error::CpinfoError::validation_error(
                "Version information not found",
            ))
        }
    };

    let version = version_captures[1].to_string();
    let build = version_captures[2].to_string();

    let kernel_regex = match Regex::new(r"kernel:\s+(R\d+\.\d+)\s+-\s+Build\s+(\d+)") {
        Ok(regex) => regex,
        Err(kernel_error) => {
            return Err(crate::error::CpinfoError::validation_error(format!(
                "Invalid kernel regex pattern: {kernel_error}"
            )))
        }
    };
    let (kernel_version, kernel_build) =
        kernel_regex
            .captures(&file_content)
            .map_or((None, None), |kernel_captures| {
                return (
                    Some(kernel_captures[1].to_string()),
                    Some(kernel_captures[2].to_string()),
                );
            });

    return Ok(VersionInfo {
        build,
        kernel_build,
        kernel_version,
        version,
    });
}

/// Implementation function for security blades parsing
///
/// # Errors
/// Returns a `CpinfoError` if the file cannot be read, or parsing fails.
#[inline]
pub fn parse_security_blades_impl<P: AsRef<Path>>(path: P) -> Result<SecurityBlades> {
    let blade_content = match fs::read_to_string(path) {
        Ok(content_string) => content_string,
        Err(error) => return Err(error.into()),
    };

    let blades_regex = match Regex::new(r"Enabled blades\s*[-=]+\s*([^\r\n]+)") {
        Ok(regex) => regex,
        Err(blade_error) => {
            return Err(crate::error::CpinfoError::validation_error(format!(
                "Invalid blades regex pattern: {blade_error}"
            )))
        }
    };
    let blades_line = match blades_regex.captures(&blade_content) {
        Some(captures) => captures,
        None => {
            return Err(crate::error::CpinfoError::validation_error(
                "Enabled blades section not found",
            ))
        }
    };

    let blades_text = blades_line[1].trim();

    let mut blades = SecurityBlades::empty();

    if blades_text.contains("fw") {
        blades |= SecurityBlades::FIREWALL;
    }
    if blades_text.contains("vpn") {
        blades |= SecurityBlades::VPN;
    }
    if blades_text.contains("urlf") {
        blades |= SecurityBlades::URL_FILTERING;
    }
    if blades_text.contains("appi") {
        blades |= SecurityBlades::APPLICATION_CONTROL;
    }
    if blades_text.contains("ips") {
        blades |= SecurityBlades::IPS;
    }
    if blades_text.contains("identityServer") {
        blades |= SecurityBlades::IDENTITY_SERVER;
    }
    if blades_text.contains("mon") {
        blades |= SecurityBlades::MONITORING;
    }

    return Ok(blades);
}
