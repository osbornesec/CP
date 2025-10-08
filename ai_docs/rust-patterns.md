# Rust Patterns Guide - cpinfo-parser

## Overview
- **Purpose**: Rust-specific best practices and idiomatic patterns for the cpinfo-parser
- **Use Cases**: Language-specific guidance throughout all refactoring phases  
- **Version**: Rust 1.75+ with 2021 edition features
- **Last Updated**: 2025-08-08

## Key Concepts

### Rust Philosophy for CLI Tools
The cpinfo-parser leverages **Rust's unique strengths** for **systems programming**:
- **Memory Safety**: Zero-cost abstractions without garbage collection overhead
- **Concurrency**: Fearless parallelism with ownership-based thread safety
- **Performance**: Predictable performance characteristics for enterprise workloads
- **Reliability**: Compile-time error prevention for production stability

### Idiomatic Rust Principles
1. **Ownership and Borrowing**: Efficient memory management without manual allocation
2. **Error Handling**: Explicit error types with comprehensive context
3. **Zero-Cost Abstractions**: High-level patterns with low-level performance
4. **Type Safety**: Compile-time guarantees for runtime correctness

## Implementation Patterns

### Primary Pattern: Ownership-Based Resource Management

```rust
/// Ownership-based file processing with automatic resource cleanup
pub struct CpinfoProcessor {
    config: ProcessingConfig,
    file_handles: Vec<OwnedFile>,          // Owned file handles
    temp_resources: Vec<TempResource>,     // Automatically cleaned up
    metrics: ProcessingMetrics,
}

/// RAII wrapper for file handles with automatic cleanup
pub struct OwnedFile {
    file: std::fs::File,
    path: PathBuf,
    metadata: FileMetadata,
}

impl OwnedFile {
    /// Create owned file with validation and metadata collection
    pub fn open(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref().to_owned();
        let file = std::fs::File::open(&path)?;
        let metadata = FileMetadata::from_file(&file)?;
        
        tracing::debug!("Opened file for processing", 
            path = %path.display(),
            size_mb = metadata.size_bytes / 1024 / 1024
        );
        
        Ok(Self { file, path, metadata })
    }
    
    /// Consume self to get inner file (transfer ownership)
    pub fn into_file(self) -> std::fs::File {
        self.file
    }
    
    /// Borrow file handle for read operations  
    pub fn as_file(&self) -> &std::fs::File {
        &self.file
    }
    
    /// Get metadata without transferring ownership
    pub fn metadata(&self) -> &FileMetadata {
        &self.metadata
    }
}

// Automatic cleanup on drop
impl Drop for OwnedFile {
    fn drop(&mut self) {
        tracing::trace!("Closing file", path = %self.path.display());
        // File is automatically closed when dropped
    }
}

impl CpinfoProcessor {
    /// Process multiple files with ownership transfer
    pub fn process_files(mut self, file_paths: Vec<PathBuf>) -> Result<ProcessingResults, ProcessingError> {
        let mut results = Vec::new();
        
        for path in file_paths {
            // Transfer ownership of file to processor
            let owned_file = OwnedFile::open(&path)
                .map_err(|e| ProcessingError::FileOpenFailed { path: path.clone(), source: e })?;
            
            // Process file, consuming ownership
            let file_result = self.process_single_file(owned_file)?;
            results.push(file_result);
        }
        
        // Return results, processor and all resources are automatically cleaned up
        Ok(ProcessingResults { individual_results: results })
    }
    
    fn process_single_file(&mut self, owned_file: OwnedFile) -> Result<SingleFileResult, ProcessingError> {
        let file_path = owned_file.path.clone();
        let file_size = owned_file.metadata().size_bytes;
        
        // Create buffered reader, transferring file ownership
        let reader = BufReader::new(owned_file.into_file());
        
        // Process with streaming approach
        let sections = self.parse_sections_streaming(reader)?;
        let organized_output = self.organize_sections(sections)?;
        
        Ok(SingleFileResult {
            source_path: file_path,
            processed_sections: organized_output.len(),
            processing_time: self.metrics.last_processing_duration(),
            output_files: organized_output,
        })
    }
}
```

### Borrowed Data Pattern for Zero-Copy Processing

```rust
/// Zero-copy string processing with lifetime management
pub struct SectionParser<'content> {
    content: &'content str,
    position: usize,
    current_line: usize,
}

impl<'content> SectionParser<'content> {
    pub fn new(content: &'content str) -> Self {
        Self {
            content,
            position: 0,
            current_line: 1,
        }
    }
    
    /// Parse section header without allocating strings
    pub fn parse_header(&mut self) -> Result<SectionHeader<'content>, ParseError> {
        let header_start = self.position;
        
        // Find delimiter pattern using string slices (no allocation)
        let delimiter_line = self.read_line()?;
        if !Self::is_delimiter_pattern(delimiter_line) {
            return Err(ParseError::InvalidDelimiter {
                line: self.current_line,
                content: delimiter_line.to_owned(), // Only allocate for error
            });
        }
        
        // Parse section name (borrowed string slice)
        let name_line = self.read_line()?;
        let section_name = name_line.trim();
        
        // Validate closing delimiter
        let closing_delimiter = self.read_line()?;
        if closing_delimiter != delimiter_line {
            return Err(ParseError::MismatchedDelimiter {
                line: self.current_line,
                expected: delimiter_line.to_owned(),
                found: closing_delimiter.to_owned(),
            });
        }
        
        Ok(SectionHeader {
            name: section_name,              // Borrowed &str, no allocation
            delimiter_type: Self::classify_delimiter(delimiter_line),
            start_position: header_start,
            end_position: self.position,
        })
    }
    
    /// Parse section content using iterator adapters (zero-copy)
    pub fn parse_content(&mut self) -> impl Iterator<Item = &'content str> {
        let remaining_content = &self.content[self.position..];
        
        remaining_content
            .lines()
            .take_while(|line| !Self::is_delimiter_pattern(line))
            .inspect(|_| self.current_line += 1) // Side effect for line tracking
    }
    
    /// Read single line, advancing position (borrowed slice)
    fn read_line(&mut self) -> Result<&'content str, ParseError> {
        let remaining = &self.content[self.position..];
        
        if let Some(newline_pos) = remaining.find('\n') {
            let line = &remaining[..newline_pos];
            self.position += newline_pos + 1;
            self.current_line += 1;
            Ok(line.trim_end_matches('\r')) // Handle Windows line endings
        } else if !remaining.is_empty() {
            // Last line without newline
            let line = remaining;
            self.position = self.content.len();
            self.current_line += 1;
            Ok(line)
        } else {
            Err(ParseError::UnexpectedEof {
                line: self.current_line,
            })
        }
    }
}

/// Section header with borrowed string references
#[derive(Debug, Clone)]
pub struct SectionHeader<'content> {
    pub name: &'content str,             // Borrowed, no allocation
    pub delimiter_type: DelimiterType,
    pub start_position: usize,
    pub end_position: usize,
}

/// Convert borrowed data to owned when necessary
impl<'content> SectionHeader<'content> {
    pub fn to_owned(&self) -> OwnedSectionHeader {
        OwnedSectionHeader {
            name: self.name.to_owned(),  // Allocate only when converting to owned
            delimiter_type: self.delimiter_type,
            start_position: self.start_position,
            end_position: self.end_position,
        }
    }
}
```

### Error Handling Pattern with Context Preservation

```rust
/// Comprehensive error types with context and recovery information
#[derive(Debug, thiserror::Error)]
pub enum ProcessingError {
    /// I/O error with file context
    #[error("File I/O error: {path}")]
    FileIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    /// Parsing error with position context
    #[error("Parse error at line {line}, column {column}: {message}")]
    Parse {
        line: usize,
        column: usize,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Configuration error with validation details
    #[error("Configuration error: {field} = '{value}' is invalid")]
    Configuration {
        field: String,
        value: String,
        reason: String,
        suggestion: Option<String>,
    },
    
    /// Resource exhaustion with current usage info
    #[error("Resource limit exceeded: {resource} usage {current} > limit {limit}")]
    ResourceLimit {
        resource: String,
        current: u64,
        limit: u64,
        recommended_action: String,
    },
}

/// Error context extension trait for preserving call stack context
pub trait ErrorContextExt<T> {
    /// Add file processing context
    fn with_file_context(self, path: &Path) -> Result<T, ProcessingError>;
    
    /// Add parsing context with position information
    fn with_parse_context(self, line: usize, column: usize) -> Result<T, ProcessingError>;
    
    /// Add suggested recovery action
    fn with_recovery_suggestion(self, suggestion: &str) -> Result<T, ProcessingError>;
}

impl<T, E> ErrorContextExt<T> for Result<T, E>
where
    E: Into<ProcessingError>,
{
    fn with_file_context(self, path: &Path) -> Result<T, ProcessingError> {
        self.map_err(|e| {
            let base_error = e.into();
            match base_error {
                ProcessingError::FileIo { source, .. } => ProcessingError::FileIo {
                    path: path.to_owned(),
                    source,
                },
                other => other,
            }
        })
    }
    
    fn with_parse_context(self, line: usize, column: usize) -> Result<T, ProcessingError> {
        self.map_err(|e| {
            let base_error = e.into();
            ProcessingError::Parse {
                line,
                column,
                message: base_error.to_string(),
                source: Some(Box::new(base_error)),
            }
        })
    }
    
    fn with_recovery_suggestion(self, suggestion: &str) -> Result<T, ProcessingError> {
        self.map_err(|e| {
            let mut base_error = e.into();
            match &mut base_error {
                ProcessingError::Configuration { suggestion: ref mut s, .. } => {
                    *s = Some(suggestion.to_string());
                }
                ProcessingError::ResourceLimit { ref mut recommended_action, .. } => {
                    *recommended_action = suggestion.to_string();
                }
                _ => {} // No suggestion field for other error types
            }
            base_error
        })
    }
}

/// Error recovery with typed recovery strategies
pub struct ErrorRecoveryManager {
    strategies: HashMap<String, RecoveryStrategy>,
    max_recovery_attempts: usize,
    recovery_statistics: RecoveryStatistics,
}

#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    SkipAndContinue,
    RetryWithBackoff { max_attempts: usize, base_delay: Duration },
    ApplyWorkaround { workaround_fn: fn(&ProcessingError) -> Result<(), ProcessingError> },
    RequestUserInput { prompt: String, validation: fn(&str) -> bool },
    GracefulDegradation { fallback_behavior: String },
}

impl ErrorRecoveryManager {
    /// Apply contextual error recovery based on error type and history
    pub fn recover_from_error(&mut self, error: &ProcessingError) -> RecoveryAction {
        let error_type = std::any::type_name_of_val(error);
        
        match error {
            ProcessingError::Parse { line, .. } => {
                if self.recovery_statistics.consecutive_parse_errors < 5 {
                    RecoveryAction::Apply(RecoveryStrategy::SkipAndContinue)
                } else {
                    RecoveryAction::Escalate("Too many consecutive parse errors".to_string())
                }
            }
            
            ProcessingError::ResourceLimit { resource, current, limit, .. } => {
                if resource == "memory" && current > limit * 2 {
                    RecoveryAction::Escalate("Memory usage critically high".to_string())
                } else {
                    RecoveryAction::Apply(RecoveryStrategy::GracefulDegradation {
                        fallback_behavior: "Reduce processing batch size".to_string(),
                    })
                }
            }
            
            ProcessingError::FileIo { source, .. } => {
                match source.kind() {
                    std::io::ErrorKind::PermissionDenied => {
                        RecoveryAction::Apply(RecoveryStrategy::RequestUserInput {
                            prompt: "Permission denied. Retry with elevated privileges? (y/n)".to_string(),
                            validation: |input| input.to_lowercase().starts_with('y') || input.to_lowercase().starts_with('n'),
                        })
                    }
                    std::io::ErrorKind::NotFound => {
                        RecoveryAction::Apply(RecoveryStrategy::SkipAndContinue)
                    }
                    _ => {
                        RecoveryAction::Apply(RecoveryStrategy::RetryWithBackoff {
                            max_attempts: 3,
                            base_delay: Duration::from_millis(500),
                        })
                    }
                }
            }
            
            _ => RecoveryAction::Apply(RecoveryStrategy::SkipAndContinue),
        }
    }
}
```

### Async Pattern with Structured Concurrency

```rust
/// Structured concurrency for batch file processing
pub struct AsyncBatchProcessor {
    semaphore: Arc<Semaphore>,
    shutdown_signal: Arc<AtomicBool>,
    task_registry: Arc<Mutex<HashMap<u64, TaskHandle>>>,
    next_task_id: Arc<AtomicU64>,
}

impl AsyncBatchProcessor {
    pub fn new(max_concurrent_tasks: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent_tasks)),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            task_registry: Arc::new(Mutex::new(HashMap::new())),
            next_task_id: Arc::new(AtomicU64::new(1)),
        }
    }
    
    /// Process files with structured concurrency and proper cleanup
    pub async fn process_files_concurrent(
        &self,
        files: Vec<PathBuf>,
        config: ProcessingConfig,
    ) -> Result<Vec<ProcessingResult>, ProcessingError> {
        let total_files = files.len();
        let mut task_handles = Vec::with_capacity(total_files);
        let results = Arc::new(Mutex::new(Vec::with_capacity(total_files)));
        
        // Create cancellation token for graceful shutdown
        let cancellation_token = CancellationToken::new();
        
        // Spawn tasks with proper resource management
        for (index, file_path) in files.into_iter().enumerate() {
            let task_id = self.next_task_id.fetch_add(1, Ordering::SeqCst);
            let permit = Arc::clone(&self.semaphore);
            let config = config.clone();
            let results = Arc::clone(&results);
            let shutdown_signal = Arc::clone(&self.shutdown_signal);
            let cancellation_token = cancellation_token.clone();
            
            let task_handle = tokio::spawn(async move {
                // Structured resource acquisition
                let _permit = permit.acquire().await
                    .map_err(|_| ProcessingError::ConcurrencyError)?;
                
                // Check for shutdown signal
                if shutdown_signal.load(Ordering::Relaxed) {
                    return Err(ProcessingError::ShutdownRequested);
                }
                
                // Process file with cancellation support
                let file_result = tokio::select! {
                    result = Self::process_single_file_async(&file_path, &config) => {
                        result?
                    }
                    _ = cancellation_token.cancelled() => {
                        tracing::info!("Task cancelled", task_id = task_id, file = %file_path.display());
                        return Err(ProcessingError::TaskCancelled);
                    }
                };
                
                // Thread-safe result collection
                {
                    let mut results_guard = results.lock().await;
                    results_guard.push(ProcessingResult {
                        file_index: index,
                        file_path: file_path.clone(),
                        result: Ok(file_result),
                    });
                }
                
                Ok::<_, ProcessingError>(())
            });
            
            // Register task for cleanup management
            {
                let mut registry = self.task_registry.lock().await;
                registry.insert(task_id, TaskHandle {
                    handle: task_handle,
                    file_path: file_path.clone(),
                    start_time: Instant::now(),
                });
            }
        }
        
        // Wait for all tasks with timeout and error aggregation
        let completion_timeout = Duration::from_secs(config.max_processing_time_secs);
        let completion_result = tokio::time::timeout(completion_timeout, async {
            let mut errors = Vec::new();
            
            // Wait for all registered tasks
            let tasks = {
                let mut registry = self.task_registry.lock().await;
                std::mem::take(&mut *registry)
            };
            
            for (task_id, task_handle) in tasks {
                match task_handle.handle.await {
                    Ok(Ok(())) => {
                        tracing::debug!("Task completed successfully", task_id = task_id);
                    }
                    Ok(Err(e)) => {
                        tracing::error!("Task failed", task_id = task_id, error = %e);
                        errors.push(e);
                    }
                    Err(join_error) => {
                        tracing::error!("Task panicked", task_id = task_id, error = %join_error);
                        errors.push(ProcessingError::TaskPanicked {
                            task_id,
                            file_path: task_handle.file_path,
                            details: join_error.to_string(),
                        });
                    }
                }
            }
            
            if !errors.is_empty() && errors.len() == total_files {
                return Err(ProcessingError::AllTasksFailed { errors });
            }
            
            Ok(errors)
        }).await;
        
        match completion_result {
            Ok(Ok(errors)) => {
                let final_results = Arc::try_unwrap(results)
                    .map_err(|_| ProcessingError::ResultCollectionFailed)?
                    .into_inner();
                
                if !errors.is_empty() {
                    tracing::warn!("Some tasks failed", failed_count = errors.len(), total = total_files);
                }
                
                Ok(final_results)
            }
            Ok(Err(processing_error)) => Err(processing_error),
            Err(_timeout) => {
                // Initiate graceful shutdown
                cancellation_token.cancel();
                Err(ProcessingError::ProcessingTimeout {
                    timeout_secs: completion_timeout.as_secs(),
                })
            }
        }
    }
    
    /// Graceful shutdown with task cleanup
    pub async fn shutdown(&self) -> Result<(), ProcessingError> {
        // Signal shutdown to all tasks
        self.shutdown_signal.store(true, Ordering::SeqCst);
        
        // Wait for tasks to finish or force termination after timeout
        let shutdown_timeout = Duration::from_secs(30);
        tokio::time::timeout(shutdown_timeout, async {
            loop {
                let active_tasks = self.task_registry.lock().await.len();
                if active_tasks == 0 {
                    break;
                }
                
                tracing::info!("Waiting for tasks to complete", active_tasks = active_tasks);
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }).await.map_err(|_| ProcessingError::ShutdownTimeout)?;
        
        tracing::info!("Batch processor shutdown complete");
        Ok(())
    }
}

/// Task handle for resource management and monitoring
struct TaskHandle {
    handle: tokio::task::JoinHandle<Result<(), ProcessingError>>,
    file_path: PathBuf,
    start_time: Instant,
}
```

## Common Gotchas

### Critical Gotcha: Lifetime Parameter Confusion
- **Problem**: Borrowed data outlives the container, causing compilation errors
- **Cause**: Misunderstanding of lifetime relationships in struct definitions
- **Solution**: Use proper lifetime annotations and consider owned alternatives
- **Example**:
```rust
// WRONG: Lifetime annotation issues
pub struct Parser<'a> {
    content: &'a str,
    sections: Vec<Section<'a>>, // Section borrows from content
}

impl<'a> Parser<'a> {
    pub fn parse(content: &'a str) -> Self {
        let mut parser = Self {
            content,
            sections: Vec::new(),
        };
        parser.extract_sections(); // This might fail if sections outlive content
        parser
    }
    
    // PROBLEM: Returning borrowed data that might outlive input
    pub fn get_sections(&self) -> &[Section<'a>] {
        &self.sections
    }
}

// CORRECT: Clear lifetime management with owned alternatives
pub struct Parser<'content> {
    content: &'content str,
    position: usize,
}

impl<'content> Parser<'content> {
    pub fn new(content: &'content str) -> Self {
        Self { content, position: 0 }
    }
    
    // Return iterator that borrows from parser, not content directly
    pub fn sections(&self) -> impl Iterator<Item = Section<'content>> + '_ {
        SectionIterator::new(self.content, self.position)
    }
    
    // Alternative: return owned sections when lifetime becomes complex
    pub fn extract_all_sections(self) -> Vec<OwnedSection> {
        self.sections()
            .map(|section| section.to_owned())
            .collect()
    }
}
```

### Performance Gotcha: Unnecessary Cloning
- **Problem**: Cloning large data structures instead of using references
- **Cause**: Overly defensive programming or misunderstanding of borrowing
- **Solution**: Use references and lifetime parameters appropriately
- **Example**:
```rust
// WRONG: Excessive cloning
pub fn process_sections(sections: Vec<Section>) -> ProcessingResults {
    let mut results = Vec::new();
    
    for section in sections {
        let section_copy = section.clone(); // Unnecessary clone!
        let processed = process_single_section(section_copy);
        results.push(processed);
    }
    
    ProcessingResults { results }
}

// CORRECT: Use references and move semantics appropriately
pub fn process_sections(sections: Vec<Section>) -> ProcessingResults {
    let results: Vec<_> = sections
        .into_iter() // Move each section, no cloning
        .map(process_single_section) // Consume each section
        .collect();
    
    ProcessingResults { results }
}

// Alternative: Use references when original data is needed elsewhere
pub fn process_sections_borrowed(sections: &[Section]) -> ProcessingResults {
    let results: Vec<_> = sections
        .iter() // Borrow each section
        .map(|section| process_single_section_ref(section)) // Process by reference
        .collect();
    
    ProcessingResults { results }
}
```

### Async Gotcha: Blocking Operations
- **Problem**: Accidentally blocking the async runtime with synchronous operations
- **Cause**: Using synchronous I/O or CPU-intensive operations in async context
- **Solution**: Use async alternatives or spawn_blocking for CPU-bound work
- **Example**:
```rust
// WRONG: Blocking the async runtime
pub async fn process_file_async(path: &Path) -> Result<ProcessingResults> {
    // This blocks the entire async thread!
    let content = std::fs::read_to_string(path)?;
    
    // This also blocks with heavy computation!
    let processed = expensive_parsing_operation(&content);
    
    Ok(ProcessingResults { data: processed })
}

// CORRECT: Non-blocking async operations
pub async fn process_file_async(path: &Path) -> Result<ProcessingResults> {
    // Non-blocking async file I/O
    let content = tokio::fs::read_to_string(path).await?;
    
    // Move CPU-intensive work to thread pool
    let processed = tokio::task::spawn_blocking(move || {
        expensive_parsing_operation(&content)
    }).await??;
    
    Ok(ProcessingResults { data: processed })
}

// Better: Use streaming for large files
pub async fn process_large_file_streaming(path: &Path) -> Result<ProcessingResults> {
    let file = tokio::fs::File::open(path).await?;
    let mut reader = tokio::io::BufReader::new(file);
    let mut results = Vec::new();
    
    let mut line_buffer = String::new();
    while reader.read_line(&mut line_buffer).await? > 0 {
        // Process line-by-line without loading entire file
        let line_result = process_line_async(&line_buffer).await?;
        results.push(line_result);
        line_buffer.clear();
    }
    
    Ok(ProcessingResults { data: results })
}
```

## Best Practices

### Memory-Efficient String Processing
```rust
/// Efficient string processing patterns for large files
pub mod string_processing {
    use std::borrow::Cow;
    
    /// Process strings in-place when possible, allocate only when necessary
    pub fn process_line_efficient(line: &str, config: &ProcessingConfig) -> Cow<'_, str> {
        if !config.needs_modification() {
            // No processing needed - return borrowed string
            return Cow::Borrowed(line);
        }
        
        let mut modifications_needed = false;
        let mut result = line;
        
        // Check if any modifications are actually needed
        if config.trim_whitespace && (line.starts_with(' ') || line.ends_with(' ')) {
            modifications_needed = true;
        }
        
        if config.sanitize_paths && line.contains('/') {
            modifications_needed = true;
        }
        
        if !modifications_needed {
            return Cow::Borrowed(line);
        }
        
        // Only allocate if we actually need to modify
        let mut owned = line.to_string();
        
        if config.trim_whitespace {
            owned = owned.trim().to_string();
        }
        
        if config.sanitize_paths {
            owned = owned.replace('/', "_");
        }
        
        Cow::Owned(owned)
    }
    
    /// Bulk string processing with reusable buffer
    pub struct BulkStringProcessor {
        buffer: String,
        temp_buffer: String,
    }
    
    impl BulkStringProcessor {
        pub fn new() -> Self {
            Self {
                buffer: String::with_capacity(1024),
                temp_buffer: String::with_capacity(1024),
            }
        }
        
        /// Process multiple lines reusing internal buffers
        pub fn process_lines<'a>(
            &mut self, 
            lines: impl Iterator<Item = &'a str>,
            config: &ProcessingConfig,
        ) -> Vec<String> {
            let mut results = Vec::new();
            
            for line in lines {
                // Reuse buffer instead of allocating for each line
                self.buffer.clear();
                self.buffer.push_str(line);
                
                if config.trim_whitespace {
                    let trimmed = self.buffer.trim();
                    if trimmed.len() != self.buffer.len() {
                        self.temp_buffer.clear();
                        self.temp_buffer.push_str(trimmed);
                        std::mem::swap(&mut self.buffer, &mut self.temp_buffer);
                    }
                }
                
                if config.sanitize_paths {
                    // In-place replacement when possible
                    if self.buffer.contains('/') {
                        self.temp_buffer.clear();
                        self.temp_buffer.push_str(&self.buffer.replace('/', "_"));
                        std::mem::swap(&mut self.buffer, &mut self.temp_buffer);
                    }
                }
                
                results.push(self.buffer.clone()); // Final allocation
            }
            
            results
        }
    }
}
```

### Type-Driven Development
```rust
/// Use Rust's type system to enforce correctness at compile time
pub mod type_safety {
    use std::marker::PhantomData;
    
    /// Phantom types for compile-time state tracking
    pub struct Validated;
    pub struct Unvalidated;
    
    /// Configuration that tracks validation state at compile time
    pub struct ProcessingConfig<State = Unvalidated> {
        buffer_size: usize,
        max_memory: usize,
        output_format: OutputFormat,
        _state: PhantomData<State>,
    }
    
    impl ProcessingConfig<Unvalidated> {
        pub fn new() -> Self {
            Self {
                buffer_size: 64 * 1024,
                max_memory: 500 * 1024 * 1024,
                output_format: OutputFormat::Standard,
                _state: PhantomData,
            }
        }
        
        /// Validate configuration and transition to validated state
        pub fn validate(self) -> Result<ProcessingConfig<Validated>, ConfigError> {
            if self.buffer_size < 1024 {
                return Err(ConfigError::BufferTooSmall { 
                    size: self.buffer_size 
                });
            }
            
            if self.max_memory < 1024 * 1024 {
                return Err(ConfigError::MemoryLimitTooSmall { 
                    limit: self.max_memory 
                });
            }
            
            Ok(ProcessingConfig {
                buffer_size: self.buffer_size,
                max_memory: self.max_memory,
                output_format: self.output_format,
                _state: PhantomData,
            })
        }
    }
    
    impl ProcessingConfig<Validated> {
        /// Only validated configs can be used for processing
        pub fn buffer_size(&self) -> usize {
            self.buffer_size
        }
        
        pub fn max_memory(&self) -> usize {
            self.max_memory
        }
    }
    
    /// Processor only accepts validated configuration
    pub struct CpinfoProcessor;
    
    impl CpinfoProcessor {
        /// Compile-time guarantee that configuration is valid
        pub fn new(config: ProcessingConfig<Validated>) -> Self {
            tracing::info!("Creating processor with validated config",
                buffer_size = config.buffer_size(),
                max_memory_mb = config.max_memory() / 1024 / 1024
            );
            Self
        }
        
        // This won't compile - unvalidated config rejected at compile time
        // pub fn new_invalid(config: ProcessingConfig<Unvalidated>) -> Self { ... }
    }
    
    /// Usage pattern enforces validation
    pub fn example_usage() -> Result<(), ConfigError> {
        let config = ProcessingConfig::new()
            .validate()?; // Must validate before use
            
        let processor = CpinfoProcessor::new(config); // Compile-time safety
        
        Ok(())
    }
}
```

### Iterator-Based Processing Patterns
```rust
/// Efficient iterator chains for data processing
pub mod iterator_patterns {
    use std::collections::HashMap;
    
    /// Chain iterators for efficient pipeline processing
    pub fn process_file_pipeline(
        content: &str,
        config: &ProcessingConfig,
    ) -> impl Iterator<Item = ProcessedSection> + '_ {
        content
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty()) // Skip empty lines
            .scan(ParserState::new(), |state, (line_num, line)| {
                // Stateful parsing using scan
                match state.process_line(line, line_num) {
                    Ok(Some(section)) => Some(Some(section)),
                    Ok(None) => Some(None), // Continue processing
                    Err(_) => None, // Stop on error
                }
            })
            .flatten() // Remove None values
            .filter(|section| config.section_filter.matches(&section.name))
            .map(|section| section.apply_transformations(config))
    }
    
    /// Grouped processing for related sections
    pub fn group_sections_by_type(
        sections: impl Iterator<Item = ProcessedSection>,
    ) -> HashMap<SectionType, Vec<ProcessedSection>> {
        sections.fold(HashMap::new(), |mut acc, section| {
            acc.entry(section.section_type.clone())
                .or_default()
                .push(section);
            acc
        })
    }
    
    /// Parallel iterator processing for CPU-intensive operations
    pub fn process_sections_parallel(
        sections: Vec<ProcessedSection>,
        config: &ProcessingConfig,
    ) -> Vec<ProcessedSection> {
        use rayon::prelude::*;
        
        sections
            .into_par_iter() // Parallel iterator
            .filter(|section| !section.is_empty())
            .map(|section| {
                // CPU-intensive processing per section
                let mut processed = section;
                if config.analyze_content {
                    processed.analysis = Some(analyze_section_content(&processed.content));
                }
                if config.extract_metadata {
                    processed.metadata = extract_section_metadata(&processed);
                }
                processed
            })
            .collect()
    }
    
    /// Memory-efficient processing with itertools
    pub fn process_large_sections_chunked<'a>(
        sections: impl Iterator<Item = &'a ProcessedSection> + 'a,
        chunk_size: usize,
    ) -> impl Iterator<Item = ProcessingResult> + 'a {
        use itertools::Itertools;
        
        sections
            .chunks(chunk_size)
            .into_iter()
            .map(move |chunk| {
                let sections_chunk: Vec<_> = chunk.collect();
                ProcessingResult {
                    section_count: sections_chunk.len(),
                    total_size: sections_chunk.iter().map(|s| s.content.len()).sum(),
                    processing_time: std::time::Instant::now(),
                }
            })
    }
}
```

## Integration Points

### Serde Integration for Configuration
```rust
/// Serde-powered configuration with custom serialization
use serde::{Deserialize, Serialize, Deserializer, Serializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    
    #[serde(default = "default_memory_limit")]
    pub memory_limit: usize,
    
    #[serde(with = "duration_serde")]
    pub timeout: std::time::Duration,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_feature: Option<FeatureConfig>,
    
    // Custom serialization for complex types
    #[serde(serialize_with = "serialize_path", deserialize_with = "deserialize_path")]
    pub output_directory: PathBuf,
}

/// Custom serialization for Duration as seconds
mod duration_serde {
    use super::*;
    
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

/// Custom validation during deserialization
impl AppConfig {
    /// Validate configuration after deserialization
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.buffer_size < 1024 {
            return Err(ConfigError::InvalidValue {
                field: "buffer_size".to_string(),
                value: self.buffer_size.to_string(),
                reason: "Must be at least 1024 bytes".to_string(),
            });
        }
        
        if self.timeout.as_secs() == 0 {
            return Err(ConfigError::InvalidValue {
                field: "timeout".to_string(),
                value: format!("{}ms", self.timeout.as_millis()),
                reason: "Timeout must be greater than 0".to_string(),
            });
        }
        
        Ok(())
    }
}

// Default value providers
fn default_buffer_size() -> usize { 64 * 1024 }
fn default_memory_limit() -> usize { 500 * 1024 * 1024 }
```

### Tracing Integration for Observability
```rust
/// Structured logging with tracing for debugging and monitoring
pub mod logging {
    use tracing::{instrument, info, warn, error, debug, span, Level};
    
    /// Instrument functions for automatic tracing
    #[instrument(skip(content), fields(content_size = content.len()))]
    pub async fn process_content_traced(
        content: &str,
        config: &ProcessingConfig,
    ) -> Result<ProcessingResults, ProcessingError> {
        let span = span!(Level::INFO, "content_processing", 
            config_type = ?config.output_format,
            buffer_size = config.buffer_size
        );
        
        let _enter = span.enter();
        
        info!("Starting content processing");
        
        // Nested spans for detailed tracing
        let parsing_span = span!(Level::DEBUG, "parsing_phase");
        let sections = {
            let _parsing_enter = parsing_span.enter();
            debug!("Beginning parsing phase");
            
            let sections = parse_sections(content, config).await?;
            
            debug!("Parsing complete", section_count = sections.len());
            sections
        };
        
        // Error context in logs
        let results = organize_sections(sections, config).await
            .map_err(|e| {
                error!("Section organization failed", error = %e);
                e
            })?;
        
        info!("Content processing complete", 
            sections_processed = results.sections.len(),
            output_files = results.output_files.len()
        );
        
        Ok(results)
    }
    
    /// Custom tracing subscriber for CLI applications
    pub fn setup_cli_tracing(config: &LoggingConfig) -> Result<(), TracingError> {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, fmt};
        
        let format_layer = fmt::layer()
            .with_target(false)
            .with_thread_ids(config.include_thread_ids)
            .with_file(config.include_file_locations)
            .compact();
        
        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(&config.default_level))
            .map_err(TracingError::FilterParseError)?;
        
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(format_layer)
            .init();
        
        Ok(())
    }
}
```

## Troubleshooting

### Rust-Specific Debugging Techniques
```rust
/// Debug utilities for Rust-specific issues
pub mod rust_debug {
    /// Debug lifetime issues with explicit annotations
    pub fn debug_lifetime_issues() {
        // Use explicit lifetime annotations to understand relationships
        fn explicit_lifetimes<'content, 'config>(
            content: &'content str,
            config: &'config ProcessingConfig,
        ) -> Result<Parser<'content>, ParseError> {
            // 'content must live as long as the returned Parser
            Ok(Parser::new(content))
        }
        
        // Alternative: use owned types to avoid lifetime complexity
        fn owned_alternative(
            content: String,
            config: ProcessingConfig,
        ) -> Result<OwnedParser, ParseError> {
            Ok(OwnedParser::new(content))
        }
    }
    
    /// Debug async context issues
    pub async fn debug_async_issues() {
        // Check if futures are Send + Sync when needed
        fn require_send_sync<T: Send + Sync>(t: T) -> T { t }
        
        let future = async move {
            // This future must be Send + Sync for tokio::spawn
            process_file_async(Path::new("test.txt")).await
        };
        
        // This will fail to compile if future is not Send + Sync
        let handle = tokio::spawn(require_send_sync(future));
        handle.await.unwrap().unwrap();
    }
    
    /// Debug performance issues with timing
    pub async fn debug_performance_issues() {
        use std::time::Instant;
        
        let start = Instant::now();
        
        // Measure individual operations
        let parsing_start = Instant::now();
        let sections = parse_sections("content").await?;
        let parsing_time = parsing_start.elapsed();
        
        let organizing_start = Instant::now();
        let results = organize_sections(sections).await?;
        let organizing_time = organizing_start.elapsed();
        
        let total_time = start.elapsed();
        
        eprintln!("Performance breakdown:");
        eprintln!("  Parsing: {:?} ({:.1}%)", parsing_time, 
            parsing_time.as_secs_f64() / total_time.as_secs_f64() * 100.0);
        eprintln!("  Organizing: {:?} ({:.1}%)", organizing_time,
            organizing_time.as_secs_f64() / total_time.as_secs_f64() * 100.0);
        eprintln!("  Total: {:?}", total_time);
        
        Ok(results)
    }
}
```

### Memory Usage Analysis
```rust
/// Memory profiling utilities for Rust applications
pub mod memory_profiling {
    /// Track memory allocations in critical sections
    pub fn track_memory_usage<F, R>(operation_name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let initial_memory = get_process_memory_usage().unwrap_or(0);
        
        let result = f();
        
        let final_memory = get_process_memory_usage().unwrap_or(0);
        let memory_delta = final_memory.saturating_sub(initial_memory);
        
        if memory_delta > 10 * 1024 * 1024 { // >10MB allocation
            eprintln!("Large memory allocation in {}: {}MB", 
                operation_name, 
                memory_delta / 1024 / 1024
            );
        }
        
        result
    }
    
    /// Monitor for memory leaks in long-running operations
    pub async fn monitor_memory_leaks() {
        let initial_memory = get_process_memory_usage().unwrap_or(0);
        let mut previous_memory = initial_memory;
        let mut leak_warnings = 0;
        
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            
            let current_memory = get_process_memory_usage().unwrap_or(0);
            let growth = current_memory.saturating_sub(previous_memory);
            let total_growth = current_memory.saturating_sub(initial_memory);
            
            if growth > 50 * 1024 * 1024 { // >50MB growth in 1 minute
                leak_warnings += 1;
                eprintln!("Potential memory leak detected: +{}MB in last minute, +{}MB total",
                    growth / 1024 / 1024,
                    total_growth / 1024 / 1024
                );
                
                if leak_warnings > 5 {
                    eprintln!("Multiple memory leak warnings - consider investigating");
                }
            }
            
            previous_memory = current_memory;
        }
    }
}
```

## References

- [The Rust Programming Language](https://doc.rust-lang.org/book/) - Comprehensive Rust guide
- [Rust by Example](https://doc.rust-lang.org/stable/rust-by-example/) - Practical examples  
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) - API design best practices
- [The Async Book](https://rust-lang.github.io/async-book/) - Async programming guide
- [Rust Performance Book](https://nnethercote.github.io/perf-book/) - Performance optimization
- [Architecture Document](architecture.md) - System design context
- [Quality Standards](quality-standards.md) - Rust-specific quality requirements