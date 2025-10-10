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
