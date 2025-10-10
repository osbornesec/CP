//! Section file parser unit tests
//! 
//! Tests for parsing individual section files that contain multiple commands 
//! and file contents delimited by specific dash patterns.
//!
//! Following Canon TDD methodology: Red → Green → Refactor

use cpinfo_parser::section_parser::{SectionDelimiterDetector, SectionDelimiterType};

#[cfg(test)]
mod section_delimiter_tests {
    use super::*;

    #[test]
    fn should_detect_exact_24_dash_command_delimiter() {
        // Test 1: Should detect exact 24-dash command delimiter pattern
        // Input: "------------------------" (exactly 24 dashes)
        // Expected: Some(SectionDelimiterType::Command24Dash)
        // Purpose: Primary command delimiter detection for section file parsing
        
        let detector = SectionDelimiterDetector::new();
        let input = "------------------------"; // exactly 24 dashes
        
        let result = detector.detect_section_delimiter(input);
        
        assert_eq!(result, Some(SectionDelimiterType::Command24Dash));
    }

    #[test]
    fn should_detect_exact_23_dash_command_delimiter() {
        // Test 2: Should detect exact 23-dash command delimiter pattern
        // Input: "-----------------------" (exactly 23 dashes)  
        // Expected: Some(SectionDelimiterType::Command23Dash)
        // Purpose: Alternative command delimiter detection for section file parsing
        
        let detector = SectionDelimiterDetector::new();
        let input = "-----------------------"; // exactly 23 dashes
        
        let result = detector.detect_section_delimiter(input);
        
        assert_eq!(result, Some(SectionDelimiterType::Command23Dash));
    }

    #[test]
    fn should_detect_exact_66_dash_file_delimiter() {
        // Test 3: Should detect exact 66-dash file delimiter pattern
        // Input: "------------------------------------------------------------------" (exactly 66 dashes)
        // Expected: Some(SectionDelimiterType::File66Dash)
        // Purpose: File content delimiter detection for section file parsing
        
        let detector = SectionDelimiterDetector::new();
        let input = "------------------------------------------------------------------"; // exactly 66 dashes
        
        let result = detector.detect_section_delimiter(input);
        
        assert_eq!(result, Some(SectionDelimiterType::File66Dash));
    }

    #[test]
    fn should_reject_22_dash_near_miss_pattern() {
        // Test 4: Should reject 22-dash near-miss pattern
        // Input: "----------------------" (22 dashes)
        // Expected: None
        // Purpose: Prevent false positives
        
        let detector = SectionDelimiterDetector::new();
        let input = "----------------------"; // 22 dashes
        
        let result = detector.detect_section_delimiter(input);
        
        assert_eq!(result, None);
    }

    #[test]
    fn should_reject_25_dash_near_miss_pattern() {
        // Test 5: Should reject 25-dash near-miss pattern
        // Input: "-------------------------" (25 dashes)
        // Expected: None
        // Purpose: Prevent false positives
        
        let detector = SectionDelimiterDetector::new();
        let input = "-------------------------"; // 25 dashes
        
        let result = detector.detect_section_delimiter(input);
        
        assert_eq!(result, None);
    }

    #[test]
    fn should_reject_65_dash_near_miss_pattern() {
        // Test 6: Should reject 65-dash near-miss pattern
        // Input: "-----------------------------------------------------------------" (65 dashes)
        // Expected: None
        // Purpose: Prevent false positives
        
        let detector = SectionDelimiterDetector::new();
        let input = "-----------------------------------------------------------------"; // 65 dashes
        
        let result = detector.detect_section_delimiter(input);
        
        assert_eq!(result, None);
    }

    #[test]
    fn should_detect_67_dash_file_delimiter() {
        // Test 7: Should detect 67-dash file delimiter pattern (variant of file output wrapper)
        // Input: "-------------------------------------------------------------------" (67 dashes)
        // Expected: Some(SectionDelimiterType::File66Dash)
        // Purpose: Accept cpinfo variants that include one additional dash in file delimiters
        
        let detector = SectionDelimiterDetector::new();
        let input = "-------------------------------------------------------------------"; // 67 dashes
        
        let result = detector.detect_section_delimiter(input);
        
        assert_eq!(result, Some(SectionDelimiterType::File66Dash));
    }

    #[test]
    fn should_handle_mixed_content_with_dashes() {
        // Test 8: Should handle mixed content with dashes
        // Input: "---command-name---"
        // Expected: None
        // Purpose: Reject invalid patterns
        
        let detector = SectionDelimiterDetector::new();
        let input = "---command-name---";
        
        let result = detector.detect_section_delimiter(input);
        
        assert_eq!(result, None);
    }
}

#[cfg(test)]
mod section_delimiter_extended_tests {
    use super::*;

    #[test]
    fn should_detect_68_dash_file_delimiter() {
        // Test: Should detect 68-dash file delimiter (another variant)
        // Input: 68 consecutive dashes
        // Expected: Some(SectionDelimiterType::File66Dash)
        // Purpose: Verify >= 66 dash detection works for various lengths
        
        let detector = SectionDelimiterDetector::new();
        let input = &"-".repeat(68);
        
        let result = detector.detect_section_delimiter(input);
        
        assert_eq\!(result, Some(SectionDelimiterType::File66Dash));
    }

    #[test]
    fn should_detect_100_dash_file_delimiter() {
        // Test: Should detect 100-dash file delimiter (extreme case)
        // Input: 100 consecutive dashes
        // Expected: Some(SectionDelimiterType::File66Dash)
        // Purpose: Verify >= 66 dash detection works for very long delimiters
        
        let detector = SectionDelimiterDetector::new();
        let input = &"-".repeat(100);
        
        let result = detector.detect_section_delimiter(input);
        
        assert_eq\!(result, Some(SectionDelimiterType::File66Dash));
    }

    #[test]
    fn should_handle_whitespace_before_delimiters() {
        // Test: Should handle whitespace before delimiters
        // Input: "  ------------------------" (spaces before 24 dashes)
        // Expected: Some(SectionDelimiterType::Command24Dash) because of trimming
        // Purpose: Verify whitespace trimming in detection
        
        let detector = SectionDelimiterDetector::new();
        let input = "  ------------------------";
        
        let result = detector.detect_section_delimiter(input);
        
        assert_eq\!(result, Some(SectionDelimiterType::Command24Dash));
    }

    #[test]
    fn should_handle_whitespace_after_delimiters() {
        // Test: Should handle whitespace after delimiters
        // Input: "------------------------  " (24 dashes followed by spaces)
        // Expected: Some(SectionDelimiterType::Command24Dash)
        // Purpose: Verify trailing whitespace trimming in detection
        
        let detector = SectionDelimiterDetector::new();
        let input = "------------------------  ";
        
        let result = detector.detect_section_delimiter(input);
        
        assert_eq\!(result, Some(SectionDelimiterType::Command24Dash));
    }

    #[test]
    fn should_reject_empty_string() {
        // Test: Should reject empty string
        // Input: ""
        // Expected: None
        // Purpose: Handle edge case of empty input
        
        let detector = SectionDelimiterDetector::new();
        let input = "";
        
        let result = detector.detect_section_delimiter(input);
        
        assert_eq\!(result, None);
    }

    #[test]
    fn should_reject_single_dash() {
        // Test: Should reject single dash
        // Input: "-"
        // Expected: None
        // Purpose: Verify minimum length requirement
        
        let detector = SectionDelimiterDetector::new();
        let input = "-";
        
        let result = detector.detect_section_delimiter(input);
        
        assert_eq\!(result, None);
    }

    #[test]
    fn should_reject_non_dash_characters() {
        // Test: Should reject strings with non-dash characters
        // Input: "=========================="
        // Expected: None
        // Purpose: Verify only dash characters are accepted
        
        let detector = SectionDelimiterDetector::new();
        let input = "==========================";
        
        let result = detector.detect_section_delimiter(input);
        
        assert_eq\!(result, None);
    }
}

#[cfg(test)]
mod section_parser_header_validation_tests {
    use cpinfo_parser::section_parser::SectionFileParser;

    #[test]
    fn should_parse_valid_command_section_with_24_dash() {
        // Test: Parse a valid command section with 24-dash delimiter
        let parser = SectionFileParser::new();
        let content = "------------------------\nls -la\n------------------------\nfile1.txt\nfile2.txt\n";
        
        let result = parser.parse_command_section(content);
        
        assert\!(result.is_ok());
        let section = result.unwrap();
        assert_eq\!(section.name, "ls -la");
        assert\!(section.content.contains("file1.txt"));
        assert\!(section.content.contains("file2.txt"));
    }

    #[test]
    fn should_parse_valid_command_section_with_23_dash() {
        // Test: Parse a valid command section with 23-dash delimiter
        let parser = SectionFileParser::new();
        let content = "-----------------------\nps aux\n-----------------------\nroot 1 0.0\n";
        
        let result = parser.parse_command_section(content);
        
        assert\!(result.is_ok());
        let section = result.unwrap();
        assert_eq\!(section.name, "ps aux");
        assert\!(section.content.contains("root 1 0.0"));
    }

    #[test]
    fn should_reject_command_section_with_empty_name() {
        // Test: Reject command section with empty name line
        let parser = SectionFileParser::new();
        let content = "------------------------\n\n------------------------\ncontent\n";
        
        let result = parser.parse_command_section(content);
        
        assert\!(result.is_err());
    }

    #[test]
    fn should_reject_command_section_with_mismatched_delimiters() {
        // Test: Reject command section where opening and closing delimiters don't match
        let parser = SectionFileParser::new();
        let content = "------------------------\ncommand\n-----------------------\ncontent\n";
        
        let result = parser.parse_command_section(content);
        
        assert\!(result.is_err());
    }

    #[test]
    fn should_parse_valid_file_section_with_66_dash() {
        // Test: Parse a valid file section with exactly 66 dashes
        let parser = SectionFileParser::new();
        let delimiter = "-".repeat(66);
        let content = format\!("{}\n/etc/hosts\n{}\n127.0.0.1 localhost\n", delimiter, delimiter);
        
        let result = parser.parse_file_section(&content);
        
        assert\!(result.is_ok());
        let section = result.unwrap();
        assert_eq\!(section.path, "/etc/hosts");
        assert\!(section.content.contains("127.0.0.1 localhost"));
    }

    #[test]
    fn should_parse_valid_file_section_with_67_dash() {
        // Test: Parse a valid file section with 67 dashes (>=66 variant)
        let parser = SectionFileParser::new();
        let delimiter = "-".repeat(67);
        let content = format\!("{}\n/var/log/messages\n{}\nlog entry 1\nlog entry 2\n", delimiter, delimiter);
        
        let result = parser.parse_file_section(&content);
        
        assert\!(result.is_ok());
        let section = result.unwrap();
        assert_eq\!(section.path, "/var/log/messages");
        assert\!(section.content.contains("log entry 1"));
        assert\!(section.content.contains("log entry 2"));
    }

    #[test]
    fn should_parse_valid_file_section_with_100_dash() {
        // Test: Parse a valid file section with 100 dashes
        let parser = SectionFileParser::new();
        let delimiter = "-".repeat(100);
        let content = format\!("{}\n/tmp/test.log\n{}\ntest data\n", delimiter, delimiter);
        
        let result = parser.parse_file_section(&content);
        
        assert\!(result.is_ok());
        let section = result.unwrap();
        assert_eq\!(section.path, "/tmp/test.log");
        assert\!(section.content.contains("test data"));
    }

    #[test]
    fn should_reject_file_section_with_empty_path() {
        // Test: Reject file section with empty path line
        let parser = SectionFileParser::new();
        let delimiter = "-".repeat(66);
        let content = format\!("{}\n\n{}\ncontent\n", delimiter, delimiter);
        
        let result = parser.parse_file_section(&content);
        
        assert\!(result.is_err());
    }

    #[test]
    fn should_reject_file_section_with_mismatched_delimiters() {
        // Test: Reject file section where delimiters don't match (one is < 66)
        let parser = SectionFileParser::new();
        let content = format\!("{}\n/path/to/file\n{}\ncontent\n", "-".repeat(66), "-".repeat(24));
        
        let result = parser.parse_file_section(&content);
        
        assert\!(result.is_err());
    }

    #[test]
    fn should_parse_section_file_with_multiple_commands_and_files() {
        // Test: Parse a section file containing both command and file sections
        let parser = SectionFileParser::new();
        let content = format\!(
            "------------------------\nls\n------------------------\nfile1\n\n{}\n/etc/passwd\n{}\nroot:x:0\n",
            "-".repeat(66), "-".repeat(66)
        );
        
        let result = parser.parse_section_file(&content);
        
        assert\!(result.is_ok());
        let (commands, files) = result.unwrap();
        assert_eq\!(commands.len(), 1);
        assert_eq\!(files.len(), 1);
        assert_eq\!(commands[0].name, "ls");
        assert_eq\!(files[0].path, "/etc/passwd");
    }

    #[test]
    fn should_skip_invalid_sections_in_mixed_content() {
        // Test: Skip invalid sections but parse valid ones
        let parser = SectionFileParser::new();
        let content = format\!(
            "------------------------\nvalid_cmd\n------------------------\noutput1\n\n{}\n\n{}\nbad section\n\n------------------------\nanother_cmd\n------------------------\noutput2\n",
            "-".repeat(66), "-".repeat(66)
        );
        
        let result = parser.parse_section_file(&content);
        
        assert\!(result.is_ok());
        let (commands, _files) = result.unwrap();
        // Should parse the two valid command sections
        assert_eq\!(commands.len(), 2);
        assert_eq\!(commands[0].name, "valid_cmd");
        assert_eq\!(commands[1].name, "another_cmd");
    }

    #[test]
    fn should_handle_command_section_at_end_of_file() {
        // Test: Handle command section that extends to end of file
        let parser = SectionFileParser::new();
        let content = "------------------------\nlast_command\n------------------------\nfinal output";
        
        let result = parser.parse_section_file(content);
        
        assert\!(result.is_ok());
        let (commands, _files) = result.unwrap();
        assert_eq\!(commands.len(), 1);
        assert_eq\!(commands[0].name, "last_command");
        assert\!(commands[0].content.contains("final output"));
    }

    #[test]
    fn should_handle_file_section_at_end_of_file() {
        // Test: Handle file section that extends to end of file
        let parser = SectionFileParser::new();
        let delimiter = "-".repeat(66);
        let content = format\!("{}\n/final/file\n{}\nlast content", delimiter, delimiter);
        
        let result = parser.parse_section_file(&content);
        
        assert\!(result.is_ok());
        let (_commands, files) = result.unwrap();
        assert_eq\!(files.len(), 1);
        assert_eq\!(files[0].path, "/final/file");
        assert\!(files[0].content.contains("last content"));
    }
}