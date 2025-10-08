# Performance Optimization Patterns - cpinfo-parser

## Overview
- **Purpose**: Performance optimization strategies and patterns for the cpinfo-parser
- **Use Cases**: Phase 3 performance optimization implementation
- **Version**: Rust 1.75+ with tokio async runtime
- **Last Updated**: 2025-08-08

## Key Concepts

### Performance Philosophy  
The cpinfo-parser targets **enterprise-grade performance** with **predictable resource usage**:
- **Streaming Architecture**: Constant memory usage regardless of file size
- **Zero-Copy Operations**: Minimize allocations and data copying
- **Async Efficiency**: Non-blocking I/O with optimal concurrency
- **Predictable Scaling**: Linear time complexity, constant memory complexity

### Performance Targets
- **Throughput**: >400MB/s for large files (>100MB)
- **Memory Usage**: <500MB total regardless of file size  
- **Startup Time**: <100ms cold start
- **CPU Efficiency**: <50% CPU usage during processing

## Implementation Patterns

### Primary Pattern: Streaming Buffer Management

```rust
/// High-performance streaming reader with adaptive buffering
pub struct StreamingCpinfoReader<R> {
    inner: BufReader<R>,
    buffer: CircularBuffer,
    position: u64,
    file_size: Option<u64>,
    buffer_size_strategy: BufferSizeStrategy,
}

/// Adaptive buffer sizing based on file characteristics
#[derive(Debug, Clone)]
pub enum BufferSizeStrategy {
    Fixed(usize),
    Adaptive { min: usize, max: usize, target_latency_ms: u64 },
    FileSize { small: usize, medium: usize, large: usize },
}

impl<R: Read> StreamingCpinfoReader<R> {
    const DEFAULT_BUFFER_SIZE: usize = 64 * 1024;    // 64KB baseline
    const LARGE_FILE_BUFFER: usize = 256 * 1024;     // 256KB for large files
    const SMALL_FILE_BUFFER: usize = 8 * 1024;       // 8KB for small files
    
    pub fn with_adaptive_buffering(reader: R, file_size: Option<u64>) -> Self {
        let buffer_size = Self::calculate_optimal_buffer_size(file_size);
        let buffer = CircularBuffer::with_capacity(buffer_size);
        
        Self {
            inner: BufReader::with_capacity(buffer_size, reader),
            buffer,
            position: 0,
            file_size,
            buffer_size_strategy: BufferSizeStrategy::Adaptive {
                min: Self::SMALL_FILE_BUFFER,
                max: Self::LARGE_FILE_BUFFER,
                target_latency_ms: 10,
            },
        }
    }
    
    fn calculate_optimal_buffer_size(file_size: Option<u64>) -> usize {
        match file_size {
            Some(size) => match size {
                0..=1_048_576 => Self::SMALL_FILE_BUFFER,        // <1MB: 8KB buffer
                1_048_577..=104_857_600 => Self::DEFAULT_BUFFER_SIZE,  // 1-100MB: 64KB buffer  
                _ => Self::LARGE_FILE_BUFFER,                    // >100MB: 256KB buffer
            },
            None => Self::DEFAULT_BUFFER_SIZE, // Unknown size: default
        }
    }
    
    /// Zero-copy line reading with delimiter detection
    pub fn read_until_delimiter(&mut self) -> Result<Option<SectionBoundary>, ReaderError> {
        let mut line_buffer = Vec::with_capacity(128); // Most lines <128 chars
        let mut delimiter_state = DelimiterState::Searching;
        
        loop {
            let bytes_read = self.inner.read_until(b'\n', &mut line_buffer)?;
            if bytes_read == 0 {
                return Ok(None); // EOF
            }
            
            self.position += bytes_read as u64;
            
            // Fast delimiter detection without string allocation
            if let Some(boundary) = self.detect_delimiter_fast(&line_buffer)? {
                return Ok(Some(boundary));
            }
            
            // Prevent unbounded memory growth
            if line_buffer.len() > 1024 * 1024 { // 1MB line limit
                return Err(ReaderError::LineTooLong {
                    length: line_buffer.len(),
                    position: self.position,
                });
            }
        }
    }
    
    /// Fast delimiter detection using byte-level operations
    fn detect_delimiter_fast(&self, line: &[u8]) -> Result<Option<SectionBoundary>, ReaderError> {
        // Skip whitespace at start and end
        let trimmed = trim_ascii_whitespace(line);
        
        // Fast path: check for exact delimiter patterns using SIMD-friendly operations
        match trimmed.len() {
            24 => {
                if trimmed.iter().all(|&b| b == b'=') {
                    return Ok(Some(SectionBoundary::Command24));
                }
                if trimmed.iter().all(|&b| b == b'-') {
                    return Ok(Some(SectionBoundary::Command24Dash));
                }
            },
            66 => {
                if trimmed.iter().all(|&b| b == b'=') {
                    return Ok(Some(SectionBoundary::File66));
                }
            },
            _ => {}
        }
        
        Ok(None)
    }
    
    /// Memory usage monitoring and adaptive buffer resizing
    pub fn monitor_and_adjust_buffer(&mut self) -> Result<(), ReaderError> {
        let current_memory = get_process_memory_usage()?;
        let buffer_memory = self.buffer.capacity() + self.inner.capacity();
        
        // If memory usage is high, reduce buffer size
        if current_memory > 400 * 1024 * 1024 { // 400MB threshold
            let new_size = (buffer_memory / 2).max(Self::SMALL_FILE_BUFFER);
            self.resize_buffers(new_size)?;
            tracing::warn!("Reduced buffer size due to memory pressure", 
                old_size = buffer_memory, 
                new_size = new_size,
                total_memory_mb = current_memory / 1024 / 1024
            );
        }
        
        Ok(())
    }
}

/// High-performance circular buffer for streaming operations
pub struct CircularBuffer {
    buffer: Box<[u8]>,      // Heap-allocated for large buffers
    read_pos: usize,
    write_pos: usize,
    size: usize,
    capacity: usize,
}

impl CircularBuffer {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: vec![0u8; capacity].into_boxed_slice(),
            read_pos: 0,
            write_pos: 0,
            size: 0,
            capacity,
        }
    }
    
    /// Zero-copy read operation returning buffer slice
    pub fn read_slice(&mut self, len: usize) -> Option<&[u8]> {
        if self.size < len {
            return None;
        }
        
        let end_pos = (self.read_pos + len).min(self.capacity);
        let slice = &self.buffer[self.read_pos..end_pos];
        
        self.consume(len);
        Some(slice)
    }
    
    /// Fast consume operation with wrapping
    pub fn consume(&mut self, amount: usize) {
        debug_assert!(amount <= self.size);
        self.read_pos = (self.read_pos + amount) % self.capacity;
        self.size -= amount;
    }
    
    /// Efficient write operation with overflow protection  
    pub fn write_slice(&mut self, data: &[u8]) -> Result<usize, BufferError> {
        let available = self.capacity - self.size;
        if available == 0 {
            return Ok(0);
        }
        
        let to_write = data.len().min(available);
        let end_pos = (self.write_pos + to_write).min(self.capacity);
        
        self.buffer[self.write_pos..end_pos].copy_from_slice(&data[..to_write]);
        self.write_pos = (self.write_pos + to_write) % self.capacity;
        self.size += to_write;
        
        Ok(to_write)
    }
}
```

### Alternative Pattern: Async Parallel Processing  

```rust
/// High-performance async batch processor with controlled concurrency
pub struct AsyncBatchProcessor {
    semaphore: Arc<Semaphore>,
    thread_pool: Runtime,
    progress_reporter: Arc<dyn ProgressReporter + Send + Sync>,
    memory_monitor: MemoryMonitor,
}

impl AsyncBatchProcessor {
    pub fn new(max_concurrent: usize) -> Self {
        let thread_pool = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(num_cpus::get())
            .enable_all()
            .thread_name("cpinfo-worker")
            .thread_stack_size(2 * 1024 * 1024) // 2MB stack
            .build()
            .expect("Failed to create async runtime");
            
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            thread_pool,
            progress_reporter: Arc::new(ConsoleProgressReporter::new()),
            memory_monitor: MemoryMonitor::new(),
        }
    }
    
    /// Process multiple files with controlled concurrency and memory management
    pub async fn process_files_parallel(
        &self,
        files: Vec<PathBuf>,
        config: ProcessingConfig,
    ) -> Result<BatchResults, ProcessingError> {
        let total_files = files.len();
        let mut tasks = Vec::with_capacity(total_files);
        let results = Arc::new(Mutex::new(Vec::new()));
        
        // Create processing tasks with semaphore-controlled concurrency
        for (index, file_path) in files.into_iter().enumerate() {
            let semaphore = Arc::clone(&self.semaphore);
            let config = config.clone();
            let progress = Arc::clone(&self.progress_reporter);
            let results = Arc::clone(&results);
            let memory_monitor = self.memory_monitor.clone();
            
            let task = self.thread_pool.spawn(async move {
                // Acquire semaphore permit for controlled concurrency
                let _permit = semaphore.acquire().await
                    .map_err(|_| ProcessingError::ConcurrencyError)?;
                
                // Memory pressure check before processing
                if memory_monitor.check_memory_pressure()? {
                    tracing::warn!("Memory pressure detected, throttling processing");
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                
                // Process individual file with progress reporting
                let file_results = Self::process_single_file_optimized(
                    &file_path, 
                    &config,
                    index,
                    total_files,
                    progress.as_ref()
                ).await?;
                
                // Thread-safe results collection
                results.lock().await.push(file_results);
                
                Ok::<_, ProcessingError>(())
            });
            
            tasks.push(task);
        }
        
        // Efficient task completion with error aggregation
        let mut errors = Vec::new();
        for task in tasks {
            if let Err(e) = task.await {
                if let Ok(processing_error) = e.try_into_panic() {
                    errors.push(ProcessingError::TaskPanic {
                        details: format!("{:?}", processing_error),
                    });
                } else {
                    errors.push(ProcessingError::TaskJoinError);
                }
            }
        }
        
        let final_results = Arc::try_unwrap(results)
            .map_err(|_| ProcessingError::ResultsCollectionError)?
            .into_inner();
        
        Ok(BatchResults {
            individual_results: final_results,
            errors,
            total_processing_time: self.progress_reporter.total_elapsed(),
        })
    }
    
    /// Optimized single file processing with memory monitoring
    async fn process_single_file_optimized(
        file_path: &Path,
        config: &ProcessingConfig,
        file_index: usize,
        total_files: usize,
        progress: &dyn ProgressReporter,
    ) -> Result<ProcessingResults, ProcessingError> {
        let start_time = Instant::now();
        
        // Pre-flight memory check
        let initial_memory = get_process_memory_usage()?;
        if initial_memory > config.memory_limit_bytes {
            return Err(ProcessingError::MemoryLimitExceeded {
                current_bytes: initial_memory,
                limit_bytes: config.memory_limit_bytes,
            });
        }
        
        // Optimized file reading with memory mapping for large files
        let file_size = tokio::fs::metadata(file_path).await?.len();
        let reader: Box<dyn AsyncRead + Unpin> = if file_size > 100 * 1024 * 1024 {
            // Large file: use memory mapping
            Box::new(MemoryMappedFile::open(file_path).await?)
        } else {
            // Small/medium file: use buffered reading
            Box::new(BufReader::new(File::open(file_path).await?))
        };
        
        let mut streaming_reader = StreamingCpinfoReader::new(reader);
        let mut section_count = 0;
        let mut total_bytes_processed = 0;
        
        // Streaming processing loop with regular memory checks
        while let Some(section_data) = streaming_reader.read_section_optimized().await? {
            section_count += 1;
            total_bytes_processed += section_data.size_bytes;
            
            // Process section with zero-copy operations where possible
            process_section_zero_copy(&section_data, config).await?;
            
            // Memory monitoring every 100 sections
            if section_count % 100 == 0 {
                let current_memory = get_process_memory_usage()?;
                if current_memory > config.memory_limit_bytes {
                    // Force garbage collection before failing
                    force_garbage_collection();
                    
                    let post_gc_memory = get_process_memory_usage()?;
                    if post_gc_memory > config.memory_limit_bytes {
                        return Err(ProcessingError::MemoryLimitExceeded {
                            current_bytes: post_gc_memory,
                            limit_bytes: config.memory_limit_bytes,
                        });
                    }
                }
                
                // Progress reporting with performance metrics
                progress.update_file_progress(FileProgress {
                    file_index,
                    total_files,
                    sections_processed: section_count,
                    bytes_processed: total_bytes_processed,
                    processing_rate_mbps: calculate_processing_rate(
                        total_bytes_processed, 
                        start_time.elapsed()
                    ),
                    memory_usage_mb: current_memory / 1024 / 1024,
                });
            }
        }
        
        let processing_time = start_time.elapsed();
        Ok(ProcessingResults {
            file_path: file_path.to_owned(),
            sections_extracted: section_count,
            bytes_processed: total_bytes_processed,
            processing_time,
            peak_memory_usage: get_peak_memory_usage()?,
            average_throughput_mbps: calculate_processing_rate(total_bytes_processed, processing_time),
        })
    }
}
```

### Memory-Efficient Section Processing

```rust
/// Zero-copy section processing with minimal allocations
pub async fn process_section_zero_copy(
    section_data: &SectionData<'_>,
    config: &ProcessingConfig,
) -> Result<(), ProcessingError> {
    // Use string slices instead of owned strings where possible
    let section_name = section_data.name(); // Returns &str, not String
    let content_lines = section_data.content_lines(); // Iterator, not Vec
    
    // Streaming output writer to avoid buffering entire section
    let mut output_writer = create_streaming_writer(section_name, config).await?;
    
    // Process lines in streaming fashion
    for line in content_lines {
        // Fast path: write line directly if no processing needed
        if !config.requires_line_processing {
            output_writer.write_line_raw(line.as_bytes()).await?;
            continue;
        }
        
        // Minimal processing path with in-place operations
        let processed_line = process_line_in_place(line, config)?;
        output_writer.write_line_processed(&processed_line).await?;
    }
    
    output_writer.flush().await?;
    Ok(())
}

/// In-place line processing to minimize allocations
fn process_line_in_place(line: &str, config: &ProcessingConfig) -> Result<Cow<'_, str>, ProcessingError> {
    if !config.sanitize_content && !config.filter_sensitive {
        // No processing needed - return borrowed string
        return Ok(Cow::Borrowed(line));
    }
    
    let mut owned_line = None;
    
    // Only allocate if we need to modify the line
    if config.sanitize_content && contains_sanitization_targets(line) {
        let sanitized = sanitize_line_content(line);
        owned_line = Some(sanitized);
    }
    
    if config.filter_sensitive && contains_sensitive_patterns(line) {
        let line_to_filter = owned_line.as_deref().unwrap_or(line);
        let filtered = filter_sensitive_content(line_to_filter);
        owned_line = Some(filtered);
    }
    
    match owned_line {
        Some(processed) => Ok(Cow::Owned(processed)),
        None => Ok(Cow::Borrowed(line)),
    }
}

/// High-performance streaming output writer
pub struct StreamingOutputWriter {
    writer: BufWriter<File>,
    buffer_size: usize,
    bytes_written: u64,
}

impl StreamingOutputWriter {
    const OUTPUT_BUFFER_SIZE: usize = 128 * 1024; // 128KB output buffer
    
    pub async fn new(output_path: &Path) -> Result<Self, std::io::Error> {
        let file = File::create(output_path).await?;
        let writer = BufWriter::with_capacity(Self::OUTPUT_BUFFER_SIZE, file);
        
        Ok(Self {
            writer,
            buffer_size: Self::OUTPUT_BUFFER_SIZE,
            bytes_written: 0,
        })
    }
    
    /// Write raw bytes without UTF-8 validation (fastest path)
    pub async fn write_line_raw(&mut self, line_bytes: &[u8]) -> Result<(), std::io::Error> {
        self.writer.write_all(line_bytes).await?;
        self.writer.write_all(b"\n").await?;
        self.bytes_written += line_bytes.len() as u64 + 1;
        Ok(())
    }
    
    /// Write processed string content
    pub async fn write_line_processed(&mut self, line: &str) -> Result<(), std::io::Error> {
        self.writer.write_all(line.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.bytes_written += line.len() as u64 + 1;
        Ok(())
    }
    
    /// Flush with performance monitoring
    pub async fn flush(&mut self) -> Result<(), std::io::Error> {
        let start = Instant::now();
        self.writer.flush().await?;
        
        let flush_time = start.elapsed();
        if flush_time > Duration::from_millis(100) {
            tracing::warn!("Slow flush detected", 
                flush_time_ms = flush_time.as_millis(),
                bytes_written = self.bytes_written
            );
        }
        
        Ok(())
    }
}
```

## Common Gotchas

### Critical Gotcha: Memory Leaks in Async Processing
- **Problem**: Async tasks hold references longer than expected, preventing memory cleanup
- **Cause**: Shared ownership with Arc/Mutex without proper cleanup
- **Solution**: Use weak references where appropriate, explicit cleanup in task completion
- **Example**:
```rust
// WRONG: Strong references prevent cleanup
pub struct BatchProcessor {
    active_tasks: Arc<Mutex<Vec<Arc<ProcessingTask>>>>,
}

impl BatchProcessor {
    async fn process_files(&self, files: Vec<PathBuf>) -> Result<()> {
        let mut tasks = Vec::new();
        
        for file in files {
            let task = Arc::new(ProcessingTask::new(file));
            self.active_tasks.lock().await.push(Arc::clone(&task));
            
            let task_handle = tokio::spawn(async move {
                task.process().await
            });
            
            tasks.push(task_handle);
        }
        
        // Tasks hold strong references - memory never released!
        futures::future::try_join_all(tasks).await?;
        Ok(())
    }
}

// CORRECT: Explicit cleanup with weak references
pub struct BatchProcessor {
    active_tasks: Arc<Mutex<Vec<Weak<ProcessingTask>>>>,
}

impl BatchProcessor {
    async fn process_files(&self, files: Vec<PathBuf>) -> Result<()> {
        let mut tasks = Vec::new();
        
        for file in files {
            let task = Arc::new(ProcessingTask::new(file));
            self.active_tasks.lock().await.push(Arc::downgrade(&task));
            
            let task_handle = tokio::spawn(async move {
                let result = task.process().await;
                // task is dropped here, memory can be reclaimed
                result
            });
            
            tasks.push(task_handle);
        }
        
        let results = futures::future::try_join_all(tasks).await?;
        
        // Clean up dead weak references
        self.active_tasks.lock().await.retain(|weak_ref| weak_ref.strong_count() > 0);
        
        Ok(())
    }
}
```

### Performance Gotcha: Blocking Operations in Async Context
- **Problem**: Blocking I/O operations block the entire async runtime thread
- **Cause**: Using synchronous I/O operations inside async functions
- **Solution**: Use async I/O exclusively, spawn_blocking for CPU-intensive work
- **Example**:
```rust
// WRONG: Blocking I/O in async context
pub async fn process_file_async(path: &Path) -> Result<ProcessingResults> {
    // This blocks the entire async thread!
    let content = std::fs::read_to_string(path)?;
    
    // This also blocks!
    let processed = expensive_cpu_processing(&content);
    
    Ok(ProcessingResults { content: processed })
}

// CORRECT: Non-blocking async operations
pub async fn process_file_async(path: &Path) -> Result<ProcessingResults> {
    // Non-blocking async file I/O
    let content = tokio::fs::read_to_string(path).await?;
    
    // CPU-intensive work on thread pool
    let processed = tokio::task::spawn_blocking(move || {
        expensive_cpu_processing(&content)
    }).await??;
    
    Ok(ProcessingResults { content: processed })
}
```

### Buffer Management Gotcha: Memory Fragmentation
- **Problem**: Frequent buffer allocations/deallocations cause memory fragmentation
- **Cause**: Creating new buffers for each operation instead of reusing
- **Solution**: Use buffer pools and pre-allocated buffers
- **Example**:
```rust
// WRONG: Frequent allocations
pub async fn process_sections(sections: &[SectionData]) -> Result<()> {
    for section in sections {
        let mut buffer = Vec::new(); // New allocation each time!
        buffer.extend_from_slice(section.raw_data());
        process_buffer(&buffer).await?;
        // Buffer dropped - memory fragmentation
    }
    Ok(())
}

// CORRECT: Buffer reuse with pool
pub struct BufferPool {
    available_buffers: Mutex<Vec<Vec<u8>>>,
    buffer_size: usize,
}

impl BufferPool {
    pub fn acquire_buffer(&self) -> Vec<u8> {
        let mut available = self.available_buffers.lock().unwrap();
        match available.pop() {
            Some(mut buffer) => {
                buffer.clear(); // Reuse existing allocation
                buffer
            }
            None => Vec::with_capacity(self.buffer_size), // New allocation only if needed
        }
    }
    
    pub fn return_buffer(&self, buffer: Vec<u8>) {
        if buffer.capacity() == self.buffer_size {
            let mut available = self.available_buffers.lock().unwrap();
            available.push(buffer);
        }
        // Drop oversized buffers to prevent memory bloat
    }
}

pub async fn process_sections_optimized(
    sections: &[SectionData],
    buffer_pool: &BufferPool,
) -> Result<()> {
    for section in sections {
        let mut buffer = buffer_pool.acquire_buffer();
        buffer.extend_from_slice(section.raw_data());
        
        process_buffer(&buffer).await?;
        
        buffer_pool.return_buffer(buffer); // Return to pool for reuse
    }
    Ok(())
}
```

## Best Practices

### Performance Monitoring Integration
```rust
/// Comprehensive performance monitoring during processing
pub struct PerformanceMonitor {
    start_time: Instant,
    checkpoint_times: Vec<(String, Instant)>,
    memory_snapshots: Vec<(String, usize)>,
    throughput_calculator: ThroughputCalculator,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            checkpoint_times: Vec::new(),
            memory_snapshots: Vec::new(),
            throughput_calculator: ThroughputCalculator::new(),
        }
    }
    
    /// Record performance checkpoint with automatic memory snapshot
    pub fn checkpoint(&mut self, operation: &str) -> Result<(), MonitoringError> {
        let now = Instant::now();
        let memory_usage = get_process_memory_usage()?;
        
        self.checkpoint_times.push((operation.to_string(), now));
        self.memory_snapshots.push((operation.to_string(), memory_usage));
        
        // Log performance metrics
        tracing::info!("Performance checkpoint",
            operation = operation,
            elapsed_ms = (now - self.start_time).as_millis(),
            memory_mb = memory_usage / 1024 / 1024,
            throughput_mbps = self.throughput_calculator.current_rate()
        );
        
        Ok(())
    }
    
    /// Generate comprehensive performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let total_time = self.start_time.elapsed();
        let peak_memory = self.memory_snapshots.iter()
            .map(|(_, mem)| *mem)
            .max()
            .unwrap_or(0);
        
        PerformanceReport {
            total_processing_time: total_time,
            peak_memory_usage: peak_memory,
            average_throughput: self.throughput_calculator.average_rate(),
            checkpoint_details: self.checkpoint_times.clone(),
            memory_profile: self.memory_snapshots.clone(),
            performance_warnings: self.analyze_performance_issues(),
        }
    }
    
    fn analyze_performance_issues(&self) -> Vec<PerformanceWarning> {
        let mut warnings = Vec::new();
        
        // Check for memory growth trends
        if let Some(memory_growth_rate) = self.calculate_memory_growth_rate() {
            if memory_growth_rate > 10.0 { // >10MB/s growth
                warnings.push(PerformanceWarning::MemoryGrowth {
                    rate_mb_per_sec: memory_growth_rate,
                });
            }
        }
        
        // Check for slow operations
        for window in self.checkpoint_times.windows(2) {
            let duration = window[1].1 - window[0].1;
            if duration > Duration::from_secs(10) {
                warnings.push(PerformanceWarning::SlowOperation {
                    operation: window[1].0.clone(),
                    duration_ms: duration.as_millis() as u64,
                });
            }
        }
        
        warnings
    }
}
```

### Benchmark-Driven Development
```rust
/// Comprehensive benchmarking setup for performance validation
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};

fn benchmark_parsing_performance(c: &mut Criterion) {
    let test_files = generate_test_files(); // Various sizes: 8MB, 50MB, 200MB
    
    let mut group = c.benchmark_group("file_parsing");
    
    for (size_mb, test_file) in test_files {
        // Set throughput measurement
        group.throughput(Throughput::Bytes(size_mb * 1024 * 1024));
        
        group.bench_with_input(
            BenchmarkId::new("streaming_parser", size_mb),
            &test_file,
            |b, file_path| {
                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        process_file_streaming(black_box(file_path)).await.unwrap()
                    })
                });
            },
        );
        
        // Memory usage validation within benchmark
        group.bench_function(
            &format!("memory_usage_{}mb", size_mb),
            |b| {
                b.iter_custom(|iters| {
                    let start_memory = get_process_memory_usage().unwrap();
                    let start_time = Instant::now();
                    
                    for _i in 0..iters {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(async {
                            process_file_streaming(black_box(&test_file)).await.unwrap()
                        });
                    }
                    
                    let peak_memory = get_peak_memory_usage().unwrap();
                    let memory_growth = peak_memory - start_memory;
                    
                    // Fail benchmark if memory usage exceeds limits
                    assert!(memory_growth < 500 * 1024 * 1024, 
                        "Memory usage exceeded 500MB: {}MB", 
                        memory_growth / 1024 / 1024
                    );
                    
                    start_time.elapsed()
                });
            },
        );
    }
    
    group.finish();
}

/// Regression testing to ensure performance doesn't degrade
fn benchmark_regression_testing(c: &mut Criterion) {
    c.bench_function("regression_baseline_8mb", |b| {
        let test_file = create_8mb_test_file();
        b.iter(|| {
            let result = process_file_streaming(black_box(&test_file));
            // Baseline: should complete in <2 seconds
            assert!(result.processing_time < Duration::from_secs(2));
            result
        });
    });
}

criterion_group!(benches, benchmark_parsing_performance, benchmark_regression_testing);
criterion_main!(benches);
```

### Resource Management Strategy
```rust
/// Comprehensive resource management with automatic cleanup
pub struct ResourceManager {
    file_handles: Vec<File>,
    memory_buffers: Vec<Vec<u8>>,
    temp_files: Vec<PathBuf>,
    max_open_files: usize,
    max_memory_usage: usize,
}

impl ResourceManager {
    pub fn new(config: &ResourceConfig) -> Self {
        Self {
            file_handles: Vec::new(),
            memory_buffers: Vec::new(),
            temp_files: Vec::new(),
            max_open_files: config.max_open_files,
            max_memory_usage: config.max_memory_bytes,
        }
    }
    
    /// Acquire file handle with automatic cleanup
    pub async fn acquire_file_handle(&mut self, path: &Path) -> Result<File, ResourceError> {
        // Check resource limits
        if self.file_handles.len() >= self.max_open_files {
            self.cleanup_oldest_handles().await?;
        }
        
        let file = File::open(path).await
            .map_err(|e| ResourceError::FileOpenFailed { 
                path: path.to_owned(), 
                source: e 
            })?;
            
        self.file_handles.push(file);
        Ok(self.file_handles.last().unwrap())
    }
    
    /// Acquire memory buffer with size validation
    pub fn acquire_buffer(&mut self, size: usize) -> Result<Vec<u8>, ResourceError> {
        let current_memory = self.calculate_total_buffer_memory();
        
        if current_memory + size > self.max_memory_usage {
            self.cleanup_unused_buffers()?;
            
            let post_cleanup_memory = self.calculate_total_buffer_memory();
            if post_cleanup_memory + size > self.max_memory_usage {
                return Err(ResourceError::MemoryLimitExceeded {
                    requested: size,
                    available: self.max_memory_usage - post_cleanup_memory,
                });
            }
        }
        
        let buffer = Vec::with_capacity(size);
        self.memory_buffers.push(buffer);
        Ok(self.memory_buffers.last().unwrap())
    }
    
    /// Automatic resource cleanup on drop
    async fn cleanup_all_resources(&mut self) -> Result<(), ResourceError> {
        // Close all file handles
        for file in self.file_handles.drain(..) {
            drop(file);
        }
        
        // Clear memory buffers
        self.memory_buffers.clear();
        
        // Remove temporary files
        for temp_path in &self.temp_files {
            if temp_path.exists() {
                tokio::fs::remove_file(temp_path).await
                    .map_err(|e| ResourceError::TempFileCleanupFailed { 
                        path: temp_path.clone(), 
                        source: e 
                    })?;
            }
        }
        self.temp_files.clear();
        
        Ok(())
    }
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        // Best-effort cleanup on drop
        if !self.temp_files.is_empty() {
            tracing::warn!("Cleaning up {} temporary files on drop", self.temp_files.len());
            for temp_path in &self.temp_files {
                let _ = std::fs::remove_file(temp_path);
            }
        }
    }
}
```

## Integration Points

### Configuration Integration
```rust
/// Performance-focused configuration with validation
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PerformanceConfig {
    /// Buffer size for file I/O operations (bytes)
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    
    /// Maximum memory usage limit (bytes)  
    #[serde(default = "default_memory_limit")]
    pub memory_limit: usize,
    
    /// Number of concurrent processing threads
    #[serde(default = "default_thread_count")]
    pub thread_count: usize,
    
    /// Enable memory monitoring and adaptive buffering
    #[serde(default = "default_true")]
    pub adaptive_memory_management: bool,
    
    /// Performance monitoring interval (milliseconds)
    #[serde(default = "default_monitoring_interval")]
    pub monitoring_interval_ms: u64,
}

impl PerformanceConfig {
    /// Validate configuration values and adjust for system constraints
    pub fn validate_and_adjust(&mut self) -> Result<(), ConfigError> {
        // Validate buffer size
        if self.buffer_size < 1024 {
            return Err(ConfigError::InvalidValue {
                key: "buffer_size".to_string(),
                value: self.buffer_size.to_string(),
                reason: "Buffer size must be at least 1KB".to_string(),
            });
        }
        
        // Adjust thread count based on system capabilities
        let max_threads = num_cpus::get();
        if self.thread_count > max_threads {
            tracing::warn!("Reducing thread count from {} to {} based on CPU cores", 
                self.thread_count, max_threads);
            self.thread_count = max_threads;
        }
        
        // Validate memory limit against available system memory
        let available_memory = get_available_system_memory()?;
        if self.memory_limit > available_memory {
            tracing::warn!("Reducing memory limit from {}MB to {}MB based on available memory",
                self.memory_limit / 1024 / 1024,
                available_memory / 1024 / 1024
            );
            self.memory_limit = available_memory;
        }
        
        Ok(())
    }
}
```

## Troubleshooting

### Performance Debugging Tools
```rust
/// Advanced performance debugging utilities
pub mod perf_debug {
    use std::time::{Duration, Instant};
    
    /// Detailed performance profiler for identifying bottlenecks
    pub struct PerformanceProfiler {
        function_timings: HashMap<String, Vec<Duration>>,
        allocation_tracker: AllocationTracker,
        cpu_sampler: CpuSampler,
    }
    
    impl PerformanceProfiler {
        /// Profile function execution with detailed timing
        pub async fn profile_async<F, T>(&mut self, name: &str, f: F) -> T 
        where
            F: Future<Output = T>,
        {
            let start_memory = self.allocation_tracker.current_allocations();
            let start_time = Instant::now();
            
            let result = f.await;
            
            let duration = start_time.elapsed();
            let end_memory = self.allocation_tracker.current_allocations();
            
            self.function_timings.entry(name.to_string())
                .or_default()
                .push(duration);
            
            if end_memory > start_memory + 1024 * 1024 { // >1MB allocation
                tracing::warn!("Large memory allocation detected",
                    function = name,
                    allocated_mb = (end_memory - start_memory) / 1024 / 1024,
                    duration_ms = duration.as_millis()
                );
            }
            
            result
        }
        
        /// Generate hotspot analysis report
        pub fn generate_hotspot_report(&self) -> HotspotReport {
            let mut function_stats: Vec<_> = self.function_timings.iter()
                .map(|(name, timings)| {
                    let total_time: Duration = timings.iter().sum();
                    let avg_time = total_time / timings.len() as u32;
                    let max_time = *timings.iter().max().unwrap();
                    
                    FunctionStats {
                        name: name.clone(),
                        call_count: timings.len(),
                        total_time,
                        average_time: avg_time,
                        max_time,
                        percentage_of_total: 0.0, // Calculated below
                    }
                })
                .collect();
            
            // Calculate percentages
            let total_execution_time: Duration = function_stats.iter()
                .map(|stats| stats.total_time)
                .sum();
            
            for stats in &mut function_stats {
                stats.percentage_of_total = 
                    stats.total_time.as_secs_f64() / total_execution_time.as_secs_f64() * 100.0;
            }
            
            // Sort by total time descending
            function_stats.sort_by(|a, b| b.total_time.cmp(&a.total_time));
            
            HotspotReport { function_stats }
        }
    }
}
```

### Common Performance Issues

**Issue**: Memory usage grows unboundedly during processing  
**Solution**: Implement streaming processing with fixed buffer sizes, regular memory monitoring
**Prevention**: Use memory profiling tools, set hard memory limits with validation

**Issue**: Async tasks don't run concurrently as expected  
**Solution**: Check for blocking operations, ensure proper async/await usage, verify semaphore limits
**Prevention**: Use async profiling tools, monitor task execution patterns

**Issue**: File I/O becomes slower as processing continues  
**Solution**: Check for file handle leaks, implement file handle pooling, monitor disk usage
**Prevention**: Use resource management patterns, monitor file descriptor usage

## References

- [Rust Performance Book](https://nnethercote.github.io/perf-book/) - Comprehensive performance guide
- [Tokio Performance Guide](https://tokio.rs/tokio/tutorial/async) - Async performance patterns  
- [Criterion Benchmarking](https://bheisler.github.io/criterion.rs/book/) - Rust benchmarking framework
- [Memory Profiling in Rust](https://github.com/koute/memory-profiler) - Memory analysis tools
- [Architecture Document](architecture.md) - System design context
- [Quality Standards](quality-standards.md) - Performance validation requirements