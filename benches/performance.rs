//! Performance benchmarks for `CPInfo` Parser
//!
//! This file implements Benchmark-Driven Implementation (BDI) approach
//! with three cycles of performance testing driving the implementation.

use core::time::Duration;
use cpinfo_parser::CpinfoParser;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use memmap2::MmapOptions;
use std::fs::File;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader as TokioBufReader};
use tokio::runtime::Runtime;

// =================================================================================
// Benchmark Configuration & Test Data Setup
// =================================================================================

// Use real cpinfo files for realistic performance testing
const SMALL_TEST_FILE: &str = "samples/6803352_SG1-s02-02_1_7_2025_19_10.info"; // ~8MB
const MEDIUM_TEST_FILE: &str = "samples/fw-02_vs0.tgz.info"; // ~219MB
const LARGE_TEST_FILE: &str = "samples/FW1cpinfo.info"; // ~535MB
const SECTION_DELIMITER: &[u8] = b"//-- SECTION_HEADER --\n";

/// Validate that required test files exist
fn validate_test_files() -> std::io::Result<()> {
    let files = [SMALL_TEST_FILE, MEDIUM_TEST_FILE, LARGE_TEST_FILE];

    for file_path in &files {
        if !Path::new(file_path).exists() {
            eprintln!("Warning: Test file {file_path} not found, skipping related benchmarks");
        }
    }
    Ok(())
}

// =================================================================================
// Cycle 1: I/O Ceiling Benchmarks
// =================================================================================

/// This cycle establishes the theoretical maximum read speed from disk.
/// We will compare memmap2 against a standard `BufReader` to validate our
/// architectural choice. The parser is NOT involved here.
fn benchmark_io_ceiling(c: &mut Criterion) {
    validate_test_files().expect("Failed to validate test files");

    // Test with medium file (~219MB) for reasonable benchmark time
    if !Path::new(MEDIUM_TEST_FILE).exists() {
        eprintln!("Skipping I/O ceiling benchmarks - test file not found");
        return;
    }

    let file_size = std::fs::metadata(MEDIUM_TEST_FILE).unwrap().len();
    let mut group = c.benchmark_group("Cycle 1: I/O Ceiling");
    group.throughput(Throughput::Bytes(file_size));
    group.measurement_time(Duration::from_secs(10));

    // Benchmark 1: Memory-mapped file read
    group.bench_function("memmap2_scan", |b| {
        let file = File::open(MEDIUM_TEST_FILE).unwrap();
        // The Mmap is created outside the loop, as we're benchmarking the scan, not the mapping.
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        // Advise the OS that we will need this data soon - Unix only
        #[cfg(unix)]
        mmap.advise(memmap2::Advice::WillNeed).unwrap();

        b.iter(|| {
            // Iterate over the bytes to ensure the OS actually pages the data in.
            // `black_box` prevents the compiler from optimizing this loop away.
            let count = mmap.iter().filter(|&&byte| byte == b'\n').count();
            black_box(count);
        });
    });

    // Benchmark 2: Asynchronous BufReader read
    group.bench_function("tokio_bufreader_scan", |b| {
        let rt = Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(async {
                let file = tokio::fs::File::open(MEDIUM_TEST_FILE).await.unwrap();
                let mut reader = TokioBufReader::new(file);
                let mut buf = Vec::with_capacity(8192);
                let mut count = 0;
                while reader.read_until(b'\n', &mut buf).await.unwrap() > 0 {
                    count += 1;
                    buf.clear();
                }
                black_box(count);
            });
        });
    });

    group.finish();
}

// =================================================================================
// Cycle 2 & 3: Parser Implementation Benchmarks
// =================================================================================

/// This cycle benchmarks the actual parser implementation.
/// We start with a single-threaded implementation and then add a concurrent one.
/// Tests multiple file sizes to validate performance scaling.
fn benchmark_parser(c: &mut Criterion) {
    validate_test_files().expect("Failed to validate test files");

    let test_files = [
        (SMALL_TEST_FILE, "Small (8MB)"),
        (MEDIUM_TEST_FILE, "Medium (219MB)"),
        (LARGE_TEST_FILE, "Large (535MB)"),
    ];

    for (file_path, file_desc) in &test_files {
        if !Path::new(file_path).exists() {
            eprintln!("Skipping {file_desc} benchmark - file not found: {file_path}");
            continue;
        }

        let file_size = std::fs::metadata(file_path).unwrap().len();
        let mut group = c.benchmark_group(format!("Parser: {file_desc}"));
        group.throughput(Throughput::Bytes(file_size));
        group.sample_size(10);
        group.measurement_time(Duration::from_secs(20));
        group.warm_up_time(Duration::from_secs(3));

        // Single-threaded parser benchmark - Target: > 400 MB/s
        group.bench_function("single_threaded", |b| {
            let rt = Runtime::new().unwrap();
            let parser = CpinfoParser::new();

            b.iter(|| {
                rt.block_on(async {
                    let result = parser.parse_file(file_path).await.unwrap();
                    black_box(result);
                });
            });
        });

        // Concurrent parser benchmark - Target: > 1 GB/s
        group.bench_function("concurrent", |b| {
            let rt = Runtime::new().unwrap();
            let parser = CpinfoParser::new();

            b.iter(|| {
                rt.block_on(async {
                    let result = parser.parse_file_concurrent(file_path).await.unwrap();
                    black_box(result);
                });
            });
        });

        group.finish();
    }
}

criterion_group!(benches, benchmark_io_ceiling, benchmark_parser);
criterion_main!(benches);
