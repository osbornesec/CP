use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "cpinfo-parser",
    version = crate::VERSION,
    about = "Parse and extract sections from Check Point cpinfo files",
        long_about = r####"Parse and extract sections from Check Point cpinfo files

      A high-performance streaming parser for Check Point diagnostic cpinfo files.
      Extracts sections while maintaining security controls and performance targets.

🚀 NEW DEFAULT BEHAVIOR - INTEGRATED WORKFLOW:
    Phase 1: Extract sections from cpinfo file → organized directories
    Phase 2: Automatically parse section files → individual commands/files
    Result: Complete output with both section files AND parsed content

💡 QUICK START:
    cpinfo-parser gateway.cpinfo output/
        → Extracts sections AND parses them automatically
        → Creates: output/sections/, output/commands/, output/files/

📖 BASIC EXAMPLES:
    Complete processing (DEFAULT):
        cpinfo-parser gateway.cpinfo output/

    Extract sections only:
        cpinfo-parser gateway.cpinfo output/ --extract-only

    Parse existing section file:
        cpinfo-parser --section-file output/sections/CP_Status.txt

    Analysis mode (no files created):
        cpinfo-parser gateway.cpinfo --read-only

👥 PROFESSIONAL WORKFLOWS:
    Network Administrator (Incident Response):
        cpinfo-parser incident.cpinfo output/ --progress --verbose
        → Real-time progress, detailed logging for quick analysis

    Security Engineer (Comprehensive Analysis):
        cpinfo-parser gateway.cpinfo output/ --security --progress
        → Security controls enabled, filtered sensitive data

    Support Engineer (TAC Submission):
        cpinfo-parser case.cpinfo output/ --extract-only
        → Section files only for selective TAC submission

    Enterprise Operations (Batch Processing):
        for file in *.cpinfo; do
                  cpinfo-parser '<filename>' 'output/<filename>/' --progress
        done

🔧 ADVANCED OPTIONS:
    Monitor progress: --progress
    Verbose output: --verbose
    Security mode: --security
    Extract only: --extract-only"####
)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "CLI arguments naturally require many boolean flags for different operational modes"
)]
#[non_exhaustive]
pub struct Args {
    /// Only extract command sections (ignore file sections)
    #[arg(
        long,
        help = "Only extract command sections when parsing section files"
    )]
    pub commands_only: bool,

    /// Extract sections only (skip automatic parsing phase)
    #[arg(
        long,
        help = "Only extract sections from cpinfo file, skip automatic section parsing"
    )]
    pub extract_only: bool,

    /// Only extract file sections (ignore command sections)
    #[arg(long, help = "Only extract file sections when parsing section files")]
    pub files_only: bool,

    /// Input file path (cpinfo file or section file)
    #[arg(value_name = "FILE")]
    pub input: PathBuf,

    /// Output directory for extracted sections
    #[arg(short, long, default_value = "output")]
    pub output: PathBuf,

    /// Enable progress reporting
    #[arg(long)]
    pub progress: bool,

    /// Read-only mode (no files created, analysis only)
    #[arg(long)]
    pub read_only: bool,

    /// Parse as section file instead of cpinfo file
    #[arg(
        long,
        help = r#"Parse individual section file (e.g., CP_Status.txt) instead of cpinfo file"#
    )]
    pub section_file: bool,

    /// Enable security controls (filter sensitive data)
    #[arg(long)]
    pub security: bool,

    /// Verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}
