/// Parameters for creating an organized extraction result to reduce function argument count
#[derive(Debug)]
#[non_exhaustive]
pub struct OrganizedExtractionParams {
    /// Map of directories created for organization
    pub directories_created: Vec<std::path::PathBuf>,
    /// Output directory where sections were saved
    pub output_directory: std::path::PathBuf,
    /// List of created section files
    pub section_files: Vec<std::path::PathBuf>,
    /// Number of sections successfully extracted
    pub sections_extracted: usize,
    /// Number of virtual systems found (if VSX detected)
    pub virtual_systems_count: usize,
    /// Whether VSX (Virtual System Extension) was detected
    pub vsx_detected: bool,
}

/// Result of section extraction operation
#[derive(Debug)]
#[non_exhaustive]
pub struct ExtractionResult {
    /// Output directory where sections were saved
    pub output_directory: std::path::PathBuf,
    /// List of created section files
    pub section_files: Vec<std::path::PathBuf>,
    /// Number of sections successfully extracted
    pub sections_extracted: usize,
}

/// Result of organized section extraction with additional metadata
#[derive(Debug)]
#[non_exhaustive]
pub struct OrganizedExtractionResult {
    /// Map of directories created for organization
    pub directories_created: Vec<std::path::PathBuf>,
    /// Output directory where sections were saved
    pub output_directory: std::path::PathBuf,
    /// List of created section files
    pub section_files: Vec<std::path::PathBuf>,
    /// Number of sections successfully extracted
    pub sections_extracted: usize,
    /// Number of virtual systems found (if VSX detected)
    pub virtual_systems_count: usize,
    /// Whether VSX (Virtual System Extension) was detected
    pub vsx_detected: bool,
}

/// Result of partial recovery extraction with error recovery metadata
#[derive(Debug)]
#[non_exhaustive]
pub struct PartialRecoveryResult {
    /// Errors encountered during extraction
    pub errors: Vec<String>,
    /// Output directory where sections were saved
    pub output_directory: std::path::PathBuf,
    /// Number of partial/incomplete sections found
    pub partial_sections: usize,
    /// List of created section files
    pub section_files: Vec<std::path::PathBuf>,
    /// Number of sections successfully extracted
    pub sections_extracted: usize,
    /// Warnings about incomplete content
    pub warnings: Vec<String>,
}

/// Result of binary content detection extraction
#[derive(Debug)]
#[non_exhaustive]
pub struct BinaryDetectionResult {
    /// Number of sections containing binary content
    pub binary_sections_detected: usize,
    /// Errors encountered during extraction
    pub errors: Vec<String>,
    /// Output directory where sections were saved
    pub output_directory: std::path::PathBuf,
    /// List of created section files
    pub section_files: Vec<std::path::PathBuf>,
    /// Number of sections successfully extracted
    pub sections_extracted: usize,
    /// Warnings about binary content found
    pub warnings: Vec<String>,
}

impl ExtractionResult {
    /// Create a new extraction result
    #[inline]
    #[must_use]
    pub const fn new(
        sections_extracted: usize,
        output_directory: std::path::PathBuf,
        section_files: Vec<std::path::PathBuf>,
    ) -> Self {
        return Self {
            output_directory,
            section_files,
            sections_extracted,
        };
    }
}

impl OrganizedExtractionResult {
    /// Create a new organized extraction result using structured parameters
    #[inline]
    #[must_use]
    pub fn new(params: OrganizedExtractionParams) -> Self {
        return Self {
            directories_created: params.directories_created,
            output_directory: params.output_directory,
            section_files: params.section_files,
            sections_extracted: params.sections_extracted,
            virtual_systems_count: params.virtual_systems_count,
            vsx_detected: params.vsx_detected,
        };
    }
}
