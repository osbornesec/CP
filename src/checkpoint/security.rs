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

    let total_rules_regex = match Regex::new(r"Total Rules:\s*(\d+)").map_err(|regex_error| {
        return crate::error::CpinfoError::validation_error(format!(
            "Invalid total rules regex pattern: {regex_error}"
        ));
    }) {
        Ok(regex_pattern) => regex_pattern,
        Err(regex_error) => return Err(regex_error),
    };
    let total_rules: u32 = match total_rules_regex
        .captures(&content)
        .ok_or_else(|| return crate::error::CpinfoError::validation_error("Total rules not found"))
        .and_then(|captures| {
            return captures[1].parse().map_err(|_parse_error| {
                return crate::error::CpinfoError::validation_error("Invalid total rules count");
            });
        }) {
        Ok(rules_count) => rules_count,
        Err(count_error) => return Err(count_error),
    };

    let allow_rules_regex = match Regex::new(r"Allow Rules:\s*(\d+)").map_err(|regex_error| {
        return crate::error::CpinfoError::validation_error(format!(
            "Invalid allow rules regex pattern: {regex_error}"
        ));
    }) {
        Ok(regex_pattern) => regex_pattern,
        Err(regex_error) => return Err(regex_error),
    };
    let allow_rules: u32 = match allow_rules_regex
        .captures(&content)
        .ok_or_else(|| {
            return crate::error::CpinfoError::validation_error("Allow rules count not found");
        })
        .and_then(|captures| {
            return captures[1].parse().map_err(|_parse_error| {
                return crate::error::CpinfoError::validation_error("Invalid allow rules count");
            });
        }) {
        Ok(rules_count) => rules_count,
        Err(count_error) => return Err(count_error),
    };

    let drop_rules_regex = match Regex::new(r"Drop Rules:\s*(\d+)").map_err(|regex_error| {
        return crate::error::CpinfoError::validation_error(format!(
            "Invalid drop rules regex pattern: {regex_error}"
        ));
    }) {
        Ok(regex_pattern) => regex_pattern,
        Err(regex_error) => return Err(regex_error),
    };
    let drop_rules: u32 = match drop_rules_regex
        .captures(&content)
        .ok_or_else(|| {
            return crate::error::CpinfoError::validation_error("Drop rules count not found");
        })
        .and_then(|captures| {
            return captures[1].parse().map_err(|_parse_error| {
                return crate::error::CpinfoError::validation_error("Invalid drop rules count");
            });
        }) {
        Ok(rules_count) => rules_count,
        Err(count_error) => return Err(count_error),
    };

    let rule_regex = match Regex::new(r"Rule \d+:\s*Name:\s*([^\r\n]+)\s*Action:\s*(\w+)\s*Source:\s*([^\r\n]+)\s*Destination:\s*([^\r\n]+)\s*Service:\s*([^\r\n]+)")
            .map_err(|rule_regex_error| return crate::error::CpinfoError::validation_error(format!("Invalid rule regex pattern: {rule_regex_error}"))) {
        Ok(regex_pattern) => regex_pattern,
        Err(regex_error) => return Err(regex_error),
    };
    let mut rules = Vec::new();

    for captures in rule_regex.captures_iter(&content) {
        rules.push(PolicyRule {
            name: captures[1].trim().to_owned(),
            action: captures[2].to_owned(),
            source: captures[3].trim().to_owned(),
            destination: captures[4].trim().to_owned(),
            service: captures[5].trim().to_owned(),
        });
    }

    return Ok(SecurityPolicies {
        allow_rules,
        drop_rules,
        rules,
        total_rules,
    });
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
    let cluster_type = match cluster_type_regex
        .captures(content)
        .ok_or_else(|| return crate::error::CpinfoError::validation_error("Cluster type not found"))
    {
        Ok(capture_match) => capture_match[1].to_owned(),
        Err(capture_error) => return Err(capture_error),
    };
    return Ok(cluster_type);
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
    let member_count: u32 = match member_count_regex
        .captures(content)
        .ok_or_else(|| return crate::error::CpinfoError::validation_error("Member count not found"))
        .and_then(|captures| {
            return captures[1].parse().map_err(|_parse_error| {
                return crate::error::CpinfoError::validation_error("Invalid member count");
            });
        }) {
        Ok(count_value) => count_value,
        Err(count_error) => return Err(count_error),
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
    let local_captures = match local_member_regex
        .captures(content)
        .ok_or_else(|| return crate::error::CpinfoError::validation_error("Local member not found"))
    {
        Ok(capture_match) => capture_match,
        Err(capture_error) => return Err(capture_error),
    };

    let local_member_name = local_captures[1].trim().to_owned();
    let _local_member_state = local_captures[2].to_owned();

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
    let local_details = match member_regex.captures(content).ok_or_else(|| {
        return crate::error::CpinfoError::validation_error("Local member details not found");
    }) {
        Ok(capture_match) => capture_match,
        Err(capture_error) => return Err(capture_error),
    };

    let local_member = ClusterMember {
        name: local_member_name,
        state: local_details[1].to_owned(),
        ip: local_details[2].trim().to_owned(),
        priority: local_details[3].parse::<u32>().unwrap_or_default(),
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
        let name = captures[1].trim().to_owned();
        if name != local_member_name {
            remote_members.push(ClusterMember {
                name,
                state: captures[2].to_owned(),
                ip: captures[3].trim().to_owned(),
                priority: captures[4].parse::<u32>().unwrap_or_default(),
            });
        }
    }
    return Ok(remote_members);
}
