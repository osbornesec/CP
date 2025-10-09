//\! Unit tests for extraction::writer::config module

use cpinfo_parser::extraction::writer::config::{sanitize_filename, WriterConfig};

#[test]
fn test_writer_config_default() {
    let config = WriterConfig::default();
    assert_eq\!(config.buffer_size, 64 * 1024);
    assert\!(config.show_progress);
    assert_eq\!(config.progress_threshold, 10_000);
}

#[test]
fn test_writer_config_new() {
    let config = WriterConfig::new(128 * 1024, false, 5_000);
    assert_eq\!(config.buffer_size, 128 * 1024);
    assert\!(\!config.show_progress);
    assert_eq\!(config.progress_threshold, 5_000);
}

#[test]
fn test_writer_config_for_performance() {
    let config = WriterConfig::for_performance();
    assert_eq\!(config.buffer_size, 128 * 1024);
    assert\!(\!config.show_progress);
    assert_eq\!(config.progress_threshold, usize::MAX);
}

#[test]
fn test_writer_config_for_user_experience() {
    let config = WriterConfig::for_user_experience();
    assert_eq\!(config.buffer_size, 64 * 1024);
    assert\!(config.show_progress);
    assert_eq\!(config.progress_threshold, 1_000);
}

#[test]
fn test_sanitize_filename_normal() {
    assert_eq\!(sanitize_filename("Normal Name"), "Normal_Name");
}

#[test]
fn test_sanitize_filename_path_separators() {
    assert_eq\!(
        sanitize_filename("Path/With\\Separators"),
        "Path_With_Separators"
    );
}

#[test]
fn test_sanitize_filename_special_chars() {
    assert_eq\!(
        sanitize_filename("Special<>|?*\"Chars"),
        "Special______Chars"
    );
}

#[test]
fn test_sanitize_filename_colons() {
    assert_eq\!(sanitize_filename("Colon:In:Name"), "Colon_In_Name");
}

#[test]
fn test_sanitize_filename_mixed() {
    assert_eq\!(
        sanitize_filename("File/Name:With<Special>Chars|Test"),
        "File_Name_With_Special_Chars_Test"
    );
}

#[test]
fn test_sanitize_filename_empty() {
    assert_eq\!(sanitize_filename(""), "");
}

#[test]
fn test_sanitize_filename_only_special() {
    assert_eq\!(sanitize_filename("/:*?\"<>|"), "________");
}

#[test]
fn test_sanitize_filename_unicode() {
    assert_eq\!(sanitize_filename("日本語"), "日本語");
}

#[test]
fn test_sanitize_filename_spaces() {
    assert_eq\!(sanitize_filename("Multiple   Spaces"), "Multiple___Spaces");
}