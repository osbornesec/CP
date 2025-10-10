//\! Tests for WriterConfig and filename sanitization

use cpinfo_parser::extraction::writer::config::{WriterConfig, sanitize_filename};

#[test]
fn writer_config_defaults() {
    let cfg = WriterConfig::default();
    assert_eq\!(cfg.buffer_size, 64 * 1024);
    assert\!(cfg.show_progress);
    assert_eq\!(cfg.progress_threshold, 10_000);
}

#[test]
fn writer_config_presets() {
    let perf = WriterConfig::for_performance();
    assert_eq\!(perf.buffer_size, 128 * 1024);
    assert\!(\!perf.show_progress);
    assert_eq\!(perf.progress_threshold, usize::MAX);

    let ux = WriterConfig::for_user_experience();
    assert_eq\!(ux.buffer_size, 64 * 1024);
    assert\!(ux.show_progress);
    assert_eq\!(ux.progress_threshold, 1_000);
}

#[test]
fn sanitize_filename_replaces_problem_chars() {
    assert_eq\!(sanitize_filename("Normal Name"), "Normal_Name");
    assert_eq\!(sanitize_filename("Colon:In:Name"), "Colon_In_Name");
    assert_eq\!(sanitize_filename("Path/With/Separators"), "Path_With_Separators");
    assert_eq\!(sanitize_filename("Special<>|?*\"Chars"), "Special______Chars");
}