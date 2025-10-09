//\! Tests for basic section extraction via SectionExtractor facade

use std::io::Write as _;
use tempfile::{NamedTempFile, tempdir};

use cpinfo_parser::SectionExtractor;

fn make_cpinfo_file() -> NamedTempFile {
    let mut f = NamedTempFile::with_suffix(".info").expect("temp file");
    // Two sections with canonical delimiter pattern
    writeln\!(f, "==============================================").unwrap();
    writeln\!(f, "System Information").unwrap();
    writeln\!(f, "==============================================").unwrap();
    writeln\!(f, "Product: Check Point Security Gateway").unwrap();
    writeln\!(f, "Version: R81.20").unwrap();
    writeln\!(f, "").unwrap();
    writeln\!(f, "==============================================").unwrap();
    writeln\!(f, "Network Configuration").unwrap();
    writeln\!(f, "==============================================").unwrap();
    writeln\!(f, "Interfaces: eth0, eth1").unwrap();
    writeln\!(f, "DNS: 8.8.8.8, 8.8.4.4").unwrap();
    writeln\!(f, "==============================================").unwrap();
    f.flush().unwrap();
    f
}

#[test]
fn extract_sections_creates_files_and_counts() {
    let input = make_cpinfo_file();
    let out_dir = tempdir().expect("out dir");

    let result = SectionExtractor::extract_sections(input.path(), out_dir.path())
        .expect("extraction should succeed");

    assert_eq\!(result.sections_extracted, 2, "should extract 2 sections");

    let sys = out_dir.path().join("System_Information.txt");
    let net = out_dir.path().join("Network_Configuration.txt");
    assert\!(sys.exists(), "system section file should exist");
    assert\!(net.exists(), "network section file should exist");

    let sys_content = std::fs::read_to_string(&sys).expect("read sys");
    assert\!(sys_content.contains("Product: Check Point Security Gateway"));
    assert\!(sys_content.contains("Version: R81.20"));

    let net_content = std::fs::read_to_string(&net).expect("read net");
    assert\!(net_content.contains("Interfaces: eth0, eth1"));
    assert\!(net_content.contains("DNS: 8.8.8.8, 8.8.4.4"));
}