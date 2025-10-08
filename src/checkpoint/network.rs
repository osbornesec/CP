use crate::checkpoint::types::{
    NetworkConfiguration, NetworkInterface, VpnConfiguration, VpnTunnel,
};
use crate::error::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Implementation function for network interfaces parsing
///
/// # Errors
/// Returns `CpinfoError` if the file cannot be read, regex compilation fails, or data parsing fails.
#[inline]
pub fn extract_network_interfaces<P: AsRef<Path>>(path: P) -> Result<NetworkConfiguration> {
    let content = match fs::read_to_string(path) {
        Ok(file_content) => file_content,
        Err(io_error) => return Err(io_error.into()),
    };

    let total_regex = match Regex::new(r"Total Interfaces:\s*(\d+)") {
        Ok(regex) => regex,
        Err(regex_error) => {
            return Err(crate::error::CpinfoError::validation_error(format!(
                "Invalid total interfaces regex pattern: {regex_error}"
            )))
        }
    };
    let total_interfaces: u32 = match total_regex.captures(&content) {
        Some(captures) => match captures.get(1) {
            Some(matched) => match matched.as_str().parse() {
                Ok(count) => count,
                Err(_parse_error) => {
                    return Err(crate::error::CpinfoError::validation_error(
                        "Invalid interface count",
                    ))
                }
            },
            None => {
                return Err(crate::error::CpinfoError::validation_error(
                    "Missing total interfaces capture group",
                ))
            }
        },
        None => {
            return Err(crate::error::CpinfoError::validation_error(
                "Total interfaces not found",
            ))
        }
    };

    let interface_regex = match Regex::new(
        r"Interface:\s*([^\r\n]+)\s*IP Address:\s*([^\r\n]+)\s*Subnet Mask:\s*([^\r\n]+)\s*State:\s*(\w+)\s*MTU:\s*(\d+)",
    ) {
        Ok(regex) => regex,
        Err(regex_error) => {
            return Err(crate::error::CpinfoError::validation_error(format!(
                "Invalid interface regex pattern: {regex_error}"
            )))
        }
    };
    let mut interfaces = Vec::new();

    for captures in interface_regex.captures_iter(&content) {
        let name = match captures.get(1) {
            Some(matched) => matched.as_str().trim().to_owned(),
            None => {
                return Err(crate::error::CpinfoError::validation_error(
                    "Missing interface name capture group",
                ))
            }
        };
        let ip_address = match captures.get(2) {
            Some(matched) => matched.as_str().trim().to_owned(),
            None => {
                return Err(crate::error::CpinfoError::validation_error(
                    "Missing IP address capture group",
                ))
            }
        };
        let subnet_mask = match captures.get(3) {
            Some(matched) => matched.as_str().trim().to_owned(),
            None => {
                return Err(crate::error::CpinfoError::validation_error(
                    "Missing subnet mask capture group",
                ))
            }
        };
        let state = match captures.get(4) {
            Some(matched) => matched.as_str().to_owned(),
            None => {
                return Err(crate::error::CpinfoError::validation_error(
                    "Missing state capture group",
                ))
            }
        };
        let mtu = match captures.get(5) {
            Some(matched) => matched.as_str().parse().unwrap_or(1500),
            None => {
                return Err(crate::error::CpinfoError::validation_error(
                    "Missing MTU capture group",
                ))
            }
        };

        interfaces.push(NetworkInterface {
            ip_address,
            mtu,
            name,
            state,
            subnet_mask,
        });
    }

    return Ok(NetworkConfiguration {
        interfaces,
        total_interfaces,
    });
}

/// Implementation function for VPN configuration parsing
///
/// # Errors
/// Returns `CpinfoError` if the file cannot be read, regex compilation fails, or data parsing fails.
#[inline]
#[allow(
    clippy::too_many_lines,
    reason = "Complex VPN configuration parsing requires extensive match expressions for proper error handling and explicit return statements as per clippy restriction requirements"
)]
pub fn parse_vpn_configuration_impl<P: AsRef<Path>>(path: P) -> Result<VpnConfiguration> {
    let content = match fs::read_to_string(path) {
        Ok(file_content) => file_content,
        Err(io_error) => return Err(io_error.into()),
    };

    let total_regex = match Regex::new(r"Total Tunnels:\s*(\d+)") {
        Ok(regex) => regex,
        Err(regex_error) => {
            return Err(crate::error::CpinfoError::validation_error(format!(
                "Invalid total tunnels regex pattern: {regex_error}"
            )))
        }
    };
    let total_tunnels: u32 = match total_regex.captures(&content) {
        Some(captures) => match captures.get(1) {
            Some(matched) => match matched.as_str().parse() {
                Ok(count) => count,
                Err(_parse_error) => {
                    return Err(crate::error::CpinfoError::validation_error(
                        "Invalid tunnel count",
                    ))
                }
            },
            None => {
                return Err(crate::error::CpinfoError::validation_error(
                    "Missing total tunnels capture group",
                ))
            }
        },
        None => {
            return Err(crate::error::CpinfoError::validation_error(
                "Total tunnels not found",
            ))
        }
    };

    let active_regex = match Regex::new(r"Active Tunnels:\s*(\d+)") {
        Ok(regex) => regex,
        Err(regex_error) => {
            return Err(crate::error::CpinfoError::validation_error(format!(
                "Invalid active tunnels regex pattern: {regex_error}"
            )))
        }
    };
    let active_tunnels: u32 = match active_regex.captures(&content) {
        Some(captures) => match captures.get(1) {
            Some(matched) => match matched.as_str().parse() {
                Ok(count) => count,
                Err(_parse_error) => {
                    return Err(crate::error::CpinfoError::validation_error(
                        "Invalid active tunnel count",
                    ))
                }
            },
            None => {
                return Err(crate::error::CpinfoError::validation_error(
                    "Missing active tunnels capture group",
                ))
            }
        },
        None => {
            return Err(crate::error::CpinfoError::validation_error(
                "Active tunnels not found",
            ))
        }
    };

    let remote_access_regex = match Regex::new(r"Remote Access:\s*(Enabled|Disabled)") {
        Ok(regex) => regex,
        Err(regex_error) => {
            return Err(crate::error::CpinfoError::validation_error(format!(
                "Invalid remote access regex pattern: {regex_error}"
            )))
        }
    };
    let remote_access_enabled = remote_access_regex
        .captures(&content)
        .and_then(|captures| return captures.get(1))
        .is_some_and(|matched| return matched.as_str() == "Enabled");

    let tunnel_regex = match Regex::new(
        r"Tunnel:\s*([^\r\n]+)\s*Remote Peer:\s*([^\r\n]+)\s*Status:\s*(\w+)\s*Encryption:\s*([^\r\n]+)\s*Authentication:\s*([^\r\n]+)",
    ) {
        Ok(regex) => regex,
        Err(regex_error) => {
            return Err(crate::error::CpinfoError::validation_error(format!(
                "Invalid tunnel regex pattern: {regex_error}"
            )))
        }
    };
    let mut tunnels = Vec::new();

    for captures in tunnel_regex.captures_iter(&content) {
        let name = match captures.get(1) {
            Some(matched) => matched.as_str().trim().to_owned(),
            None => {
                return Err(crate::error::CpinfoError::validation_error(
                    "Missing tunnel name capture group",
                ))
            }
        };
        let remote_peer = match captures.get(2) {
            Some(matched) => matched.as_str().trim().to_owned(),
            None => {
                return Err(crate::error::CpinfoError::validation_error(
                    "Missing remote peer capture group",
                ))
            }
        };
        let status = match captures.get(3) {
            Some(matched) => matched.as_str().to_owned(),
            None => {
                return Err(crate::error::CpinfoError::validation_error(
                    "Missing status capture group",
                ))
            }
        };
        let encryption = match captures.get(4) {
            Some(matched) => matched.as_str().trim().to_owned(),
            None => {
                return Err(crate::error::CpinfoError::validation_error(
                    "Missing encryption capture group",
                ))
            }
        };
        let authentication = match captures.get(5) {
            Some(matched) => matched.as_str().trim().to_owned(),
            None => {
                return Err(crate::error::CpinfoError::validation_error(
                    "Missing authentication capture group",
                ))
            }
        };

        tunnels.push(VpnTunnel {
            authentication,
            encryption,
            name,
            remote_peer,
            status,
        });
    }

    return Ok(VpnConfiguration {
        active_tunnels,
        remote_access_enabled,
        total_tunnels,
        tunnels,
    });
}
