//\! Organized extraction unit tests
//\! 
//\! Tests for section boundary detection and organized extraction logic.
//\! Covers edge cases in section parsing, empty sections, malformed delimiters, etc.

use cpinfo_parser::extraction::basic::SectionExtractor;
use std::fs;
use tempfile::tempdir;

#[cfg(test)]
mod section_boundary_detection_tests {
    use super::*;

    #[test]
    fn should_detect_section_with_valid_three_line_header() {
        // Test: Valid section with delimiter-name-delimiter structure
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let content = "\
Check Point Support Information

==============================================
Valid Section
==============================================
Section content here
More content

==============================================
Another Section
==============================================
More content
";
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        assert_eq\!(extraction.sections_extracted, 2);
    }

    #[test]
    fn should_reject_section_with_empty_name() {
        // Test: Section with empty name line should be rejected
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let content = "\
Check Point Support Information

==============================================

==============================================
This should not be extracted

==============================================
Valid Section
==============================================
This should be extracted
";
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        // Should only extract the valid section
        assert_eq\!(extraction.sections_extracted, 1);
    }

    #[test]
    fn should_reject_section_with_delimiter_as_name() {
        // Test: Section where name is the delimiter itself should be rejected
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let content = "\
Check Point Support Information

==============================================
==============================================
==============================================
This should not be extracted

==============================================
Valid Section
==============================================
This should be extracted
";
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        assert_eq\!(extraction.sections_extracted, 1);
    }

    #[test]
    fn should_reject_section_without_closing_delimiter() {
        // Test: Section without proper closing delimiter on line 3
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let content = "\
Check Point Support Information

==============================================
Invalid Section
Not a delimiter
This should not be extracted

==============================================
Valid Section
==============================================
This should be extracted
";
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        assert_eq\!(extraction.sections_extracted, 1);
    }

    #[test]
    fn should_handle_section_at_end_of_file() {
        // Test: Section that is the last content in file (no trailing delimiter)
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let content = "\
Check Point Support Information

==============================================
Last Section
==============================================
Final content without following section";
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        assert_eq\!(extraction.sections_extracted, 1);
        
        // Verify content was extracted
        let section_file = &extraction.section_files[0];
        let content = fs::read_to_string(section_file).expect("Failed to read section file");
        assert\!(content.contains("Final content"));
    }

    #[test]
    fn should_handle_section_exactly_at_eof_with_line_index() {
        // Test: Edge case where section ends exactly at line count
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let content = "\
Check Point Support Information

==============================================
Section One
==============================================
Content
==============================================
Section Two
==============================================";
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        // Section Two should be detected even though it has no content after the closing delimiter
        assert_eq\!(extraction.sections_extracted, 2);
    }

    #[test]
    fn should_correctly_detect_section_boundaries() {
        // Test: Multiple sections with content should have correct boundaries
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let content = "\
Check Point Support Information

==============================================
Section One
==============================================
Line 1 of section one
Line 2 of section one

==============================================
Section Two
==============================================
Line 1 of section two
Line 2 of section two

==============================================
Section Three
==============================================
Line 1 of section three
";
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        assert_eq\!(extraction.sections_extracted, 3);
        
        // Verify each section has its own content
        for (idx, section_file) in extraction.section_files.iter().enumerate() {
            let content = fs::read_to_string(section_file).expect("Failed to read section file");
            assert\!(content.contains(&format\!("section {}", match idx {
                0 => "one",
                1 => "two",
                2 => "three",
                _ => panic\!("Unexpected section"),
            })));
        }
    }

    #[test]
    fn should_not_confuse_delimiter_in_content_with_section_start() {
        // Test: Delimiter appearing in content should not start new section without valid header
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let delimiter = "=".repeat(46);
        let content = format\!("\
Check Point Support Information

{delim}
Section One
{delim}
Content line 1
{delim}
This is not a valid section name because next line is not a delimiter
Content continues
{delim}
Invalid section attempt
{delim}

{delim}
Section Two
{delim}
Content of section two
", delim = delimiter);
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        // Should extract both valid sections
        assert_eq\!(extraction.sections_extracted, 2);
    }

    #[test]
    fn should_handle_whitespace_in_section_names() {
        // Test: Section names with leading/trailing whitespace
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let content = "\
Check Point Support Information

==============================================
  Section With Spaces  
==============================================
Content here
";
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        assert_eq\!(extraction.sections_extracted, 1);
        
        // Section name should be trimmed in filename
        let section_file = &extraction.section_files[0];
        let filename = section_file.file_name().unwrap().to_str().unwrap();
        assert\!(filename.contains("Section_With_Spaces"));
    }

    #[test]
    fn should_handle_consecutive_sections_no_content() {
        // Test: Multiple sections with no content between them
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let content = "\
Check Point Support Information

==============================================
Empty Section One
==============================================
==============================================
Empty Section Two
==============================================
==============================================
Section With Content
==============================================
Some content
";
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        // All three sections should be detected
        assert_eq\!(extraction.sections_extracted, 3);
    }
}

#[cfg(test)]
mod organized_output_structure_tests {
    use super::*;

    #[test]
    fn should_create_sections_directory() {
        // Test: Should create a 'sections' directory in output path
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let content = "\
Check Point Support Information

==============================================
Test Section
==============================================
Content
";
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        
        // Verify sections directory exists
        let sections_dir = output_dir.path().join("sections");
        assert\!(sections_dir.exists());
        assert\!(sections_dir.is_dir());
    }

    #[test]
    fn should_place_sections_in_sections_directory() {
        // Test: All section files should be in sections/ subdirectory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let content = "\
Check Point Support Information

==============================================
Section One
==============================================
Content 1

==============================================
Section Two
==============================================
Content 2
";
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        
        // All section files should be under sections/
        for section_file in &extraction.section_files {
            let parent = section_file.parent().unwrap();
            assert_eq\!(parent.file_name().unwrap(), "sections");
        }
    }

    #[test]
    fn should_track_directories_created() {
        // Test: Should track which directories were created
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let content = "\
Check Point Support Information

==============================================
Test Section
==============================================
Content
";
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        
        // Should have created the sections directory
        assert\!(\!extraction.directories_created.is_empty());
        assert\!(extraction.directories_created.iter().any(|d| {
            d.file_name().map(|n| n == "sections").unwrap_or(false)
        }));
    }

    #[test]
    fn should_not_duplicate_sections_directory() {
        // Test: Should not create sections directory if it already exists
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let content = "\
Check Point Support Information

==============================================
Test Section
==============================================
Content
";
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        // Pre-create the sections directory
        let sections_dir = output_dir.path().join("sections");
        fs::create_dir(&sections_dir).expect("Failed to create sections dir");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        
        // directories_created should be empty since sections/ already existed
        assert\!(extraction.directories_created.is_empty());
    }
}

#[cfg(test)]
mod section_content_end_detection_tests {
    use super::*;

    #[test]
    fn should_find_next_section_as_content_end() {
        // Test: Content should end when next valid section header is found
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let content = "\
Check Point Support Information

==============================================
Section One
==============================================
Line 1
Line 2
==============================================
Section Two
==============================================
Line 3
";
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        
        // Read first section
        let section_one = fs::read_to_string(&extraction.section_files[0])
            .expect("Failed to read section one");
        
        // Section one should contain Line 1 and Line 2, but not Line 3
        assert\!(section_one.contains("Line 1"));
        assert\!(section_one.contains("Line 2"));
        assert\!(\!section_one.contains("Line 3"));
        
        // Section two should contain Line 3
        let section_two = fs::read_to_string(&extraction.section_files[1])
            .expect("Failed to read section two");
        assert\!(section_two.contains("Line 3"));
    }

    #[test]
    fn should_ignore_false_section_starts_in_content() {
        // Test: Incomplete section headers in content should not end section
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let delimiter = "=".repeat(46);
        let content = format\!("\
Check Point Support Information

{delim}
Section One
{delim}
Line 1
{delim}
Not a section because line 3 is wrong
Still section one content
{delim}
Section Two
{delim}
Line 2
", delim = delimiter);
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        
        // Should find both valid sections
        assert_eq\!(extraction.sections_extracted, 2);
        
        // Section one should include the false start
        let section_one = fs::read_to_string(&extraction.section_files[0])
            .expect("Failed to read section one");
        assert\!(section_one.contains("Not a section"));
        assert\!(section_one.contains("Still section one content"));
    }

    #[test]
    fn should_handle_delimiter_with_empty_lines_in_between() {
        // Test: Empty name or delimiter-as-name should not start section
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let delimiter = "=".repeat(46);
        let content = format\!("\
Check Point Support Information

{delim}
Section One
{delim}
Content line 1
{delim}

{delim}
Content line 2
{delim}
{delim}
{delim}
Content line 3
{delim}
Section Two
{delim}
Content line 4
", delim = delimiter);
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        assert_eq\!(extraction.sections_extracted, 2);
        
        // Section one should include all false starts
        let section_one = fs::read_to_string(&extraction.section_files[0])
            .expect("Failed to read section one");
        assert\!(section_one.contains("Content line 1"));
        assert\!(section_one.contains("Content line 2"));
        assert\!(section_one.contains("Content line 3"));
        assert\!(\!section_one.contains("Content line 4"));
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn should_handle_file_with_only_header_no_sections() {
        // Test: File with only Check Point header but no sections
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let content = "Check Point Support Information\n\nSome metadata\n";
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        assert_eq\!(extraction.sections_extracted, 0);
    }

    #[test]
    fn should_handle_file_with_only_delimiters() {
        // Test: File with only delimiter lines (no valid sections)
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let delimiter = "=".repeat(46);
        let content = format\!("{delim}\n{delim}\n{delim}\n{delim}\n", delim = delimiter);
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        assert_eq\!(extraction.sections_extracted, 0);
    }

    #[test]
    fn should_handle_very_long_section_names() {
        // Test: Very long section names should be handled
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let long_name = "A".repeat(500);
        let content = format\!("\
Check Point Support Information

==============================================
{name}
==============================================
Content
", name = long_name);
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        assert_eq\!(extraction.sections_extracted, 1);
    }

    #[test]
    fn should_handle_special_characters_in_section_names() {
        // Test: Special characters in section names should be sanitized
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.cpinfo");
        
        let content = "\
Check Point Support Information

==============================================
Section/With\\Special:Characters*
==============================================
Content
";
        
        fs::write(&file_path, content).expect("Failed to write test file");
        let output_dir = tempdir().expect("Failed to create output directory");
        
        let result = SectionExtractor::extract_sections_organized(&file_path, output_dir.path());
        
        assert\!(result.is_ok());
        let extraction = result.unwrap();
        assert_eq\!(extraction.sections_extracted, 1);
        
        // Filename should have special characters replaced
        let filename = extraction.section_files[0].file_name().unwrap().to_str().unwrap();
        assert\!(\!filename.contains('/'));
        assert\!(\!filename.contains('\\'));
        assert\!(\!filename.contains(':'));
        assert\!(\!filename.contains('*'));
    }
}