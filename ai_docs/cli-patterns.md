# CLI Design Patterns - cpinfo-parser

## Overview
- **Purpose**: Command-line interface design patterns and best practices
- **Use Cases**: Phase 1-2 CLI implementation, user experience optimization
- **Version**: clap 4.5+ with derive macros
- **Last Updated**: 2025-08-08

## Key Concepts

### CLI Philosophy for Enterprise Tools
The cpinfo-parser CLI follows **enterprise software principles** with **administrator-friendly design**:
- **Discoverability**: Self-documenting interfaces with comprehensive help
- **Consistency**: Predictable command patterns and option naming
- **Reliability**: Robust error handling with actionable feedback
- **Efficiency**: Streamlined workflows for common administrative tasks

### Design Principles
1. **Progressive Disclosure**: Simple defaults with advanced options available
2. **Error Prevention**: Input validation and confirmation for destructive operations
3. **Feedback**: Clear progress indication and result summaries
4. **Flexibility**: Multiple input formats and output options

## Implementation Patterns

### Primary Pattern: Hierarchical Command Structure

```rust
/// Comprehensive CLI structure with enterprise-grade features
use clap::{Parser, Subcommand, Args, ValueEnum};

#[derive(Parser)]
#[command(name = "cpinfo-parser")]
#[command(about = "Check Point diagnostic section parser for enterprise environments")]
#[command(long_about = r#"
High-performance parser for Check Point cpinfo diagnostic files with enterprise features.

Processes Check Point diagnostic output files to extract individual sections for analysis,
troubleshooting, and TAC submission. Supports VSX environments, batch processing,
and enterprise security features.

For detailed usage examples and enterprise deployment guidance, use:
    cpinfo-parser help examples
"#)]
#[command(version)]
#[command(author = "Check Point Systems Parser Team")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Global verbose output control
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,
    
    /// Configuration file path
    #[arg(short, long, global = true, value_name = "FILE")]
    #[arg(help = "Path to configuration file (TOML format)")]
    config: Option<PathBuf>,
    
    /// Enable enterprise mode for enhanced features
    #[arg(long, global = true)]
    #[arg(help = "Enable enterprise features (enhanced security, audit trails)")]
    enterprise: bool,
    
    /// Output format for structured data
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Human)]
    output_format: OutputFormat,
    
    /// Disable color output (for scripting)
    #[arg(long, global = true)]
    #[arg(help = "Disable colored output for scripting environments")]
    no_color: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract sections from diagnostic files
    #[command(name = "extract")]
    #[command(about = "Extract sections from Check Point diagnostic files")]
    #[command(long_about = r#"
Extract individual sections from Check Point cpinfo diagnostic files.

This command processes diagnostic files and creates organized output directories
containing extracted sections. Supports filtering, VSX context separation,
and various output formats for different workflows.

Examples:
    cpinfo-parser extract diagnostic.txt
    cpinfo-parser extract --vsx-aware --output ./extracted/ diagnostic.txt
    cpinfo-parser extract --filter fw,vpn --tac-format diagnostic.txt
"#)]
    Extract {
        /// Input diagnostic file or directory
        #[arg(value_name = "INPUT")]
        #[arg(help = "Path to cpinfo diagnostic file or directory containing multiple files")]
        input: PathBuf,
        
        #[command(flatten)]
        extract_options: ExtractOptions,
    },
    
    /// Validate diagnostic file format and integrity
    #[command(name = "validate")]
    #[command(about = "Validate diagnostic file format and completeness")]
    #[command(long_about = r#"
Validate Check Point diagnostic files for format correctness and completeness.

Performs comprehensive validation including format verification, section integrity
checks, and completeness analysis. Useful for troubleshooting collection issues
or preparing files for processing.

Examples:
    cpinfo-parser validate diagnostic.txt
    cpinfo-parser validate --detailed --check-completeness diagnostic.txt
"#)]
    Validate {
        /// Input file to validate
        #[arg(value_name = "INPUT")]
        input: PathBuf,
        
        /// Show detailed validation report
        #[arg(long)]
        #[arg(help = "Display comprehensive validation details")]
        detailed: bool,
        
        /// Check for missing expected sections
        #[arg(long)]
        #[arg(help = "Analyze completeness based on Check Point version")]
        check_completeness: bool,
        
        /// Validate against specific Check Point version
        #[arg(long, value_name = "VERSION")]
        #[arg(help = "Expected Check Point version (e.g., R81.10, R82)")]
        expected_version: Option<String>,
    },
    
    /// Process multiple files in batch mode
    #[command(name = "batch")]
    #[command(about = "Process multiple diagnostic files in parallel")]
    #[command(long_about = r#"
Process multiple Check Point diagnostic files in parallel with progress tracking.

Batch mode enables efficient processing of multiple files with parallel execution,
progress monitoring, and consolidated reporting. Ideal for processing diagnostic
collections from multiple gateways or time periods.

Examples:
    cpinfo-parser batch ./diagnostics/ --output ./processed/
    cpinfo-parser batch ./diagnostics/ --threads 8 --memory-limit 2GB
"#)]
    Batch {
        /// Input directory containing diagnostic files
        #[arg(value_name = "INPUT_DIR")]
        input_dir: PathBuf,
        
        /// Output base directory
        #[arg(short, long, value_name = "OUTPUT_DIR")]
        #[arg(help = "Base directory for organized output")]
        output_dir: Option<PathBuf>,
        
        /// Number of parallel processing threads
        #[arg(short, long, default_value = "4")]
        #[arg(help = "Number of files to process in parallel")]
        threads: usize,
        
        /// Maximum memory usage limit
        #[arg(long, value_parser = parse_memory_size)]
        #[arg(help = "Memory limit (e.g., 1GB, 512MB)")]
        memory_limit: Option<u64>,
        
        /// File pattern for input filtering
        #[arg(long, default_value = "*.txt,*.log")]
        #[arg(help = "File patterns to match (comma-separated)")]
        pattern: String,
        
        /// Continue processing after individual file failures
        #[arg(long)]
        #[arg(help = "Continue batch processing if individual files fail")]
        continue_on_error: bool,
    },
    
    /// Analyze security blade configuration and status
    #[command(name = "analyze")]
    #[command(about = "Analyze security blade configuration and performance")]
    #[command(long_about = r#"
Analyze Check Point security blade configuration, licensing, and performance.

Provides comprehensive analysis of security blade status, configuration issues,
performance metrics, and recommendations. Useful for health checks and
optimization planning.

Examples:
    cpinfo-parser analyze diagnostic.txt
    cpinfo-parser analyze --blade-focus ips,vpn --include-performance diagnostic.txt
"#)]
    Analyze {
        /// Input diagnostic file
        #[arg(value_name = "INPUT")]
        input: PathBuf,
        
        /// Focus analysis on specific security blades
        #[arg(long, value_delimiter = ',')]
        #[arg(help = "Security blades to focus on (fw,ips,vpn,etc.)")]
        blade_focus: Option<Vec<String>>,
        
        /// Include performance metrics analysis
        #[arg(long)]
        #[arg(help = "Include performance and resource utilization analysis")]
        include_performance: bool,
        
        /// Generate recommendations report
        #[arg(long)]
        #[arg(help = "Generate optimization and configuration recommendations")]
        recommendations: bool,
        
        /// Export analysis in machine-readable format
        #[arg(long, value_enum)]
        #[arg(help = "Export format for integration with other tools")]
        export_format: Option<ExportFormat>,
    },
    
    /// Create TAC submission package
    #[command(name = "tac-package")]
    #[command(about = "Prepare diagnostic data for TAC submission")]
    #[command(long_about = r#"
Create TAC (Technical Assistance Center) submission packages from diagnostic files.

Prepares diagnostic data for Check Point support submission by organizing,
filtering, and anonymizing content as needed. Creates multiple output formats
optimized for different TAC workflows.

Examples:
    cpinfo-parser tac-package --case-priority high diagnostic.txt
    cpinfo-parser tac-package --anonymize --issue-category connectivity diagnostic.txt
"#)]
    TacPackage {
        /// Input diagnostic file
        #[arg(value_name = "INPUT")]
        input: PathBuf,
        
        /// TAC case priority level
        #[arg(long, value_enum, default_value_t = CasePriority::Medium)]
        #[arg(help = "TAC case priority for content prioritization")]
        case_priority: CasePriority,
        
        /// Issue category for targeted extraction
        #[arg(long, value_enum)]
        #[arg(help = "Issue category to optimize section selection")]
        issue_category: Option<IssueCategory>,
        
        /// Anonymize sensitive data
        #[arg(long)]
        #[arg(help = "Anonymize IP addresses, hostnames, and other sensitive data")]
        anonymize: bool,
        
        /// Customer ID for case tracking
        #[arg(long, value_name = "CUSTOMER_ID")]
        #[arg(help = "Customer ID for TAC case tracking")]
        customer_id: Option<String>,
        
        /// Include diagnostic summary
        #[arg(long)]
        #[arg(help = "Generate comprehensive diagnostic summary for TAC engineers")]
        include_summary: bool,
        
        /// Output directory for TAC package
        #[arg(short, long, value_name = "OUTPUT_DIR")]
        #[arg(help = "Directory for TAC submission package")]
        output: Option<PathBuf>,
    },
    
    /// Interactive guided mode
    #[command(name = "interactive")]
    #[command(about = "Interactive mode with guided workflows")]
    #[command(long_about = r#"
Interactive mode provides guided workflows for common tasks.

Launches an interactive session that guides users through common operations
with step-by-step prompts, validation, and contextual help. Ideal for
infrequent users or complex multi-step operations.
"#)]
    Interactive,
    
    /// Show detailed examples and tutorials
    #[command(name = "examples")]
    #[command(about = "Show detailed usage examples and tutorials")]
    Examples {
        /// Show examples for specific workflow
        #[arg(value_enum)]
        #[arg(help = "Specific workflow examples to display")]
        workflow: Option<WorkflowType>,
    },
}

/// Extract command options with comprehensive configuration
#[derive(Args)]
struct ExtractOptions {
    /// Output directory for extracted sections
    #[arg(short, long, value_name = "DIR")]
    #[arg(help = "Output directory (default: auto-generated based on input file)")]
    output: Option<PathBuf>,
    
    /// Enable VSX-aware processing with context separation
    #[arg(long)]
    #[arg(help = "Enable VSX context detection and separation")]
    vsx_aware: bool,
    
    /// Filter sections by type or pattern
    #[arg(long, value_delimiter = ',')]
    #[arg(help = "Section filters (fw,vpn,ips,system,logs,advanced)")]
    #[arg(long_help = r#"Filter extracted sections by type or pattern.

Supported filters:
  fw          - Firewall related sections
  vpn         - VPN configuration and status
  ips         - Intrusion Prevention System
  system      - System configuration and status
  logs        - Log files and analysis
  advanced    - Advanced troubleshooting sections
  
Custom patterns also supported using glob syntax:
  --filter "*fw*,*vpn*"
"#)]
    filter: Option<Vec<String>>,
    
    /// Format output for TAC submission
    #[arg(long)]
    #[arg(help = "Organize output for Check Point TAC submission")]
    tac_format: bool,
    
    /// Preserve binary content integrity
    #[arg(long)]
    #[arg(help = "Preserve binary files without text conversion")]
    preserve_binary: bool,
    
    /// Memory limit for processing
    #[arg(long, value_parser = parse_memory_size, default_value = "500MB")]
    #[arg(help = "Memory usage limit (e.g., 500MB, 1GB)")]
    memory_limit: u64,
    
    /// Continue processing after individual section errors
    #[arg(long)]
    #[arg(help = "Continue extraction if individual sections fail")]
    continue_on_error: bool,
    
    /// Enable detailed progress reporting
    #[arg(long)]
    #[arg(help = "Show detailed progress including section-by-section status")]
    detailed_progress: bool,
    
    /// Dry run mode (validate without extracting)
    #[arg(long)]
    #[arg(help = "Validate file and show extraction plan without actually extracting")]
    dry_run: bool,
    
    /// Overwrite existing output directory
    #[arg(long)]
    #[arg(help = "Overwrite existing output directory (default: prompt for confirmation)")]
    overwrite: bool,
}

/// Value enums for structured options
#[derive(ValueEnum, Clone, Debug)]
enum OutputFormat {
    Human,
    Json,
    Yaml,
    Table,
}

#[derive(ValueEnum, Clone, Debug)]
enum CasePriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(ValueEnum, Clone, Debug)]  
enum IssueCategory {
    Connectivity,
    Performance,
    Security,
    Configuration,
    Upgrade,
    Other,
}

#[derive(ValueEnum, Clone, Debug)]
enum ExportFormat {
    Json,
    Csv,
    Excel,
    Xml,
}

#[derive(ValueEnum, Clone, Debug)]
enum WorkflowType {
    Basic,
    Vsx,
    Tac,
    Batch,
    Analysis,
}

/// Custom value parser for memory sizes
fn parse_memory_size(s: &str) -> Result<u64, String> {
    let s = s.to_lowercase();
    
    if s.ends_with("gb") {
        let num: f64 = s.trim_end_matches("gb").parse()
            .map_err(|_| format!("Invalid memory size: {}", s))?;
        Ok((num * 1024.0 * 1024.0 * 1024.0) as u64)
    } else if s.ends_with("mb") {
        let num: f64 = s.trim_end_matches("mb").parse()
            .map_err(|_| format!("Invalid memory size: {}", s))?;
        Ok((num * 1024.0 * 1024.0) as u64)
    } else if s.ends_with("kb") {
        let num: f64 = s.trim_end_matches("kb").parse()
            .map_err(|_| format!("Invalid memory size: {}", s))?;
        Ok((num * 1024.0) as u64)
    } else {
        // Assume bytes if no unit
        s.parse().map_err(|_| format!("Invalid memory size: {}", s))
    }
}
```

### Progress Reporting Pattern

```rust
/// Enterprise-grade progress reporting with multiple output modes
pub struct EnterpriseProgressReporter {
    multi_progress: MultiProgress,
    main_bar: ProgressBar,
    detail_bars: HashMap<String, ProgressBar>,
    start_time: Instant,
    config: ProgressConfig,
    statistics: ProgressStatistics,
}

#[derive(Debug, Clone)]
pub struct ProgressConfig {
    pub show_eta: bool,
    pub show_throughput: bool,
    pub show_memory_usage: bool,
    pub update_interval_ms: u64,
    pub detailed_mode: bool,
    pub quiet_mode: bool,
}

impl EnterpriseProgressReporter {
    pub fn new(total_work: u64, config: ProgressConfig) -> Self {
        let multi_progress = MultiProgress::new();
        
        // Main progress bar with enterprise styling
        let main_bar = multi_progress.add(ProgressBar::new(total_work));
        main_bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}"
            )
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  ")
        );
        
        Self {
            multi_progress,
            main_bar,
            detail_bars: HashMap::new(),
            start_time: Instant::now(),
            config,
            statistics: ProgressStatistics::new(),
        }
    }
    
    /// Report progress with contextual information
    pub fn report_progress(&mut self, progress: ProgressUpdate) {
        match progress {
            ProgressUpdate::FileStarted { file_name, file_size } => {
                if self.config.detailed_mode {
                    let detail_bar = self.multi_progress.add(
                        ProgressBar::new(file_size)
                    );
                    detail_bar.set_style(
                        ProgressStyle::with_template(
                            "  └─ {bar:30.green/yellow} {bytes:>10}/{total_bytes:10} {bytes_per_sec:>12} {msg}"
                        )
                        .unwrap()
                        .progress_chars("█▉▊▋▌▍▎▏  ")
                    );
                    detail_bar.set_message(format!("Processing: {}", file_name));
                    
                    self.detail_bars.insert(file_name.clone(), detail_bar);
                }
                
                self.statistics.files_started += 1;
            }
            
            ProgressUpdate::FileProgress { file_name, bytes_processed, sections_found } => {
                if let Some(detail_bar) = self.detail_bars.get(&file_name) {
                    detail_bar.set_position(bytes_processed);
                    
                    if self.config.show_memory_usage {
                        let memory_mb = get_current_memory_usage_mb().unwrap_or(0);
                        detail_bar.set_message(format!(
                            "Processing: {} | {} sections | {}MB RAM",
                            file_name, sections_found, memory_mb
                        ));
                    }
                }
            }
            
            ProgressUpdate::FileCompleted { file_name, sections_extracted, processing_time } => {
                if let Some(detail_bar) = self.detail_bars.remove(&file_name) {
                    detail_bar.finish_with_message(format!(
                        "✓ {} ({} sections, {:.1}s)",
                        file_name, sections_extracted, processing_time.as_secs_f32()
                    ));
                }
                
                self.main_bar.inc(1);
                self.statistics.files_completed += 1;
                self.statistics.total_sections += sections_extracted;
                
                // Update main bar message with summary
                let avg_time = self.start_time.elapsed().as_secs_f64() / self.statistics.files_completed as f64;
                let throughput = self.statistics.total_bytes_processed as f64 / self.start_time.elapsed().as_secs_f64() / 1024.0 / 1024.0;
                
                self.main_bar.set_message(format!(
                    "Completed: {} files, {} sections, {:.1} MB/s",
                    self.statistics.files_completed,
                    self.statistics.total_sections,
                    throughput
                ));
            }
            
            ProgressUpdate::Error { file_name, error } => {
                if let Some(detail_bar) = self.detail_bars.remove(&file_name) {
                    detail_bar.abandon_with_message(format!(
                        "✗ {} - Error: {}",
                        file_name, error
                    ));
                }
                
                self.statistics.files_failed += 1;
                self.main_bar.inc(1); // Still count as processed
            }
        }
    }
    
    /// Complete progress reporting with final summary
    pub fn complete(self) -> ProgressSummary {
        let total_time = self.start_time.elapsed();
        let success_rate = (self.statistics.files_completed as f64 / 
                          (self.statistics.files_completed + self.statistics.files_failed) as f64) * 100.0;
        
        self.main_bar.finish_with_message(format!(
            "✓ Processing complete - {:.1}% success rate in {:.1}s",
            success_rate, total_time.as_secs_f32()
        ));
        
        ProgressSummary {
            total_time,
            files_processed: self.statistics.files_completed + self.statistics.files_failed,
            files_successful: self.statistics.files_completed,
            files_failed: self.statistics.files_failed,
            sections_extracted: self.statistics.total_sections,
            average_throughput_mbps: self.statistics.total_bytes_processed as f64 / total_time.as_secs_f64() / 1024.0 / 1024.0,
            success_rate,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ProgressUpdate {
    FileStarted { file_name: String, file_size: u64 },
    FileProgress { file_name: String, bytes_processed: u64, sections_found: usize },
    FileCompleted { file_name: String, sections_extracted: usize, processing_time: Duration },
    Error { file_name: String, error: String },
}
```

### Interactive Mode Pattern

```rust
/// Interactive CLI mode with guided workflows
pub struct InteractiveMode {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    input_handler: InputHandler,
    workflow_engine: WorkflowEngine,
    help_system: ContextualHelpSystem,
}

impl InteractiveMode {
    pub async fn run(&mut self) -> Result<(), InteractiveError> {
        self.show_welcome_screen()?;
        
        loop {
            let user_choice = self.show_main_menu().await?;
            
            match user_choice {
                MainMenuChoice::ExtractSingle => {
                    self.guided_single_extraction().await?;
                }
                MainMenuChoice::BatchProcess => {
                    self.guided_batch_processing().await?;
                }
                MainMenuChoice::TacPackage => {
                    self.guided_tac_package_creation().await?;
                }
                MainMenuChoice::Analysis => {
                    self.guided_analysis().await?;
                }
                MainMenuChoice::Help => {
                    self.show_contextual_help().await?;
                }
                MainMenuChoice::Exit => {
                    break;
                }
            }
        }
        
        self.show_goodbye_message()?;
        Ok(())
    }
    
    /// Guided single file extraction workflow
    async fn guided_single_extraction(&mut self) -> Result<(), InteractiveError> {
        println!("\n🔍 Single File Extraction Wizard");
        println!("This wizard will guide you through extracting sections from a diagnostic file.\n");
        
        // Step 1: File selection with validation
        let input_file = loop {
            let file_path = self.prompt_for_file_path("Enter path to diagnostic file:")?;
            
            match self.validate_input_file(&file_path).await {
                Ok(file_info) => {
                    println!("✓ File validated: {} ({} MB)", 
                        file_info.display_name, 
                        file_info.size_mb
                    );
                    break file_path;
                }
                Err(e) => {
                    println!("✗ File validation failed: {}", e);
                    if !self.ask_yes_no("Try another file?")? {
                        return Ok(());
                    }
                }
            }
        };
        
        // Step 2: Output directory with suggestion
        let output_dir = self.prompt_for_output_directory(&input_file)?;
        
        // Step 3: Processing options
        let options = self.configure_extraction_options().await?;
        
        // Step 4: Confirmation and execution
        self.show_extraction_summary(&input_file, &output_dir, &options)?;
        
        if self.ask_yes_no("Proceed with extraction?")? {
            let results = self.execute_extraction(input_file, output_dir, options).await?;
            self.show_results_summary(&results)?;
        }
        
        Ok(())
    }
    
    /// Configure extraction options through guided prompts
    async fn configure_extraction_options(&mut self) -> Result<ExtractOptions, InteractiveError> {
        let mut options = ExtractOptions::default();
        
        println!("\n⚙️  Configuration Options");
        
        // VSX awareness
        if self.ask_yes_no("Enable VSX-aware processing? (Recommended for VSX environments)")? {
            options.vsx_aware = true;
        }
        
        // Section filtering
        if self.ask_yes_no("Filter specific section types?")? {
            let available_filters = vec![
                ("fw", "Firewall configurations and status"),
                ("vpn", "VPN configurations and tunnels"),
                ("ips", "Intrusion Prevention System"),
                ("system", "System configuration and status"),
                ("logs", "Log files and analysis"),
                ("advanced", "Advanced troubleshooting sections"),
            ];
            
            println!("\nAvailable section filters:");
            for (filter, description) in &available_filters {
                println!("  {} - {}", filter, description);
            }
            
            let filter_input = self.prompt_for_input("Enter filters (comma-separated, or 'all' for no filtering):")?;
            
            if filter_input.trim().to_lowercase() != "all" {
                options.filter = Some(filter_input.split(',')
                    .map(|s| s.trim().to_string())
                    .collect());
            }
        }
        
        // Memory limit
        let memory_limit = self.prompt_for_memory_limit()?;
        options.memory_limit = memory_limit;
        
        // Progress detail level
        if self.ask_yes_no("Show detailed progress information?")? {
            options.detailed_progress = true;
        }
        
        Ok(options)
    }
    
    /// Show extraction summary before execution
    fn show_extraction_summary(
        &self, 
        input_file: &Path, 
        output_dir: &Path, 
        options: &ExtractOptions
    ) -> Result<(), InteractiveError> {
        println!("\n📋 Extraction Summary");
        println!("────────────────────────────────");
        println!("Input File:    {}", input_file.display());
        println!("Output Dir:    {}", output_dir.display());
        println!("VSX Aware:     {}", if options.vsx_aware { "Yes" } else { "No" });
        
        if let Some(filters) = &options.filter {
            println!("Filters:       {}", filters.join(", "));
        } else {
            println!("Filters:       All sections");
        }
        
        println!("Memory Limit:  {} MB", options.memory_limit / 1024 / 1024);
        println!("Progress Mode: {}", if options.detailed_progress { "Detailed" } else { "Summary" });
        println!();
        
        Ok(())
    }
    
    /// Prompt for file path with autocomplete and validation
    fn prompt_for_file_path(&mut self, prompt: &str) -> Result<PathBuf, InteractiveError> {
        loop {
            print!("{} ", prompt);
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            if input.is_empty() {
                println!("Please enter a file path.");
                continue;
            }
            
            // Handle common path expansions
            let expanded_path = if input.starts_with("~/") {
                dirs::home_dir()
                    .ok_or_else(|| InteractiveError::HomeDirectoryNotFound)?
                    .join(&input[2..])
            } else {
                PathBuf::from(input)
            };
            
            return Ok(expanded_path);
        }
    }
    
    /// Yes/no confirmation with clear default handling
    fn ask_yes_no(&mut self, question: &str) -> Result<bool, InteractiveError> {
        loop {
            print!("{} (y/n): ", question);
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();
            
            match input.as_str() {
                "y" | "yes" => return Ok(true),
                "n" | "no" => return Ok(false),
                "" => {
                    println!("Please enter 'y' for yes or 'n' for no.");
                }
                _ => {
                    println!("Invalid input. Please enter 'y' for yes or 'n' for no.");
                }
            }
        }
    }
    
    /// Contextual help system with examples
    async fn show_contextual_help(&mut self) -> Result<(), InteractiveError> {
        println!("\n📚 Contextual Help System");
        
        let help_topics = vec![
            ("basic", "Basic extraction workflows"),
            ("vsx", "VSX environment processing"),
            ("tac", "TAC package preparation"),
            ("batch", "Batch processing multiple files"),
            ("analysis", "Security blade analysis"),
            ("troubleshooting", "Common issues and solutions"),
        ];
        
        println!("\nAvailable help topics:");
        for (topic, description) in &help_topics {
            println!("  {} - {}", topic, description);
        }
        
        let topic = self.prompt_for_input("\nEnter help topic (or 'menu' to return):")?;
        
        match topic.as_str() {
            "basic" => self.show_basic_help()?,
            "vsx" => self.show_vsx_help()?,
            "tac" => self.show_tac_help()?,
            "batch" => self.show_batch_help()?,
            "analysis" => self.show_analysis_help()?,
            "troubleshooting" => self.show_troubleshooting_help()?,
            "menu" => return Ok(()),
            _ => {
                println!("Unknown help topic: {}", topic);
            }
        }
        
        self.wait_for_keypress()?;
        Ok(())
    }
}
```

## Common Gotchas

### Critical Gotcha: Argument Parsing Edge Cases
- **Problem**: CLI arguments not parsed correctly for complex scenarios
- **Cause**: Insufficient validation or conflicting argument patterns
- **Solution**: Comprehensive argument validation and conflict resolution
- **Example**:
```rust
// WRONG: No validation or conflict handling
#[derive(Args)]
struct ExtractOptions {
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    #[arg(long)]
    dry_run: bool,
    
    #[arg(long)]
    overwrite: bool, // Conflicts with dry_run
}

// CORRECT: Validation and conflict resolution
#[derive(Args)]
struct ExtractOptions {
    #[arg(short, long)]
    #[arg(help = "Output directory (created if it doesn't exist)")]
    output: Option<PathBuf>,
    
    #[arg(long)]
    #[arg(help = "Show what would be extracted without actually extracting")]
    #[arg(conflicts_with = "overwrite")]
    dry_run: bool,
    
    #[arg(long)]
    #[arg(help = "Overwrite existing output directory without confirmation")]
    #[arg(conflicts_with = "dry_run")]
    overwrite: bool,
    
    #[arg(long, value_parser = validate_memory_size)]
    #[arg(help = "Memory limit (e.g., 500MB, 1GB)")]
    memory_limit: Option<u64>,
}

fn validate_memory_size(s: &str) -> Result<u64, String> {
    parse_memory_size(s).and_then(|size| {
        if size < 64 * 1024 * 1024 { // 64MB minimum
            Err("Memory limit must be at least 64MB".to_string())
        } else if size > 32 * 1024 * 1024 * 1024 { // 32GB maximum
            Err("Memory limit cannot exceed 32GB".to_string())
        } else {
            Ok(size)
        }
    })
}
```

### User Experience Gotcha: Poor Error Messages
- **Problem**: Cryptic error messages that don't help users fix issues
- **Cause**: Generic error handling without user-friendly context
- **Solution**: Contextual error messages with specific guidance
- **Example**:
```rust
// WRONG: Generic unhelpful errors
pub fn process_file(path: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::open(path)?; // Generic I/O error
    let content = read_file_content(file)?; // Could be encoding, size, etc.
    process_content(&content)?; // Could be format, parsing, etc.
    Ok(())
}

// CORRECT: User-friendly contextual errors
pub fn process_file(path: &Path) -> Result<(), ProcessingError> {
    // File access with specific guidance
    let file = File::open(path)
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => ProcessingError::FileNotFound {
                path: path.to_owned(),
                suggestion: format!(
                    "Verify the file path is correct: {}",
                    path.display()
                ),
            },
            ErrorKind::PermissionDenied => ProcessingError::PermissionDenied {
                path: path.to_owned(),
                suggestion: "Try running with elevated privileges or check file permissions".to_string(),
            },
            _ => ProcessingError::FileAccess {
                path: path.to_owned(),
                source: e,
                suggestion: "Check if the file is in use by another program".to_string(),
            }
        })?;
    
    // Content reading with size and encoding context
    let metadata = file.metadata()
        .map_err(|e| ProcessingError::FileAccess {
            path: path.to_owned(),
            source: e,
            suggestion: "File may be corrupted or inaccessible".to_string(),
        })?;
    
    if metadata.len() == 0 {
        return Err(ProcessingError::EmptyFile {
            path: path.to_owned(),
            suggestion: "Ensure the diagnostic file was generated successfully".to_string(),
        });
    }
    
    if metadata.len() > 10 * 1024 * 1024 * 1024 { // 10GB
        return Err(ProcessingError::FileTooLarge {
            path: path.to_owned(),
            size_gb: metadata.len() / 1024 / 1024 / 1024,
            suggestion: "Consider using batch mode or increasing memory limits".to_string(),
        });
    }
    
    let content = read_file_content_with_encoding_detection(file)
        .map_err(|e| ProcessingError::EncodingError {
            path: path.to_owned(),
            details: e.to_string(),
            suggestion: "The file may have an unsupported encoding. Try saving as UTF-8.".to_string(),
        })?;
    
    process_content(&content)
        .map_err(|e| ProcessingError::FormatError {
            path: path.to_owned(),
            source: Box::new(e),
            suggestion: "Verify this is a valid Check Point diagnostic file (cpinfo output)".to_string(),
        })?;
    
    Ok(())
}
```

### Performance Gotcha: Blocking Progress Updates
- **Problem**: Progress updates slow down processing significantly
- **Cause**: Synchronous progress reporting in tight processing loops
- **Solution**: Asynchronous progress reporting with batched updates
- **Example**:
```rust
// WRONG: Blocking progress updates
pub fn process_sections(sections: Vec<Section>) -> Result<()> {
    let total = sections.len();
    let progress_bar = ProgressBar::new(total as u64);
    
    for (i, section) in sections.iter().enumerate() {
        process_single_section(section)?;
        
        // This can be expensive if called frequently
        progress_bar.set_position((i + 1) as u64);
        progress_bar.set_message(format!("Processing {}", section.name));
    }
    
    Ok(())
}

// CORRECT: Batched asynchronous progress updates
pub async fn process_sections_async(sections: Vec<Section>) -> Result<()> {
    let total = sections.len();
    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel();
    
    // Spawn progress reporting task
    let progress_handle = tokio::spawn(async move {
        let progress_bar = ProgressBar::new(total as u64);
        let mut last_update = Instant::now();
        let update_interval = Duration::from_millis(100); // 10 FPS max
        
        while let Some(update) = progress_rx.recv().await {
            let now = Instant::now();
            if now - last_update >= update_interval {
                match update {
                    ProgressUpdate::Position(pos) => progress_bar.set_position(pos),
                    ProgressUpdate::Message(msg) => progress_bar.set_message(msg),
                }
                last_update = now;
            }
        }
        
        progress_bar.finish();
    });
    
    // Process sections with non-blocking progress updates
    for (i, section) in sections.iter().enumerate() {
        process_single_section(section)?;
        
        // Non-blocking progress update (every 10th section or last)
        if i % 10 == 0 || i == sections.len() - 1 {
            let _ = progress_tx.send(ProgressUpdate::Position((i + 1) as u64));
            let _ = progress_tx.send(ProgressUpdate::Message(
                format!("Processed {}/{}", i + 1, total)
            ));
        }
    }
    
    drop(progress_tx); // Signal completion
    progress_handle.await?;
    
    Ok(())
}
```

## Best Practices

### Help System Design
```rust
/// Comprehensive help system with examples and context
pub struct CliHelpSystem {
    examples: HashMap<String, Vec<HelpExample>>,
    troubleshooting: HashMap<String, TroubleshootingGuide>,
}

#[derive(Debug, Clone)]
pub struct HelpExample {
    pub title: String,
    pub description: String,
    pub command: String,
    pub expected_output: String,
    pub notes: Vec<String>,
}

impl CliHelpSystem {
    pub fn generate_contextual_help(&self, command: &str, args: &ExtractOptions) -> String {
        let mut help = String::new();
        
        // Command-specific help
        help.push_str(&format!("Help for: {}\n\n", command));
        
        // Context-aware examples
        if args.vsx_aware {
            help.push_str("VSX Environment Examples:\n");
            help.push_str("  cpinfo-parser extract --vsx-aware --output ./vsx_extracted/ diagnostic.txt\n");
            help.push_str("  # This will create separate directories for each virtual system\n\n");
        }
        
        if args.tac_format {
            help.push_str("TAC Submission Examples:\n");
            help.push_str("  cpinfo-parser tac-package --case-priority high --anonymize diagnostic.txt\n");
            help.push_str("  # Creates a package optimized for TAC submission\n\n");
        }
        
        // Common scenarios
        help.push_str("Common Scenarios:\n");
        help.push_str("  1. Basic extraction:\n");
        help.push_str("     cpinfo-parser extract diagnostic.txt\n\n");
        help.push_str("  2. Extract specific sections:\n");
        help.push_str("     cpinfo-parser extract --filter fw,vpn diagnostic.txt\n\n");
        help.push_str("  3. Process multiple files:\n");
        help.push_str("     cpinfo-parser batch ./diagnostics/ --threads 4\n\n");
        
        // Troubleshooting section
        help.push_str("Troubleshooting:\n");
        help.push_str("  • File not found: Check path and permissions\n");
        help.push_str("  • Memory errors: Increase --memory-limit or use batch mode\n");
        help.push_str("  • Slow processing: Reduce --threads or check disk space\n\n");
        
        help.push_str("For more help: cpinfo-parser examples\n");
        
        help
    }
}
```

### Configuration Management
```rust
/// Hierarchical configuration system for CLI applications
pub struct ConfigManager {
    global_config: Option<AppConfig>,
    user_config: Option<AppConfig>,
    project_config: Option<AppConfig>,
    cli_overrides: CliOverrides,
}

impl ConfigManager {
    pub fn load_configuration() -> Result<AppConfig, ConfigError> {
        let mut config_builder = config::Config::builder();
        
        // Layer 1: Built-in defaults
        config_builder = config_builder.add_source(
            config::File::from_str(include_str!("default_config.toml"), FileFormat::Toml)
        );
        
        // Layer 2: System-wide configuration
        if let Some(system_config) = Self::find_system_config()? {
            config_builder = config_builder.add_source(config::File::from(system_config));
        }
        
        // Layer 3: User configuration
        if let Some(user_config) = Self::find_user_config()? {
            config_builder = config_builder.add_source(config::File::from(user_config));
        }
        
        // Layer 4: Project-local configuration
        if let Some(project_config) = Self::find_project_config()? {
            config_builder = config_builder.add_source(config::File::from(project_config));
        }
        
        // Layer 5: Environment variables
        config_builder = config_builder.add_source(
            config::Environment::with_prefix("CPINFO")
                .separator("_")
                .try_parsing(true)
        );
        
        let config = config_builder.build()?;
        let app_config: AppConfig = config.try_deserialize()?;
        
        // Validate final configuration
        app_config.validate()?;
        
        Ok(app_config)
    }
    
    /// Apply CLI argument overrides to configuration
    pub fn apply_cli_overrides(&self, mut config: AppConfig, cli_args: &Cli) -> AppConfig {
        // Global CLI overrides
        if cli_args.enterprise {
            config.features.enterprise_mode = true;
        }
        
        if cli_args.no_color {
            config.display.color_output = false;
        }
        
        // Verbosity level override
        config.logging.level = match cli_args.verbose {
            0 => "info".to_string(),
            1 => "debug".to_string(),
            2.. => "trace".to_string(),
        };
        
        config
    }
}
```

### Shell Integration
```rust
/// Shell completion and integration utilities
pub mod shell_integration {
    use clap_complete::{generate, Generator, Shell};
    
    /// Generate shell completion scripts
    pub fn generate_completions<G: Generator>(gen: G, app: &mut clap::Command) {
        generate(gen, app, "cpinfo-parser", &mut std::io::stdout());
    }
    
    /// Install shell completions for the current user
    pub fn install_completions() -> Result<(), std::io::Error> {
        let app = crate::cli::Cli::command();
        
        // Detect shell from environment
        let shell = detect_shell().unwrap_or(Shell::Bash);
        
        let completion_dir = get_completion_directory(shell)?;
        std::fs::create_dir_all(&completion_dir)?;
        
        let completion_file = completion_dir.join("cpinfo-parser");
        let mut file = std::fs::File::create(completion_file)?;
        
        generate(shell, &mut app.clone(), "cpinfo-parser", &mut file);
        
        println!("Shell completions installed for {:?}", shell);
        println!("Restart your shell or source the completion file to enable.");
        
        Ok(())
    }
    
    /// Shell integration for common workflows
    pub fn generate_shell_aliases() -> String {
        formatdoc! {r#"
            # cpinfo-parser shell aliases and functions
            
            # Quick extraction
            alias cpextract='cpinfo-parser extract'
            alias cpbatch='cpinfo-parser batch'
            alias cpvalidate='cpinfo-parser validate'
            
            # Common workflows
            cptac() {{
                cpinfo-parser tac-package --case-priority "${{1:-medium}}" --anonymize "$2"
            }}
            
            cpvsx() {{
                cpinfo-parser extract --vsx-aware --detailed-progress "$1"
            }}
            
            cpanalyze() {{
                cpinfo-parser analyze --include-performance --recommendations "$1"
            }}
            
            # Completion helper for file paths
            _cpinfo_files() {{
                find . -name "*.txt" -o -name "*.log" | grep -E "(cpinfo|diagnostic)" 2>/dev/null
            }}
        "#}
    }
}
```

## Integration Points

### Terminal UI Integration
```rust
/// Rich terminal interface with ratatui for complex interactions
pub struct TerminalInterface {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    app_state: AppState,
}

impl TerminalInterface {
    /// File browser for interactive file selection
    pub fn show_file_browser(&mut self, initial_path: &Path) -> Result<Option<PathBuf>, TuiError> {
        let mut file_browser = FileBrowser::new(initial_path)?;
        
        loop {
            self.terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(1), Constraint::Length(3)])
                    .split(f.size());
                
                // File listing
                file_browser.render(f, chunks[0]);
                
                // Help text
                let help = Paragraph::new("↑↓: Navigate, Enter: Select, q: Cancel")
                    .style(Style::default().fg(Color::Gray))
                    .wrap(Wrap { trim: true });
                f.render_widget(help, chunks[1]);
            })?;
            
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => file_browser.previous(),
                    KeyCode::Down => file_browser.next(),
                    KeyCode::Enter => {
                        if let Some(selected) = file_browser.selected() {
                            return Ok(Some(selected));
                        }
                    }
                    KeyCode::Char('q') => return Ok(None),
                    _ => {}
                }
            }
        }
    }
    
    /// Progress visualization with real-time updates
    pub fn show_progress_dashboard(&mut self, progress_rx: mpsc::Receiver<ProgressUpdate>) -> Result<(), TuiError> {
        let mut dashboard = ProgressDashboard::new();
        
        loop {
            // Non-blocking progress update receive
            while let Ok(update) = progress_rx.try_recv() {
                dashboard.update(update);
                
                if dashboard.is_complete() {
                    self.show_completion_screen(&dashboard)?;
                    return Ok(());
                }
            }
            
            self.terminal.draw(|f| {
                dashboard.render(f, f.size());
            })?;
            
            // Check for user input (pause, cancel, etc.)
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('p') => dashboard.toggle_pause(),
                        KeyCode::Char('c') => dashboard.request_cancel(),
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    }
                }
            }
        }
    }
}
```

## Troubleshooting

### CLI Testing Strategies
```rust
/// Comprehensive testing framework for CLI applications
#[cfg(test)]
mod cli_tests {
    use super::*;
    use assert_cmd::Command;
    use predicates::prelude::*;
    use tempfile::TempDir;
    
    /// Test CLI argument parsing and validation
    #[test]
    fn test_cli_argument_validation() {
        // Valid arguments
        let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
        cmd.arg("extract")
           .arg("test_file.txt")
           .arg("--memory-limit")
           .arg("500MB")
           .assert()
           .success();
        
        // Invalid memory limit
        let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
        cmd.arg("extract")
           .arg("test_file.txt")
           .arg("--memory-limit")
           .arg("invalid")
           .assert()
           .failure()
           .stderr(predicate::str::contains("Invalid memory size"));
        
        // Conflicting arguments
        let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
        cmd.arg("extract")
           .arg("test_file.txt")
           .arg("--dry-run")
           .arg("--overwrite")
           .assert()
           .failure()
           .stderr(predicate::str::contains("conflicts with"));
    }
    
    /// Test help system completeness
    #[test]
    fn test_help_completeness() {
        // Main help
        let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
        cmd.arg("--help")
           .assert()
           .success()
           .stdout(predicate::str::contains("Check Point diagnostic section parser"));
        
        // Subcommand help
        let subcommands = ["extract", "validate", "batch", "analyze", "tac-package"];
        for subcommand in &subcommands {
            let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
            cmd.arg(subcommand)
               .arg("--help")
               .assert()
               .success()
               .stdout(predicate::str::contains("Examples:"));
        }
    }
    
    /// Test error message quality
    #[test]
    fn test_error_messages() {
        let temp_dir = TempDir::new().unwrap();
        
        // File not found error
        let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
        cmd.arg("extract")
           .arg("nonexistent_file.txt")
           .assert()
           .failure()
           .stderr(predicate::str::contains("File not found"))
           .stderr(predicate::str::contains("Verify the file path"));
        
        // Permission denied simulation (if testable)
        // Empty file error
        let empty_file = temp_dir.path().join("empty.txt");
        std::fs::write(&empty_file, "").unwrap();
        
        let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
        cmd.arg("extract")
           .arg(&empty_file)
           .assert()
           .failure()
           .stderr(predicate::str::contains("Empty file"))
           .stderr(predicate::str::contains("diagnostic file was generated successfully"));
    }
    
    /// Integration test with real diagnostic file
    #[test]
    fn test_real_file_processing() {
        let test_file = create_test_diagnostic_file();
        let output_dir = TempDir::new().unwrap();
        
        let mut cmd = Command::cargo_bin("cpinfo-parser").unwrap();
        cmd.arg("extract")
           .arg(&test_file)
           .arg("--output")
           .arg(output_dir.path())
           .arg("--dry-run")
           .assert()
           .success()
           .stdout(predicate::str::contains("sections would be extracted"));
        
        // Verify no actual extraction in dry-run mode
        assert!(output_dir.path().read_dir().unwrap().count() == 0);
    }
}
```

### Common CLI Issues Debug Guide
```markdown
## CLI Troubleshooting Guide

### Issue: Command not recognized
**Symptoms**: "command not found" or similar error
**Causes**: 
- Binary not in PATH
- Installation incomplete
- Wrong binary name
**Solutions**:
1. Check installation: `which cpinfo-parser`
2. Add to PATH: `export PATH=$PATH:/path/to/binary`
3. Verify binary name and permissions

### Issue: Arguments not parsed correctly  
**Symptoms**: Unexpected behavior or "invalid argument" errors
**Causes**:
- Quoting issues with paths containing spaces
- Conflicting argument combinations
- Invalid value formats
**Solutions**:
1. Quote paths: `cpinfo-parser extract "path with spaces/file.txt"`
2. Check for conflicting flags: `--dry-run` vs `--overwrite`
3. Validate value formats: `--memory-limit 500MB` not `500 MB`

### Issue: Poor performance or hanging
**Symptoms**: Very slow processing or apparent freezing
**Causes**:
- Large files exceeding memory limits
- Too many concurrent operations
- I/O bottlenecks
**Solutions**:
1. Increase memory limit: `--memory-limit 2GB`
2. Reduce concurrency: `--threads 2`
3. Use batch mode for multiple files
4. Check available disk space

### Issue: Output formatting problems
**Symptoms**: Mangled text, encoding issues, or missing content
**Causes**:
- Terminal encoding mismatch  
- Color codes in non-color terminals
- Pipe buffer issues
**Solutions**:
1. Force UTF-8: `export LANG=en_US.UTF-8`
2. Disable colors: `--no-color`
3. Use appropriate output format: `--output-format json`
```

## References

- [clap Documentation](https://docs.rs/clap/) - CLI argument parsing framework
- [indicatif Documentation](https://docs.rs/indicatif/) - Progress bar library
- [ratatui Documentation](https://ratatui.rs/) - Terminal UI framework  
- [CLI Guidelines](https://clig.dev/) - Command line interface design principles
- [12 Factor CLI Apps](https://medium.com/@jdxcode/12-factor-cli-apps-dd3c227a0e46) - CLI application best practices
- [Architecture Document](architecture.md) - System design context
- [User Experience Principles](https://ux.gov.au/guidance/principles/) - UX design guidance