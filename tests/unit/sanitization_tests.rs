//\! Sanitization module unit tests
//\! 
//\! Comprehensive tests for filename and path sanitization functions.
//\! Tests cover happy paths, edge cases, special characters, and security concerns.

use cpinfo_parser::section_parser::sanitization::{
    command_output_filename, file_output_filename, sanitize_command_name, sanitize_file_path,
};

#[cfg(test)]
mod sanitize_file_path_tests {
    use super::*;

    #[test]
    fn should_preserve_alphanumeric_characters() {
        // Test: Alphanumeric characters should pass through unchanged
        let input = "abc123XYZ";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "abc123XYZ");
    }

    #[test]
    fn should_preserve_hyphens_and_underscores() {
        // Test: Hyphens and underscores should be preserved
        let input = "file-name_with_dashes";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "file-name_with_dashes");
    }

    #[test]
    fn should_preserve_dots() {
        // Test: Dots should be preserved (for extensions)
        let input = "file.name.txt";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "file.name.txt");
    }

    #[test]
    fn should_replace_forward_slashes_with_underscores() {
        // Test: Forward slashes should be replaced with underscores
        let input = "path/to/file";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "path_to_file");
    }

    #[test]
    fn should_replace_backslashes_with_underscores() {
        // Test: Backslashes should be replaced with underscores
        let input = r"path\to\file";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "path_to_file");
    }

    #[test]
    fn should_replace_spaces_with_underscores() {
        // Test: Spaces should be replaced with underscores
        let input = "file name with spaces";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "file_name_with_spaces");
    }

    #[test]
    fn should_replace_special_characters() {
        // Test: Special characters should be replaced with underscores
        let input = "file@name#with$special%chars";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "file_name_with_special_chars");
    }

    #[test]
    fn should_handle_absolute_unix_paths() {
        // Test: Absolute Unix paths should be sanitized
        let input = "/etc/hosts";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "_etc_hosts");
    }

    #[test]
    fn should_handle_absolute_windows_paths() {
        // Test: Absolute Windows paths should be sanitized
        let input = r"C:\Windows\System32\config";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "C__Windows_System32_config");
    }

    #[test]
    fn should_handle_paths_with_multiple_slashes() {
        // Test: Multiple consecutive slashes should each be replaced
        let input = "path//to///file";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "path__to___file");
    }

    #[test]
    fn should_handle_unicode_characters() {
        // Test: Unicode characters should be replaced with underscores
        let input = "файл名前.txt";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "______.txt");
    }

    #[test]
    fn should_handle_empty_string() {
        // Test: Empty string should return empty string
        let input = "";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "");
    }

    #[test]
    fn should_handle_string_with_only_invalid_chars() {
        // Test: String with only invalid characters
        let input = "\!@#$%^&*()";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "__________");
    }

    #[test]
    fn should_preserve_file_extensions_with_dots() {
        // Test: File extensions with dots should be preserved
        let input = "document.tar.gz";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "document.tar.gz");
    }

    #[test]
    fn should_handle_mixed_case_paths() {
        // Test: Mixed case should be preserved
        let input = "MyFile.TXT";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "MyFile.TXT");
    }

    #[test]
    fn should_handle_paths_with_colons() {
        // Test: Colons (common in timestamps) should be replaced
        let input = "log-2024:01:15.txt";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "log-2024_01_15.txt");
    }

    #[test]
    fn should_handle_paths_with_parentheses() {
        // Test: Parentheses should be replaced
        let input = "file(1).txt";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "file_1_.txt");
    }

    #[test]
    fn should_handle_paths_with_brackets() {
        // Test: Brackets should be replaced
        let input = "array[0].txt";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "array_0_.txt");
    }

    #[test]
    fn should_handle_long_paths() {
        // Test: Long paths should be fully sanitized
        let input = "/var/log/checkpoint/fw1/2024/01/15/messages.log";
        let result = sanitize_file_path(input);
        assert_eq\!(result, "_var_log_checkpoint_fw1_2024_01_15_messages.log");
    }
}

#[cfg(test)]
mod file_output_filename_tests {
    use super::*;

    #[test]
    fn should_add_txt_extension_to_path_without_extension() {
        // Test: Paths without .txt should get .txt appended
        let input = "/etc/hosts";
        let result = file_output_filename(input);
        assert\!(result.ends_with(".txt"));
        assert_eq\!(result, "_etc_hosts.txt");
    }

    #[test]
    fn should_not_duplicate_txt_extension() {
        // Test: Paths already ending in .txt should not get duplicate extension
        let input = "/var/log/messages.txt";
        let result = file_output_filename(input);
        // Count how many times .txt appears
        let txt_count = result.matches(".txt").count();
        assert_eq\!(txt_count, 1);
        assert_eq\!(result, "_var_log_messages.txt");
    }

    #[test]
    fn should_add_txt_extension_to_file_with_other_extension() {
        // Test: Files with non-.txt extensions should get .txt added
        let input = "/etc/config.conf";
        let result = file_output_filename(input);
        assert\!(result.ends_with(".txt"));
        assert_eq\!(result, "_etc_config.conf.txt");
    }

    #[test]
    fn should_handle_path_with_no_extension() {
        // Test: Path with no extension should get .txt
        let input = "/usr/bin/bash";
        let result = file_output_filename(input);
        assert_eq\!(result, "_usr_bin_bash.txt");
    }

    #[test]
    fn should_sanitize_and_add_extension() {
        // Test: Both sanitization and extension should be applied
        let input = "/path/with spaces/file@name";
        let result = file_output_filename(input);
        assert_eq\!(result, "_path_with_spaces_file_name.txt");
    }

    #[test]
    fn should_handle_windows_paths() {
        // Test: Windows paths should be sanitized and get .txt
        let input = r"C:\Windows\System32\drivers\etc\hosts";
        let result = file_output_filename(input);
        assert\!(result.ends_with(".txt"));
        assert_eq\!(result, "C__Windows_System32_drivers_etc_hosts.txt");
    }

    #[test]
    fn should_handle_multiple_dots_in_path() {
        // Test: Multiple dots should be preserved, .txt added if not present
        let input = "/var/log/my.log.1";
        let result = file_output_filename(input);
        assert_eq\!(result, "_var_log_my.log.1.txt");
    }

    #[test]
    fn should_handle_empty_string() {
        // Test: Empty string should result in just .txt
        let input = "";
        let result = file_output_filename(input);
        assert_eq\!(result, ".txt");
    }

    #[test]
    fn should_handle_path_ending_with_slash() {
        // Test: Path ending with slash (directory) should be sanitized
        let input = "/var/log/";
        let result = file_output_filename(input);
        assert_eq\!(result, "_var_log_.txt");
    }

    #[test]
    fn should_preserve_txt_in_middle_of_name() {
        // Test: .txt in middle of name should not prevent adding .txt at end
        let input = "/path/to/file.txt.backup";
        let result = file_output_filename(input);
        assert_eq\!(result, "_path_to_file.txt.backup.txt");
    }
}

#[cfg(test)]
mod command_output_filename_tests {
    use super::*;

    #[test]
    fn should_sanitize_simple_command() {
        // Test: Simple command should be sanitized with .txt
        let input = "ls -la";
        let result = command_output_filename(input);
        assert_eq\!(result, "ls_-la.txt");
    }

    #[test]
    fn should_sanitize_command_with_pipes() {
        // Test: Commands with pipes should have pipes replaced
        let input = "ps aux | grep process";
        let result = command_output_filename(input);
        assert\!(result.ends_with(".txt"));
        assert_eq\!(result, "ps_aux___grep_process.txt");
    }

    #[test]
    fn should_sanitize_command_with_redirection() {
        // Test: Commands with redirection should be sanitized
        let input = "cat file > output";
        let result = command_output_filename(input);
        assert_eq\!(result, "cat_file___output.txt");
    }

    #[test]
    fn should_handle_command_with_special_chars() {
        // Test: Special characters in commands should be replaced
        let input = "find . -name \"*.log\"";
        let result = command_output_filename(input);
        assert\!(result.ends_with(".txt"));
    }

    #[test]
    fn should_handle_empty_command() {
        // Test: Empty command string
        let input = "";
        let result = command_output_filename(input);
        assert_eq\!(result, ".txt");
    }

    #[test]
    fn should_handle_command_with_path() {
        // Test: Commands with full paths
        let input = "/usr/bin/cpinfo";
        let result = command_output_filename(input);
        assert_eq\!(result, "_usr_bin_cpinfo.txt");
    }

    #[test]
    fn should_handle_long_command_line() {
        // Test: Long command lines should be fully sanitized
        let input = "iptables -L -n -v --line-numbers";
        let result = command_output_filename(input);
        assert\!(result.ends_with(".txt"));
        assert_eq\!(result, "iptables_-L_-n_-v_--line-numbers.txt");
    }
}

#[cfg(test)]
mod sanitize_command_name_tests {
    use super::*;

    #[test]
    fn should_preserve_alphanumeric_in_commands() {
        // Test: Alphanumeric characters should be preserved
        let input = "cmd123";
        let result = sanitize_command_name(input);
        assert_eq\!(result, "cmd123");
    }

    #[test]
    fn should_preserve_hyphens_and_underscores_in_commands() {
        // Test: Hyphens and underscores should be preserved
        let input = "my-command_name";
        let result = sanitize_command_name(input);
        assert_eq\!(result, "my-command_name");
    }

    #[test]
    fn should_replace_spaces_in_commands() {
        // Test: Spaces in commands should be replaced
        let input = "ls -la /home";
        let result = sanitize_command_name(input);
        assert_eq\!(result, "ls_-la__home");
    }

    #[test]
    fn should_replace_special_chars_in_commands() {
        // Test: Special characters should be replaced
        let input = "cmd@host#123";
        let result = sanitize_command_name(input);
        assert_eq\!(result, "cmd_host_123");
    }

    #[test]
    fn should_handle_empty_command_name() {
        // Test: Empty string should return empty string
        let input = "";
        let result = sanitize_command_name(input);
        assert_eq\!(result, "");
    }

    #[test]
    fn should_handle_command_with_dots() {
        // Test: Dots should be replaced in command names
        let input = "script.sh";
        let result = sanitize_command_name(input);
        assert_eq\!(result, "script_sh");
    }
}

#[cfg(test)]
mod sanitization_security_tests {
    use super::*;

    #[test]
    fn should_prevent_directory_traversal_in_paths() {
        // Test: Directory traversal attempts should be neutralized
        let input = "../../../etc/passwd";
        let result = sanitize_file_path(input);
        assert\!(\!result.contains(".."));
        assert_eq\!(result, ".._.._.._etc_passwd");
    }

    #[test]
    fn should_prevent_null_bytes_in_paths() {
        // Test: Null bytes should be replaced
        let input = "file\0name";
        let result = sanitize_file_path(input);
        assert\!(\!result.contains('\0'));
        assert_eq\!(result, "file_name");
    }

    #[test]
    fn should_handle_extremely_long_paths() {
        // Test: Very long paths should be handled (stress test)
        let input = "a".repeat(1000);
        let result = sanitize_file_path(&input);
        assert_eq\!(result.len(), 1000); // All 'a's are valid
    }

    #[test]
    fn should_prevent_command_injection_in_filenames() {
        // Test: Command injection attempts should be neutralized
        let input = "file; rm -rf /";
        let result = sanitize_file_path(input);
        assert\!(\!result.contains(';'));
        assert_eq\!(result, "file__rm_-rf__");
    }

    #[test]
    fn should_prevent_path_injection_with_mixed_slashes() {
        // Test: Mixed slashes should all be replaced
        let input = r"path/to\file/test\end";
        let result = sanitize_file_path(input);
        assert\!(\!result.contains('/'));
        assert\!(\!result.contains('\\'));
        assert_eq\!(result, "path_to_file_test_end");
    }
}