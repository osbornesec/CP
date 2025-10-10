use crate::error::{CpinfoError, Result};

/// Configuration for content sanitization operations.
///
/// This struct controls various limits and behaviors during sanitization
/// of file content and command outputs.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SanitizationConfig {
    /// Maximum file size in kilobytes before rejecting content
    pub max_file_size_kb: usize,
    /// Maximum length allowed for individual lines before truncation
    pub max_line_length: usize,
}

impl Default for SanitizationConfig {
    /// Creates a new `SanitizationConfig` with default values.
    ///
    /// # Returns
    ///
    /// A new configuration with reasonable default limits for sanitization.
    #[inline]
    fn default() -> Self {
        return Self {
            max_line_length: 1024,
            max_file_size_kb: 4096, // 4MB
        };
    }
}

/// Result of content sanitization operation.
///
/// Contains statistics about what modifications were made during sanitization.
#[derive(Debug)]
#[non_exhaustive]
pub struct SanitizationResult {
    /// Whether the entire file was truncated due to size limits
    pub file_truncated: bool,
    /// Number of lines that were truncated due to length limits
    pub lines_truncated: u32,
}

/// Sanitizes content by applying length and size limits.
///
/// This function processes input content to ensure it meets the configured
/// size and line length constraints, truncating lines that exceed limits.
///
/// # Arguments
///
/// * `content` - The raw content to sanitize
/// * `config` - Configuration specifying sanitization limits and behavior
///
/// # Returns
///
/// A tuple containing the sanitized content string and statistics about
/// modifications made during sanitization.
///
/// # Errors
///
/// Returns an error if the content exceeds the maximum file size limit.
#[inline]
pub fn sanitize_content(
    content: &str,
    config: &SanitizationConfig,
) -> Result<(String, SanitizationResult)> {
    let mut sanitized_content = String::new();
    let mut lines_truncated = 0;

    if content.len() > config.max_file_size_kb * 1024 {
        return Err(CpinfoError::validation_error("File exceeds size limit"));
    }

    for line in content.lines() {
        if line.len() > config.max_line_length {
            // Safe truncation respecting UTF-8 boundaries
            let truncated_line = match line.char_indices().nth(config.max_line_length) {
                Some((byte_index, _)) => line
                    .get(..byte_index)
                    .map_or(line, |safe_slice| return safe_slice),
                None => line, // Line is shorter than max_line_length chars
            };
            sanitized_content.push_str(truncated_line);
            sanitized_content.push('\n');
            lines_truncated += 1;
        } else {
            sanitized_content.push_str(line);
            sanitized_content.push('\n');
        }
    }

    return Ok((
        sanitized_content,
        SanitizationResult {
            lines_truncated,
            file_truncated: false, // For now, we don't truncate the whole file
        },
    ));
}

/// Generates a sanitized filename for command output.
///
/// Creates a safe filename by sanitizing the command name and adding
/// a `.txt` extension suitable for file system storage.
///
/// # Arguments
///
/// * `command_name` - The raw command name to sanitize
///
/// # Returns
///
/// A sanitized filename string suitable for file system use.
#[must_use]
#[inline]
pub fn command_output_filename(command_name: &str) -> String {
    return format!("{}.txt", sanitize_command_name(command_name));
}

/// Generates a sanitized filename from a file path.
///
/// The input path is converted to a filesystem-safe name; if the sanitized result does not already end with
/// `.txt`, the suffix `.txt` is appended.
///
/// # Examples
///
/// ```
/// assert_eq!(file_output_filename("dir/sub/file"), "dir_sub_file.txt");
/// assert_eq!(file_output_filename("notes.txt"), "notes.txt");
/// ```
#[must_use]
#[inline]
pub fn file_output_filename(file_path: &str) -> String {
    let sanitized = sanitize_file_path(file_path);
    if sanitized.ends_with(".txt") {
        return sanitized;
    }
    return format!("{sanitized}.txt");
}

/// Sanitizes a command name for safe file system use.
///
/// Replaces any characters that are not alphanumeric, hyphens, or underscores
/// with underscores to create a filename-safe string.
///
/// # Arguments
///
/// * `command` - The raw command name to sanitize
///
/// # Returns
///
/// A sanitized command name containing only safe characters.
#[must_use]
#[inline]
pub fn sanitize_command_name(command: &str) -> String {
    return command
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => return character,
            _ => return '_',
        })
        .collect::<String>();
}

/// Produce a filesystem-safe path by replacing or normalizing unsafe characters.
///
/// This function keeps ASCII letters, digits, hyphens (`-`), underscores (`_`), and dots (`.`) as-is;
/// it converts forward and backward slashes (`/`, `\`) to underscores (`_`); and it replaces any
/// other character with an underscore.
///
/// # Examples
///
/// ```
/// assert_eq!(sanitize_file_path("src/main.rs"), "src_main.rs");
/// assert_eq!(sanitize_file_path("dir/sub-dir/file.name"), "dir_sub-dir_file.name");
/// assert_eq!(sanitize_file_path("weird|name<>.txt"), "weird_name__.txt");
/// ```
#[must_use]
#[inline]
pub fn sanitize_file_path(path: &str) -> String {
    return path
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => character,
            '.' => '.',
            '/' | '\\' => '_',
            _ => '_',
        })
        .collect();
}