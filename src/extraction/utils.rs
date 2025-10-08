#[must_use]
#[inline]
pub fn categorize_section(section_name: &str) -> String {
    let name_lower = section_name.to_lowercase();

    if name_lower.contains("general") || name_lower.contains("information") {
        return "general".to_owned();
    } else if name_lower.contains("network") || name_lower.contains("interface") {
        return "network".to_owned();
    } else if name_lower.contains("security")
        || name_lower.contains("policy")
        || name_lower.contains("firewall")
        || name_lower.contains("ips")
    {
        return "security".to_owned();
    } else if name_lower.contains("vsx") || name_lower.contains("virtual") {
        return "vsx".to_owned();
    } else {
        return "misc".to_owned();
    }
}
