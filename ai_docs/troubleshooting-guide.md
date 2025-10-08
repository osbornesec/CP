# Troubleshooting Guide - cpinfo-parser Refactoring

## Overview
- **Purpose**: Comprehensive troubleshooting for PRP refactoring implementation
- **Use Cases**: Problem resolution during all refactoring phases  
- **Version**: All phases (Foundation through Final Polish)
- **Last Updated**: 2025-08-08

## Quick Navigation

### 🚨 Phase-Specific Issues
- **[Phase 1: Foundation Issues](#phase-1-foundation-issues)** - Error handling, module structure
- **[Phase 2: Code Quality Issues](#phase-2-code-quality-issues)** - Clippy lints, patterns
- **[Phase 3: Performance Issues](#phase-3-performance-issues)** - Memory, throughput
- **[Phase 4: Polish Issues](#phase-4-polish-issues)** - Documentation, deployment

### 🔧 Common Problems
- **[Compilation Errors](#compilation-errors)** - Build failures and fixes
- **[Runtime Errors](#runtime-errors)** - Execution problems
- **[Performance Problems](#performance-problems)** - Speed and memory issues
- **[Quality Gate Failures](#quality-gate-failures)** - Lint and test failures

## Phase 1: Foundation Issues

### Error Handling Migration Problems

#### Issue: Existing unwrap() calls causing compilation errors
```bash
error: use of `unwrap()` is not allowed
 --> src/parser.rs:45:32
   |
45 |     let file = File::open(path).unwrap();
   |                                ^^^^^^^^ help: try this: `?`
```

**Root Cause**: Clippy deny rule for `unwrap_used`  
**Solution**:
```rust
// BEFORE: Panic-prone code
let file = File::open(path).unwrap();

// AFTER: Proper error propagation
let file = File::open(path)
    .map_err(|e| ProcessingError::FileAccess {
        path: path.to_owned(),
        source: e,
    })?;
```

**Prevention**: Run `cargo clippy -- -W clippy::unwrap_used` during development

#### Issue: Complex error chains causing type issues
```bash
error[E0277]: the trait `From<std::io::Error>` is not implemented for `ProcessingError`
```

**Root Cause**: Missing error conversion implementations  
**Solution**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum ProcessingError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),  // Automatic conversion
    
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
}
```

**Prevention**: Use `thiserror` derive macros for automatic conversions

### Module Organization Problems  

#### Issue: Circular dependency errors
```bash
error[E0369]: circular modules are not allowed
```

**Root Cause**: Improper module extraction order  
**Solution**: Use dependency graph approach from [Refactoring Patterns](refactoring-patterns.md#module-boundary-refactoring)

1. Extract leaf modules first (no dependencies)
2. Work upward through dependency layers
3. Use trait abstractions to break cycles

**Commands**:
```bash
# Analyze dependencies before refactoring
cargo tree --duplicates
cargo modules generate graph --file deps.svg
```

## Phase 2: Code Quality Issues

### Clippy Lint Failures

#### Issue: Mass clippy warnings after enabling strict lints
```bash
warning: this function has too many arguments (8/7)
warning: large types being passed by value  
warning: missing documentation for public items
... (50+ warnings)
```

**Root Cause**: Applying all clippy rules simultaneously  
**Solution**: Incremental lint adoption strategy

1. **Start with critical lints**:
```toml
[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
missing_docs = "deny"
```

2. **Add pedantic rules gradually**:
```bash
# Enable one category at a time
cargo clippy -- -W clippy::complexity
cargo clippy -- -W clippy::perf  
cargo clippy -- -W clippy::style
```

3. **Use targeted fixes**:
```bash
# Fix specific lint types
cargo clippy --fix -- -W clippy::needless_pass_by_value
```

#### Issue: Documentation coverage gaps
```bash
error: missing documentation for public items
 --> src/lib.rs:12:1
   |
12 | pub struct Parser {
   |            ^^^^^^ missing docs for `Parser`
```

**Solution**: Use documentation template from [Quality Standards](quality-standards.md#code-documentation-standards)

**Quick Fix Script**:
```bash
#!/bin/bash
# Generate documentation stubs
find src -name "*.rs" -exec grep -l "pub struct\|pub enum\|pub fn" {} \; | \
while read file; do
    echo "Adding documentation stubs to $file"
    sed -i 's/pub struct//// TODO: Document this struct\npub struct/g' "$file"
done
```

### Pattern Implementation Problems

#### Issue: Lifetime parameter confusion
```bash
error[E0621]: explicit lifetime required in the type of `content`
```

**Root Cause**: Complex lifetime relationships  
**Solution**: Follow lifetime patterns from [Rust Patterns](rust-patterns.md#borrowed-data-pattern-for-zero-copy-processing)

```rust
// Problematic: Complex lifetime mixing
pub struct Parser<'a> {
    content: &'a str,
    sections: Vec<Section<'a>>,
}

// Solution: Clear lifetime separation
pub struct Parser<'content> {
    content: &'content str,
    position: usize,
}

impl<'content> Parser<'content> {
    pub fn sections(&self) -> impl Iterator<Item = Section<'content>> + '_ {
        // Clear lifetime relationships
    }
}
```

## Phase 3: Performance Issues

### Memory Usage Problems

#### Issue: Memory consumption exceeds limits during processing
```bash
thread 'main' panicked at 'memory allocation of 2147483648 bytes failed'
```

**Root Cause**: Unbounded memory growth in streaming processing  
**Solution**: Apply memory management patterns from [Performance Patterns](performance-patterns.md#memory-efficient-section-processing)

**Debug Commands**:
```bash
# Monitor memory usage during development
cargo run --bin cpinfo-parser -- extract large_file.txt &
PID=$!
while kill -0 $PID 2>/dev/null; do
    ps -o pid,rss,vsz,comm -p $PID
    sleep 2
done
```

**Memory-Efficient Fix**:
```rust
// BEFORE: Loading entire sections
let sections: Vec<Section> = parser.parse_all_sections(content)?;

// AFTER: Streaming processing
let section_iter = parser.sections();
for section in section_iter {
    process_section_streaming(section).await?;
    // Section dropped immediately after processing
}
```

#### Issue: Performance benchmarks failing
```bash
test bench_large_file_processing ... FAILED
Expected: >400MB/s, Got: 50MB/s
```

**Root Cause**: Inefficient I/O patterns or excessive allocations  
**Diagnostic Steps**:

1. **Profile with criterion**:
```bash
cargo bench --bench performance -- --profile-time=30
```

2. **Check for allocation hotspots**:
```bash
# Use allocation profiler (requires instrumentation)
RUSTFLAGS="-C instrument-coverage" cargo run --release
```

3. **Apply performance patterns**:
- Use zero-copy string processing
- Implement buffer reuse strategies  
- Apply streaming I/O with proper buffer sizing

### Async Context Issues

#### Issue: Async functions not running concurrently
```bash
# Expected: 4 files processed in parallel
# Actual: Files processed sequentially
```

**Root Cause**: Blocking operations in async context  
**Solution**: Follow async patterns from [Rust Patterns](rust-patterns.md#async-pattern-with-structured-concurrency)

**Debug Async Issues**:
```rust
// Add timing diagnostics
#[tracing::instrument]
async fn process_file_async(path: &Path) -> Result<()> {
    let start = Instant::now();
    
    // Check for blocking operations
    let result = tokio::task::spawn_blocking(move || {
        // CPU-intensive work here
        process_file_sync(&path)
    }).await??;
    
    tracing::info!("File processed", 
        path = %path.display(),
        duration_ms = start.elapsed().as_millis()
    );
    
    Ok(result)
}
```

## Phase 4: Polish Issues

### Documentation Coverage Problems

#### Issue: rustdoc generation failures
```bash
error: unresolved link to `Section`
 --> src/parser.rs:15:25
   |
15 | /// Parses content into [`Section`] objects
   |                         ^^^^^^^^^ no item named `Section` in scope
```

**Root Cause**: Broken intra-doc links after refactoring  
**Solution**:

1. **Fix link syntax**:
```rust
// WRONG: Broken link
/// Parses content into [`Section`] objects

// CORRECT: Proper intra-doc link
/// Parses content into [`crate::parser::Section`] objects
```

2. **Validate doc links**:
```bash
# Check for broken links during build
RUSTDOCFLAGS="-D rustdoc::broken-intra-doc-links" cargo doc --all-features
```

### Enterprise Feature Integration

#### Issue: Configuration loading failures in enterprise mode
```bash
Error: Configuration file not found: /etc/cpinfo-parser/config.toml
```

**Root Cause**: Missing configuration file handling  
**Solution**: Implement graceful configuration fallback from [CLI Patterns](cli-patterns.md#configuration-management)

```rust
// Graceful configuration loading
impl ConfigManager {
    pub fn load_with_fallback() -> Result<AppConfig, ConfigError> {
        // Try enterprise config first
        if let Ok(config) = Self::load_enterprise_config() {
            return Ok(config);
        }
        
        // Fall back to user config
        if let Ok(config) = Self::load_user_config() {
            return Ok(config);
        }
        
        // Use defaults
        Ok(AppConfig::default())
    }
}
```

## Compilation Errors

### Dependency Resolution Issues

#### Issue: Version conflicts in Cargo.toml
```bash
error: failed to select a version for the requirement `tokio = "^1.40"`
```

**Solution**:
1. **Check for conflicts**:
```bash
cargo tree --duplicates --invert tokio
```

2. **Pin specific versions**:
```toml
[dependencies]
tokio = { version = "=1.40.0", features = ["rt-multi-thread", "fs"] }
```

3. **Update dependencies systematically**:
```bash
cargo update --package tokio
```

### Feature Flag Issues

#### Issue: Features not compiling correctly
```bash
error[E0432]: unresolved import `enterprise_features`
```

**Solution**: Proper feature flag configuration
```toml
[features]
default = ["standard"]
standard = []
enterprise = ["dep:enterprise_features", "dep:audit_logger"]

[dependencies]
enterprise_features = { version = "1.0", optional = true }
```

## Runtime Errors

### File Processing Errors

#### Issue: Large file processing failures
```bash
Error: Memory limit exceeded: 1500MB > 500MB
```

**Solution**: Implement streaming processing
```rust
// Configure appropriate limits based on file size
let memory_limit = match file_size {
    0..=100_000_000 => 500 * 1024 * 1024,      // 500MB for small files
    _ => 2 * 1024 * 1024 * 1024,               // 2GB for large files
};
```

### VSX Context Issues

#### Issue: VSX sections not properly separated
```bash
Warning: VSX context detection failed, sections may be mixed
```

**Solution**: Apply VSX patterns from [Check Point Context](checkpoint-context.md#vsx-context-management-pattern)

## Performance Problems

### Throughput Issues

#### Issue: Processing speed below targets
Expected: >400MB/s, Actual: 50MB/s

**Diagnostic Steps**:

1. **Profile bottlenecks**:
```bash
cargo build --release
perf record --call-graph dwarf ./target/release/cpinfo-parser extract large_file.txt
perf report
```

2. **Check I/O patterns**:
```rust
// Monitor I/O efficiency
use std::time::Instant;

let start = Instant::now();
let bytes_processed = process_file_streaming(&path).await?;
let duration = start.elapsed();

let throughput_mbps = (bytes_processed as f64) / duration.as_secs_f64() / 1024.0 / 1024.0;
println!("Throughput: {:.2} MB/s", throughput_mbps);
```

3. **Apply optimizations**:
- Increase buffer sizes for large files
- Use parallel processing for batch operations
- Implement zero-copy string processing

## Quality Gate Failures

### Test Failures

#### Issue: Integration tests failing after refactoring
```bash
test integration_test_large_file ... FAILED
thread 'integration_test_large_file' panicked at 'assertion failed'
```

**Solution**:
1. **Update test expectations**:
```rust
// Account for refactored behavior
#[test]
fn test_refactored_behavior() {
    let result = process_file("test_file.txt").unwrap();
    
    // Update assertions for new structure
    assert_eq!(result.sections.len(), expected_count);
    assert!(result.processing_time < Duration::from_secs(10));
}
```

2. **Add regression tests**:
```rust
#[test]
fn test_backward_compatibility() {
    // Ensure old behavior still works where expected
    let old_format_result = process_file_legacy_mode("old_file.txt").unwrap();
    let new_format_result = process_file("old_file.txt").unwrap();
    
    assert_eq!(old_format_result.sections, new_format_result.sections);
}
```

### Benchmark Failures

#### Issue: Performance regression detected
```bash
Performance regression: large_file_processing 15% slower than baseline
```

**Solution**:
1. **Identify regression source**:
```bash
git bisect start
git bisect bad HEAD
git bisect good <last_good_commit>
# Git will help identify the problematic commit
```

2. **Profile the difference**:
```bash
# Compare performance profiles
cargo bench --bench performance > current_results.txt
git checkout <baseline_commit>
cargo bench --bench performance > baseline_results.txt
diff -u baseline_results.txt current_results.txt
```

## Emergency Procedures

### Rollback Strategy

#### Issue: Critical failure requiring immediate rollback

**Steps**:
1. **Identify last known good state**:
```bash
git log --oneline -10
git show <commit_hash> --stat
```

2. **Create emergency branch**:
```bash
git checkout -b emergency-rollback-$(date +%Y%m%d)
git cherry-pick <critical_fixes>
```

3. **Quick validation**:
```bash
cargo check --all-targets
cargo test --lib
cargo clippy -- -D warnings
```

### Production Hotfix

#### Issue: Critical bug found during final polish phase

**Procedure**:
1. **Isolate the issue**:
```bash
# Create minimal reproduction case
cargo test failing_test_name -- --nocapture
```

2. **Apply targeted fix**:
```rust
// Use feature flags for emergency fixes
#[cfg(feature = "emergency-fix")]
pub fn emergency_workaround() -> Result<()> {
    // Minimal code change to resolve critical issue
}
```

3. **Fast-track validation**:
```bash
# Skip lengthy tests, focus on critical path
cargo test --lib critical_tests
cargo bench --bench critical_performance -- --quick
```

## Support Resources

### Debug Commands Reference
```bash
# Comprehensive debugging toolkit

# Memory analysis
valgrind --tool=massif ./target/release/cpinfo-parser extract large_file.txt
heaptrack ./target/release/cpinfo-parser extract large_file.txt

# Performance profiling  
perf record --call-graph dwarf ./target/release/cpinfo-parser extract large_file.txt
cargo flamegraph --bin cpinfo-parser -- extract large_file.txt

# Dependency analysis
cargo tree --duplicates
cargo audit

# Code quality
cargo clippy -- -D warnings -A clippy::module_inception
cargo fmt --check
cargo doc --document-private-items --open

# Testing
cargo test --all-features --lib -- --nocapture
cargo bench --bench performance
```

### Useful Environment Variables
```bash
# Debugging
export RUST_BACKTRACE=1
export RUST_LOG=debug
export RUST_LOG_STYLE=always

# Performance
export CARGO_BUILD_JOBS=8
export RUSTFLAGS="-C target-cpu=native"

# Memory debugging
export MALLOC_CHECK_=2
export MALLOC_PERTURB_=1
```

### Communication Templates

#### Bug Report Template
```markdown
## Issue Description
Brief description of the problem

## Phase Context
- [ ] Phase 1: Foundation
- [ ] Phase 2: Code Quality
- [ ] Phase 3: Performance  
- [ ] Phase 4: Final Polish

## Reproduction Steps
1. Command executed: `cargo ...`
2. Expected behavior: ...
3. Actual behavior: ...

## Environment
- Rust version: `rustc --version`
- OS: `uname -a`
- Commit hash: `git rev-parse HEAD`

## Error Output
```
[Paste error output here]
```

## Attempted Solutions
- [ ] Checked relevant context guide
- [ ] Applied suggested troubleshooting steps
- [ ] Verified no conflicting changes
```

#### Performance Issue Template  
```markdown
## Performance Problem
Description of performance issue

## Measurements
- Expected: >400MB/s
- Actual: XXX MB/s
- Test file size: XXX MB
- Hardware: CPU/RAM specifications

## Profiling Data
[Attach flamegraph or perf output]

## Context
- Phase: X
- Related changes: [list recent changes]
- Suspected bottleneck: [hypothesis]
```

## Conclusion

This troubleshooting guide provides systematic approaches to resolve issues across all PRP refactoring phases. When encountering problems:

1. **Identify the phase context** - Different phases have different common issues
2. **Use diagnostic commands** - Gather data before attempting fixes
3. **Apply targeted solutions** - Reference the appropriate context guides
4. **Validate fixes** - Run relevant quality gates after changes
5. **Document lessons learned** - Update context guides with new discoveries

For complex issues not covered here, escalate to the project maintainers with detailed bug reports using the provided templates.

**Quick Reference**: Always start with `cargo check`, then `cargo clippy`, then `cargo test` for a basic health check after any changes.