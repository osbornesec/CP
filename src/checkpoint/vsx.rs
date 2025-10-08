use crate::checkpoint::types::{VirtualSystem, VsxDeployment};
use crate::error::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Implementation function for VSX deployment parsing
///
/// # Errors
/// Returns `CpinfoError` if the file cannot be read, or parsing fails.
#[inline]
pub fn parse_vsx_deployment_impl<P: AsRef<Path>>(path: P) -> Result<VsxDeployment> {
    let content = match fs::read_to_string(path) {
        Ok(file_content) => file_content,
        Err(error) => return Err(error.into()),
    };

    if !content.contains("Type: VSX Gateway") && !content.contains("VSX Enabled: true") {
        return Err(crate::error::CpinfoError::validation_error(
            "Not a VSX deployment",
        ));
    }

    let mut virtual_systems = Vec::new();

    let vs_regex = match Regex::new(
        r"VS (\d+) \(([^)]+)\)\s*[-=]+\s*Virtual System ID:\s*(\d+)\s*Context Type:\s*(\w+)\s*Name:\s*([^\r\n]+)\s*State:\s*(\w+)(?:\s*Interfaces:\s*([^\r\n]+))?",
    ) {
        Ok(regex_pattern) => regex_pattern,
        Err(regex_error) => {
            return Err(crate::error::CpinfoError::validation_error(format!(
                "Invalid VS regex pattern: {regex_error}"
            )));
        }
    };

    for captures in vs_regex.captures_iter(&content) {
        let id: u32 = captures[3].parse().unwrap_or(0);
        let context_type = captures[4].to_string();
        let name = captures[5].trim().to_owned();
        let state = captures[6].to_string();

        let interfaces = captures.get(7).map_or_else(
            || {
                return Vec::new();
            },
            |interfaces_str| {
                return interfaces_str
                    .as_str()
                    .split(',')
                    .map(|interface_string| {
                        return interface_string.trim().to_owned();
                    })
                    .filter(|interface_string| {
                        return !interface_string.is_empty();
                    })
                    .collect();
            },
        );

        virtual_systems.push(VirtualSystem {
            context_type,
            id,
            interfaces,
            name,
            state,
        });
    }

    if virtual_systems.is_empty() {
        return Err(crate::error::CpinfoError::validation_error(
            "No virtual systems found",
        ));
    }

    return Ok(VsxDeployment {
        deployment_type: "VSX".to_owned(),
        virtual_systems,
    });
}
