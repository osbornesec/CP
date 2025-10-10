use crate::checkpoint::regex_utils::{extract_u32, require_capture};
use crate::checkpoint::types::{ClusterConfiguration, ClusterMember, PolicyRule, SecurityPolicies};
use crate::error::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Implementation function for security policies parsing
///
/// # Errors
/// Returns `CpinfoError` if the file cannot be read, regex compilation fails, or data parsing fails.
#[inline]
pub fn parse_security_policies_impl<P: AsRef<Path>>(path: P) -> Result<SecurityPolicies> {
    let content = match fs::read_to_string(path) {
        Ok(file_content) => file_content,
        Err(io_error) => return Err(io_error.into()),
    };

    let total_rules = match extract_u32(
        &content,
        r"Total Rules:\s*(\d+)",
        "total rules",
        "Total rules not found",
    ) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };
    let allow_rules = match extract_u32(
        &content,
        r"Allow Rules:\s*(\d+)",
        "allow rules",
        "Allow rules count not found",
    ) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };
    let drop_rules = match extract_u32(
        &content,
        r"Drop Rules:\s*(\d+)",
        "drop rules",
        "Drop rules count not found",
    ) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };

    let rules = match parse_policy_rules(&content) {
        Ok(parsed_rules) => parsed_rules,
        Err(error) => return Err(error),
    };

    return Ok(SecurityPolicies {
        allow_rules,
        drop_rules,
        rules,
        total_rules,
    });
}

#[allow(
    clippy::single_call_fn,
    reason = "Helper keeps policy rule parsing focused and testable"
)]
#[inline]
fn parse_policy_rules(content: &str) -> Result<Vec<PolicyRule>> {
    let rule_regex = match Regex::new(r"Rule \d+:\s*Name:\s*([^\r\n]+)\s*Action:\s*(\w+)\s*Source:\s*([^\r\n]+)\s*Destination:\s*([^\r\n]+)\s*Service:\s*([^\r\n]+)")
        .map_err(|rule_regex_error| {
            return crate::error::CpinfoError::validation_error(format!(
                "Invalid policy rule regex pattern: {rule_regex_error}"
            ));
        }) {
        Ok(regex_pattern) => regex_pattern,
        Err(regex_error) => return Err(regex_error),
    };

    return rule_regex
        .captures_iter(content)
        .map(|captures| {
            let name_match =
                match require_capture(&captures, 1, "Invalid policy rule capture: missing name") {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                };
            let action_match = match require_capture(
                &captures,
                2,
                "Invalid policy rule capture: missing action",
            ) {
                Ok(value) => value,
                Err(error) => return Err(error),
            };
            let source_match = match require_capture(
                &captures,
                3,
                "Invalid policy rule capture: missing source",
            ) {
                Ok(value) => value,
                Err(error) => return Err(error),
            };
            let destination_match = match require_capture(
                &captures,
                4,
                "Invalid policy rule capture: missing destination",
            ) {
                Ok(value) => value,
                Err(error) => return Err(error),
            };
            let service_match =
                match require_capture(&captures, 5, "Invalid policy rule capture: missing service")
                {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                };

            return Ok(PolicyRule {
                action: action_match.to_owned(),
                destination: destination_match.trim().to_owned(),
                name: name_match.trim().to_owned(),
                service: service_match.trim().to_owned(),
                source: source_match.trim().to_owned(),
            });
        })
        .collect();
}

/// Implementation function for cluster configuration parsing
///
/// # Errors
/// Returns `CpinfoError` if the file cannot be read, regex compilation fails, or data parsing fails.
#[inline]
pub fn parse_cluster_configuration_impl<P: AsRef<Path>>(path: P) -> Result<ClusterConfiguration> {
    let content = match fs::read_to_string(path) {
        Ok(file_content) => file_content,
        Err(io_error) => return Err(io_error.into()),
    };

    let cluster_type = match extract_cluster_type(&content) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };
    let member_count = match extract_member_count(&content) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };
    let local_member = match extract_local_member(&content) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };
    let remote_members = match extract_remote_members(&content, &local_member.name) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };

    return Ok(ClusterConfiguration {
        cluster_type,
        local_member,
        member_count,
        remote_members,
    });
}

/// Extract cluster type from cpinfo content
///
/// # Arguments
///
/// * `content` - Raw file content to parse
///
/// # Returns
///
/// Cluster type string on success
///
/// # Errors
///
/// Returns error if regex compilation fails or cluster type not found
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Helper function for splitting long parse_cluster_configuration_impl"
)]
fn extract_cluster_type(content: &str) -> Result<String> {
    let cluster_type_regex = match Regex::new(r"Cluster Type:\s*(\w+)").map_err(|regex_error| {
        return crate::error::CpinfoError::validation_error(format!(
            "Invalid cluster type regex pattern: {regex_error}"
        ));
    }) {
        Ok(regex_pattern) => regex_pattern,
        Err(regex_error) => return Err(regex_error),
    };
    let captures = match cluster_type_regex.captures(content) {
        Some(capture_match) => capture_match,
        None => {
            return Err(crate::error::CpinfoError::validation_error(
                "Cluster type not found",
            ))
        }
    };
    let cluster_type_match =
        match require_capture(&captures, 1, "Invalid cluster type capture: missing value") {
            Ok(value) => value,
            Err(error) => return Err(error),
        };
    return Ok(cluster_type_match.to_owned());
}

/// Extract member count from cpinfo content
///
/// # Arguments
///
/// * `content` - Raw file content to parse
///
/// # Returns
///
/// Member count as u32 on success
///
/// # Errors
///
/// Returns error if regex compilation fails or member count not found/invalid
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Helper function for splitting long parse_cluster_configuration_impl"
)]
fn extract_member_count(content: &str) -> Result<u32> {
    let member_count_regex = match Regex::new(r"Member Count:\s*(\d+)").map_err(|regex_error| {
        return crate::error::CpinfoError::validation_error(format!(
            "Invalid member count regex pattern: {regex_error}"
        ));
    }) {
        Ok(regex_pattern) => regex_pattern,
        Err(regex_error) => return Err(regex_error),
    };
    let captures = match member_count_regex.captures(content) {
        Some(capture_match) => capture_match,
        None => {
            return Err(crate::error::CpinfoError::validation_error(
                "Member count not found",
            ))
        }
    };
    let member_count_match =
        match require_capture(&captures, 1, "Invalid member count: missing capture") {
            Ok(value) => value,
            Err(error) => return Err(error),
        };
    let member_count: u32 = match member_count_match.trim().parse() {
        Ok(count_value) => count_value,
        Err(_parse_error) => {
            return Err(crate::error::CpinfoError::validation_error(
                "Invalid member count",
            ))
        }
    };
    return Ok(member_count);
}

/// Extract local cluster member information from cpinfo content
///
/// # Arguments
///
/// * `content` - Raw file content to parse
///
/// # Returns
///
/// `ClusterMember` struct with local member details on success
///
/// # Errors
///
/// Returns error if regex compilation fails or local member data not found
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Helper function for splitting long parse_cluster_configuration_impl"
)]
fn extract_local_member(content: &str) -> Result<ClusterMember> {
    let local_member_regex = match Regex::new(r"Local Member:\s*([^\r\n]+)\s*Local State:\s*(\w+)")
        .map_err(|regex_error| {
            return crate::error::CpinfoError::validation_error(format!(
                "Invalid local member regex pattern: {regex_error}"
            ));
        }) {
        Ok(regex_pattern) => regex_pattern,
        Err(regex_error) => return Err(regex_error),
    };
    let local_captures = match local_member_regex.captures(content) {
        Some(capture_match) => capture_match,
        None => {
            return Err(crate::error::CpinfoError::validation_error(
                "Local member not found",
            ))
        }
    };

    let local_member_name = match require_capture(
        &local_captures,
        1,
        "Invalid local member capture: missing name",
    ) {
        Ok(value) => value.trim().to_owned(),
        Err(error) => return Err(error),
    };
    let _local_member_state = match require_capture(
        &local_captures,
        2,
        "Invalid local member capture: missing state",
    ) {
        Ok(value) => value.to_owned(),
        Err(error) => return Err(error),
    };

    let member_regex = match Regex::new(&format!(
        r"Member \d+:\s*Name:\s*{}\s*State:\s*(\w+)\s*IP:\s*([^\r\n]+)\s*Priority:\s*(\d+)",
        regex::escape(&local_member_name)
    ))
    .map_err(|regex_error| {
        return crate::error::CpinfoError::validation_error(format!(
            "Invalid member regex pattern: {regex_error}"
        ));
    }) {
        Ok(regex_pattern) => regex_pattern,
        Err(regex_error) => return Err(regex_error),
    };
    let local_details = match member_regex.captures(content) {
        Some(capture_match) => capture_match,
        None => {
            return Err(crate::error::CpinfoError::validation_error(
                "Local member details not found",
            ))
        }
    };

    let local_state_details_match = match require_capture(
        &local_details,
        1,
        "Invalid local member details: missing state",
    ) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };
    let local_ip_match = match require_capture(
        &local_details,
        2,
        "Invalid local member details: missing IP",
    ) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };
    let local_priority_match = match require_capture(
        &local_details,
        3,
        "Invalid local member details: missing priority",
    ) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };

    let local_priority = match local_priority_match.trim().parse::<u32>() {
        Ok(priority) => priority,
        Err(_parse_error) => {
            return Err(crate::error::CpinfoError::validation_error(
                "Invalid local member priority",
            ))
        }
    };

    let local_member = ClusterMember {
        name: local_member_name,
        state: local_state_details_match.to_owned(),
        ip: local_ip_match.trim().to_owned(),
        priority: local_priority,
    };
    return Ok(local_member);
}

/// Extract remote cluster members from cpinfo content
///
/// # Arguments
///
/// * `content` - Raw file content to parse
/// * `local_member_name` - Name of local member to exclude from remote list
///
/// # Returns
///
/// Vector of `ClusterMember` structs for remote members on success
///
/// # Errors
///
/// Returns error if regex compilation fails
#[inline]
#[allow(
    clippy::single_call_fn,
    reason = "Helper function for splitting long parse_cluster_configuration_impl"
)]
fn extract_remote_members(content: &str, local_member_name: &str) -> Result<Vec<ClusterMember>> {
    let all_members_regex = match Regex::new(
        r"Member \d+:\s*Name:\s*([^\r\n]+)\s*State:\s*(\w+)\s*IP:\s*([^\r\n]+)\s*Priority:\s*(\d+)",
    )
    .map_err(|regex_error| {
        return crate::error::CpinfoError::validation_error(format!(
            "Invalid all members regex pattern: {regex_error}"
        ));
    }) {
        Ok(regex_pattern) => regex_pattern,
        Err(regex_error) => return Err(regex_error),
    };
    let mut remote_members = Vec::new();

    for captures in all_members_regex.captures_iter(content) {
        let name = match require_capture(&captures, 1, "Invalid member capture: missing name") {
            Ok(value) => value.trim().to_owned(),
            Err(error) => return Err(error),
        };
        if name != local_member_name {
            let state_match =
                match require_capture(&captures, 2, "Invalid member capture: missing state") {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                };
            let ip_match = match require_capture(&captures, 3, "Invalid member capture: missing IP")
            {
                Ok(value) => value,
                Err(error) => return Err(error),
            };
            let priority_match =
                match require_capture(&captures, 4, "Invalid member capture: missing priority") {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                };
            let member_priority = match priority_match.trim().parse::<u32>() {
                Ok(priority) => priority,
                Err(_parse_error) => {
                    return Err(crate::error::CpinfoError::validation_error(
                        "Invalid remote member priority",
                    ))
                }
            };

            remote_members.push(ClusterMember {
                name,
                state: state_match.to_owned(),
                ip: ip_match.trim().to_owned(),
                priority: member_priority,
            });
        }
    }
    return Ok(remote_members);
}
