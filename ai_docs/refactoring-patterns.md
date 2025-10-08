# Refactoring Patterns Guide - cpinfo-parser

## Overview
- **Purpose**: Common refactoring patterns for the cpinfo-parser codebase
- **Use Cases**: Phase 1-2 refactoring implementation guidance
- **Version**: Rust 1.75+ with strict clippy configuration
- **Last Updated**: 2025-08-08

## Key Concepts

### Refactoring Philosophy
The cpinfo-parser refactoring follows **Clean Architecture** principles with **Rust-specific optimizations**:
- **Separation of Concerns**: Clear boundaries between modules
- **Dependency Inversion**: Abstractions don't depend on details
- **Single Responsibility**: Each module has one reason to change
- **Memory Safety**: Zero unsafe code with compile-time guarantees

### Refactoring Stages
1. **Foundation Phase**: Error handling, logging, basic structure
2. **Quality Phase**: Patterns, standards, comprehensive testing
3. **Performance Phase**: Optimization, streaming, memory management
4. **Polish Phase**: Documentation, deployment, monitoring

## Implementation Patterns

### Primary Pattern: Error-First Refactoring

```rust
// BEFORE: Basic error handling
fn parse_section(content: &str) -> Option<Section> {
    // Simple parsing with potential panics
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 3 {
        return None;
    }
    // ... parsing logic that might panic
}

// AFTER: Comprehensive error handling
fn parse_section(content: &str) -> Result<Section, ParseError> {
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.len() < 3 {
        return Err(ParseError::InsufficientContent {
            found: lines.len(),
            minimum_required: 3,
        });
    }
    
    let header = parse_header(lines[0])
        .map_err(|e| ParseError::HeaderParseError {
            line: 0,
            source: Box::new(e),
        })?;
        
    let body = parse_body(&lines[1..lines.len()-1])
        .map_err(|e| ParseError::BodyParseError {
            source: Box::new(e),
        })?;
        
    Ok(Section { header, body })
}

// Domain-specific error types
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Insufficient content: found {found} lines, need at least {minimum_required}")]
    InsufficientContent {
        found: usize,
        minimum_required: usize,
    },
    
    #[error("Header parse error at line {line}")]
    HeaderParseError {
        line: usize,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Body parse error")]
    BodyParseError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
```

### Alternative Pattern: Module Boundary Refactoring

```rust
// BEFORE: Monolithic structure
mod parser {
    pub struct CpinfoParser {
        // All functionality mixed together
    }
    
    impl CpinfoParser {
        pub fn parse_file(&self, path: &Path) -> Result<()> {
            // File I/O, parsing, validation, output all mixed
        }
    }
}

// AFTER: Clean separation of concerns
pub mod io {
    pub trait FileReader: Send + Sync {
        async fn read_file(&self, path: &Path) -> Result<String, IoError>;
    }
    
    pub struct BufferedFileReader {
        buffer_size: usize,
    }
    
    impl FileReader for BufferedFileReader {
        async fn read_file(&self, path: &Path) -> Result<String, IoError> {
            // Optimized file reading implementation
        }
    }
}

pub mod parsing {
    use crate::io::FileReader;
    
    pub trait Parser: Send + Sync {
        async fn parse(&self, content: &str) -> Result<ParsedContent, ParseError>;
    }
    
    pub struct CpinfoParser<R: FileReader> {
        reader: R,
        validator: Validator,
    }
    
    impl<R: FileReader> Parser for CpinfoParser<R> {
        async fn parse(&self, content: &str) -> Result<ParsedContent, ParseError> {
            // Pure parsing logic, no I/O concerns
        }
    }
}

pub mod output {
    pub trait OutputWriter: Send + Sync {
        async fn write_sections(&self, sections: &[Section]) -> Result<(), OutputError>;
    }
    
    pub struct FileSystemWriter {
        base_path: PathBuf,
        organizer: OutputOrganizer,
    }
}

// Orchestration layer
pub mod workflow {
    use crate::{io::FileReader, parsing::Parser, output::OutputWriter};
    
    pub struct ProcessingWorkflow<R, P, W> {
        reader: R,
        parser: P,
        writer: W,
    }
    
    impl<R, P, W> ProcessingWorkflow<R, P, W>
    where
        R: FileReader,
        P: Parser,
        W: OutputWriter,
    {
        pub async fn process_file(&self, path: &Path) -> Result<ProcessingResults, WorkflowError> {
            let content = self.reader.read_file(path).await?;
            let parsed = self.parser.parse(&content).await?;
            self.writer.write_sections(&parsed.sections).await?;
            
            Ok(ProcessingResults {
                sections_extracted: parsed.sections.len(),
                processing_time: parsed.processing_time,
            })
        }
    }
}
```

### State Machine Refactoring Pattern

```rust
// BEFORE: Imperative parsing with complex control flow
fn parse_cpinfo_content(content: &str) -> Result<Vec<Section>, ParseError> {
    let mut sections = Vec::new();
    let mut in_section = false;
    let mut current_section_lines = Vec::new();
    let mut section_name = String::new();
    
    for line in content.lines() {
        if line.starts_with("========================") {
            if in_section {
                // Complex logic to handle section completion
                sections.push(parse_section_from_lines(&current_section_lines, &section_name)?);
                current_section_lines.clear();
            }
            in_section = !in_section;
            if in_section {
                // Complex logic to extract section name
            }
        } else if in_section {
            current_section_lines.push(line);
        }
    }
    
    Ok(sections)
}

// AFTER: State machine with clear transitions
#[derive(Debug, Clone, PartialEq)]
pub enum ParserState {
    ScanningForDelimiter,
    ValidatingPattern {
        pattern_type: DelimiterType,
        line_count: usize,
        opening_line: String,
    },
    ExtractingContent {
        content_type: ContentType,
        content_name: String,
        start_line: usize,
    },
    ProcessingComplete,
    ErrorRecovery {
        error: ParseError,
    },
}

pub struct StateMachineParser {
    state: ParserState,
    sections: Vec<Section>,
    current_content: Vec<String>,
}

impl StateMachineParser {
    pub fn process_line(&mut self, line: &str, line_number: usize) -> Result<Vec<ParserAction>, ParseError> {
        match (&self.state, line) {
            (ParserState::ScanningForDelimiter, line) if self.is_delimiter(line) => {
                self.state = ParserState::ValidatingPattern {
                    pattern_type: self.detect_delimiter_type(line)?,
                    line_count: 1,
                    opening_line: line.to_string(),
                };
                Ok(vec![ParserAction::ContinueProcessing])
            },
            
            (ParserState::ValidatingPattern { pattern_type, line_count, opening_line }, line) => {
                match line_count {
                    1 => {
                        // Section name line
                        self.state = ParserState::ExtractingContent {
                            content_type: pattern_type.to_content_type(),
                            content_name: line.trim().to_string(),
                            start_line: line_number + 1,
                        };
                        Ok(vec![ParserAction::BeginExtraction(line.trim().to_string())])
                    },
                    2 => {
                        // Closing delimiter validation
                        if line == opening_line {
                            Ok(vec![ParserAction::ContinueProcessing])
                        } else {
                            self.enter_error_recovery(ParseError::MismatchedDelimiter {
                                line: line_number,
                                expected: opening_line.clone(),
                                found: line.to_string(),
                            })
                        }
                    },
                    _ => {
                        self.enter_error_recovery(ParseError::InvalidDelimiterStructure {
                            line: line_number,
                        })
                    }
                }
            },
            
            (ParserState::ExtractingContent { content_name, .. }, line) => {
                if self.is_delimiter(line) {
                    // Complete current section and start new one
                    let section = self.complete_current_section(content_name)?;
                    self.sections.push(section);
                    self.current_content.clear();
                    
                    self.state = ParserState::ValidatingPattern {
                        pattern_type: self.detect_delimiter_type(line)?,
                        line_count: 1,
                        opening_line: line.to_string(),
                    };
                    
                    Ok(vec![
                        ParserAction::SectionCompleted(content_name.clone()),
                        ParserAction::ContinueProcessing,
                    ])
                } else {
                    self.current_content.push(line.to_string());
                    Ok(vec![ParserAction::ContentExtracted])
                }
            },
            
            (ParserState::ErrorRecovery { .. }, line) => {
                if self.is_delimiter(line) {
                    self.state = ParserState::ValidatingPattern {
                        pattern_type: self.detect_delimiter_type(line)?,
                        line_count: 1,
                        opening_line: line.to_string(),
                    };
                    Ok(vec![ParserAction::RecoverySuccessful, ParserAction::ContinueProcessing])
                } else {
                    Ok(vec![ParserAction::ContinueRecovery])
                }
            },
            
            _ => Ok(vec![ParserAction::ContinueProcessing]),
        }
    }
}
```

## Common Gotchas

### Critical Gotcha: Async Context Propagation
- **Problem**: Async operations in refactored modules don't properly propagate context
- **Cause**: Missing `Send + Sync` bounds on trait objects, blocking operations in async contexts
- **Solution**: Use proper async traits and ensure all operations are non-blocking
- **Example**:
```rust
// WRONG: Blocking I/O in async context
pub trait Parser {
    async fn parse(&self, content: &str) -> Result<ParsedContent, ParseError>;
}

impl Parser for CpinfoParser {
    async fn parse(&self, content: &str) -> Result<ParsedContent, ParseError> {
        // This blocks the async runtime
        std::fs::write("debug.txt", content).unwrap();
        // ... parsing logic
    }
}

// CORRECT: Non-blocking operations with proper error handling
#[async_trait]
pub trait Parser: Send + Sync {
    async fn parse(&self, content: &str) -> Result<ParsedContent, ParseError>;
}

#[async_trait]
impl Parser for CpinfoParser {
    async fn parse(&self, content: &str) -> Result<ParsedContent, ParseError> {
        // Non-blocking async I/O
        tokio::fs::write("debug.txt", content).await
            .map_err(|e| ParseError::DebugWriteFailed { source: e })?;
        // ... parsing logic
    }
}
```

### Critical Gotcha: Error Context Loss in Refactoring
- **Problem**: Error context is lost when refactoring from simple to complex error types
- **Cause**: Not properly chaining error sources or losing contextual information
- **Solution**: Use `thiserror` with proper source chaining and contextual error variants
- **Example**:
```rust
// WRONG: Context loss during error propagation
fn parse_header(line: &str) -> Result<Header, ParseError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != 3 {
        return Err(ParseError::InvalidFormat); // Lost: what was wrong, which line
    }
    // ...
}

// CORRECT: Preserved context with detailed error information
fn parse_header(line: &str, line_number: usize) -> Result<Header, ParseError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != 3 {
        return Err(ParseError::InvalidHeaderFormat {
            line_number,
            line_content: line.to_string(),
            expected_parts: 3,
            found_parts: parts.len(),
            suggestion: "Header format should be: <type> <name> <size>".to_string(),
        });
    }
    // ...
}
```

### Performance Gotcha: Unnecessary Allocations in Refactored Code
- **Problem**: Refactoring introduces unnecessary string allocations and cloning
- **Cause**: Over-use of owned types instead of borrowing, defensive cloning
- **Solution**: Use lifetime parameters and borrowing where appropriate
- **Example**:
```rust
// WRONG: Excessive allocations
pub struct Section {
    pub name: String,
    pub content: Vec<String>,
}

impl Section {
    pub fn new(name: String, lines: Vec<String>) -> Self {
        Self {
            name: name.clone(), // Unnecessary clone
            content: lines.into_iter().map(|s| s.clone()).collect(), // More clones
        }
    }
}

// CORRECT: Minimal allocations with borrowing
pub struct Section {
    pub name: String,
    pub content: Vec<String>,
}

impl Section {
    pub fn new(name: String, lines: Vec<String>) -> Self {
        Self { name, content: lines } // Move semantics, no cloning
    }
    
    pub fn from_borrowed(name: &str, lines: &[&str]) -> Self {
        Self {
            name: name.to_string(), // Only clone when necessary
            content: lines.iter().map(|&s| s.to_string()).collect(),
        }
    }
}
```

## Best Practices

### Refactoring Process
1. **Start with Error Types**: Define comprehensive error types before refactoring logic
2. **Extract Traits First**: Define interfaces before implementing concrete types  
3. **Preserve Behavior**: Each refactoring step should maintain existing functionality
4. **Add Tests Incrementally**: Write tests for new patterns as you introduce them
5. **Use Compiler Guidance**: Let Rust's type system guide the refactoring process

### Module Organization Principles
```rust
// Good module structure following Clean Architecture
src/
├── domain/              # Business logic, pure functions
│   ├── models.rs       # Domain types and entities
│   ├── services.rs     # Domain services and use cases
│   └── errors.rs       # Domain-specific error types
├── infrastructure/     # External concerns (I/O, persistence)
│   ├── file_io.rs      # File system operations
│   ├── logging.rs      # Logging infrastructure
│   └── cli.rs          # Command-line interface
├── application/        # Application orchestration
│   ├── workflow.rs     # Application workflows
│   ├── config.rs       # Configuration management
│   └── handlers.rs     # Request/response handlers
└── lib.rs              # Public API surface
```

### Error Handling Strategy
```rust
// Hierarchical error types with proper context
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("Configuration error")]
    Config(#[from] ConfigError),
    
    #[error("Parsing error")]
    Parse(#[from] ParseError),
    
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    
    #[error("Workflow error: {message}")]
    Workflow {
        message: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

// Context-preserving error conversion
impl From<ParseError> for ApplicationError {
    fn from(err: ParseError) -> Self {
        match err {
            ParseError::InvalidFormat { .. } => {
                ApplicationError::Workflow {
                    message: "Failed to parse cpinfo format".to_string(),
                    source: Box::new(err),
                }
            }
            _ => ApplicationError::Parse(err),
        }
    }
}
```

### Testing Strategy for Refactored Code
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    // Property-based testing for refactored parsing logic
    proptest! {
        #[test]
        fn refactored_parser_maintains_original_behavior(
            content in generate_valid_cpinfo_content(),
            config in generate_parser_config()
        ) {
            let original_result = original_parse_function(&content, &config);
            let refactored_result = refactored_parser_workflow(&content, &config);
            
            // Both should produce equivalent results
            prop_assert_eq!(
                original_result.map(normalize_sections),
                refactored_result.map(normalize_sections)
            );
        }
    }
    
    // Unit tests for individual refactored components
    #[test]
    fn state_machine_parser_handles_error_recovery() {
        let mut parser = StateMachineParser::new();
        let input_lines = [
            "========================", // Valid delimiter
            "Test Section",            // Section name
            "========================", // Valid closing
            "content line 1",
            "invalid-delimiter-pattern", // Should trigger error recovery
            "more content",
            "========================", // Should recover here
        ];
        
        let mut results = Vec::new();
        for (i, line) in input_lines.iter().enumerate() {
            results.extend(parser.process_line(line, i)?);
        }
        
        assert!(results.contains(&ParserAction::RecoverySuccessful));
    }
}
```

## Integration Points

### CLI Integration Pattern
```rust
// Clean separation between CLI and business logic
pub mod cli {
    use crate::application::workflow::ProcessingWorkflow;
    
    pub struct CliHandler<W> {
        workflow: W,
    }
    
    impl<W: ProcessingWorkflow> CliHandler<W> {
        pub async fn handle_extract_command(&self, args: ExtractArgs) -> Result<(), CliError> {
            let config = ProcessingConfig::from_args(&args)?;
            let results = self.workflow.process_files(&args.input_files, config).await?;
            
            self.display_results(&results).await?;
            Ok(())
        }
        
        async fn display_results(&self, results: &ProcessingResults) -> Result<(), CliError> {
            // Presentation logic separated from business logic
            println!("Successfully processed {} files", results.file_count);
            println!("Extracted {} sections", results.section_count);
            Ok(())
        }
    }
}
```

### Configuration Integration
```rust
// Layered configuration with validation
pub mod config {
    #[derive(Debug, Clone, serde::Deserialize)]
    pub struct AppConfig {
        pub parsing: ParsingConfig,
        pub output: OutputConfig,
        pub logging: LoggingConfig,
    }
    
    impl AppConfig {
        pub fn load() -> Result<Self, ConfigError> {
            let mut config = config::Config::builder()
                .add_source(config::File::with_name("default"))
                .add_source(config::File::with_name("local").required(false))
                .add_source(config::Environment::with_prefix("CPINFO"))
                .build()?;
                
            let app_config: AppConfig = config.try_deserialize()?;
            app_config.validate()?;
            Ok(app_config)
        }
        
        fn validate(&self) -> Result<(), ConfigError> {
            if self.parsing.buffer_size < 1024 {
                return Err(ConfigError::InvalidValue {
                    key: "parsing.buffer_size".to_string(),
                    value: self.parsing.buffer_size.to_string(),
                    reason: "Buffer size must be at least 1024 bytes".to_string(),
                });
            }
            Ok(())
        }
    }
}
```

## Troubleshooting

### Common Refactoring Issues

**Issue**: Circular dependencies after module extraction
**Solution**: Use dependency inversion with traits, introduce intermediate abstraction layer
**Prevention**: Design module boundaries before extracting code

**Issue**: Performance regression after refactoring
**Solution**: Profile before and after, identify allocation hotspots, use benchmarks
**Prevention**: Run performance tests as part of refactoring validation

**Issue**: Lost error context in complex error hierarchies
**Solution**: Use `anyhow::Context` for ad-hoc context, `thiserror` for structured errors
**Prevention**: Plan error taxonomy before refactoring error handling

### Debug Techniques
```rust
// Debugging refactored async workflows
#[tracing::instrument(skip(self))]
pub async fn process_file(&self, path: &Path) -> Result<ProcessingResults, WorkflowError> {
    tracing::info!("Starting file processing", path = %path.display());
    
    let content = self.reader.read_file(path).await
        .map_err(|e| {
            tracing::error!("File read failed", error = %e, path = %path.display());
            WorkflowError::IoError { path: path.to_owned(), source: e }
        })?;
    
    tracing::debug!("File content loaded", size = content.len());
    // ... rest of processing
}

// Use structured logging for debugging refactored state machines
impl StateMachineParser {
    pub fn process_line(&mut self, line: &str, line_number: usize) -> Result<Vec<ParserAction>, ParseError> {
        tracing::trace!("Processing line", 
            line_number = line_number,
            current_state = ?self.state,
            line_preview = &line[..line.len().min(50)]
        );
        
        let actions = self.process_line_internal(line, line_number)?;
        
        tracing::trace!("Line processed", 
            actions = ?actions,
            new_state = ?self.state
        );
        
        Ok(actions)
    }
}
```

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Clean Architecture in Rust](https://github.com/rust-clean/rust-clean-architecture)
- [Error Handling in Rust](https://blog.burntsushi.net/rust-error-handling/)
- [Async Rust Patterns](https://tokio.rs/tokio/tutorial)
- [Architecture Document](architecture.md) - System design context
- [Quality Standards](quality-standards.md) - Code quality requirements