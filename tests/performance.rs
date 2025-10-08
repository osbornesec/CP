// Phase 4: Performance and Reliability Tests (Tests 37-45)
// Following Canon TDD principles for performance optimization

use cpinfo_parser::{CpinfoParser, PerformanceConfig};
use std::io::Write;
use tempfile::NamedTempFile;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create default performance config
    fn default_performance_config() -> PerformanceConfig {
        PerformanceConfig {
            max_memory_mb: 100,
            enable_memory_monitoring: false,
            enable_speed_monitoring: false,
            enable_concurrent_processing: false,
            enable_resource_monitoring: false,
            enable_caching: false,
            enable_load_balancing: false,
            enable_profiling: false,
            enable_bottleneck_detection: false,
            enable_scalability_optimization: false,
            enable_failure_recovery: false,
            enable_graceful_degradation: false,
            monitoring_interval_ms: 1000,
            cache_size_mb: 50,
            worker_count: 4,
            profiling_granularity: "medium".to_string(),
            memory_pressure_threshold: 0.8,
            buffer_size: 8192,
            max_concurrent_files: 1,
            // Add missing fields from Phase 5
            max_disk_space_mb: 1024,
            enable_disk_monitoring: false,
            enable_cleanup_on_pressure: false,
            cleanup_threshold_percent: 80.0,
            enable_cpu_monitoring: false,
            cpu_throttle_threshold: 80.0,
            enable_adaptive_processing: false,
            max_processing_threads: 4,
        }
    }

    // Test 37: Memory optimization for large file processing
    #[test]
    fn test_37_memory_optimization_for_large_files() {
        // RED PHASE: This test should FAIL initially

        // Create a large test file (simulate 1GB content)
        let large_test_file = create_large_cpinfo_file(1024 * 1024 * 1024); // 1GB
        let file_path = large_test_file.path();

        // Configure memory constraints
        let mut memory_config = default_performance_config();
        memory_config.max_memory_mb = 100;
        memory_config.enable_memory_monitoring = true;
        memory_config.buffer_size = 8192; // 8KB buffer

        // Process the large file with memory monitoring
        let result = CpinfoParser::parse_with_memory_monitoring(file_path, memory_config);

        match result {
            Ok(memory_stats) => {
                // Verify memory usage stayed under 100MB throughout processing
                assert!(
                    memory_stats.peak_memory_mb <= 100.0,
                    "Peak memory usage {} MB exceeded limit of 100 MB",
                    memory_stats.peak_memory_mb
                );

                // Verify file was processed successfully
                assert!(
                    memory_stats.bytes_processed > 0,
                    "File should have been processed"
                );
                assert!(
                    memory_stats.sections_extracted > 0,
                    "Sections should have been extracted"
                );

                // Verify processing completed within reasonable time
                assert!(
                    memory_stats.processing_duration_ms < 30000,
                    "Processing took {} ms, should be under 30 seconds",
                    memory_stats.processing_duration_ms
                );

                println!("✅ Test 37: Large file processed successfully");
                println!("   Peak memory: {} MB", memory_stats.peak_memory_mb);
                println!("   Bytes processed: {}", memory_stats.bytes_processed);
                println!("   Sections extracted: {}", memory_stats.sections_extracted);
                println!("   Duration: {} ms", memory_stats.processing_duration_ms);
            }
            Err(e) => panic!("Large file processing failed: {}", e),
        }
    }

    // Test 38: Processing speed optimization
    #[test]
    fn test_38_processing_speed_optimization() {
        // RED PHASE: This test should FAIL initially

        // Create a 100MB test file for speed testing
        let speed_test_file = create_large_cpinfo_file(100 * 1024 * 1024); // 100MB
        let file_path = speed_test_file.path();

        // Configure for speed optimization
        let mut speed_config = default_performance_config();
        speed_config.enable_speed_monitoring = true;
        speed_config.buffer_size = 16384; // 16KB buffer for faster processing

        let start_time = std::time::Instant::now();
        let result = CpinfoParser::parse_with_speed_monitoring(file_path, speed_config);
        let elapsed = start_time.elapsed();

        match result {
            Ok(speed_stats) => {
                let file_size_mb = 100.0;
                let processing_speed_mb_per_sec = file_size_mb / elapsed.as_secs_f64();

                // Verify processing speed meets target of 100MB/second
                assert!(
                    processing_speed_mb_per_sec >= 100.0,
                    "Processing speed {} MB/s is below target of 100 MB/s",
                    processing_speed_mb_per_sec
                );

                // Verify sections were extracted efficiently
                assert!(
                    speed_stats.sections_per_second > 10.0,
                    "Section extraction rate {} sections/s is too slow",
                    speed_stats.sections_per_second
                );

                println!("✅ Test 38: Processing speed optimization successful");
                println!(
                    "   Processing speed: {:.2} MB/s",
                    processing_speed_mb_per_sec
                );
                println!(
                    "   Sections per second: {}",
                    speed_stats.sections_per_second
                );
                println!("   Total duration: {:?}", elapsed);
            }
            Err(e) => panic!("Speed optimization test failed: {}", e),
        }
    }

    // Test 39: Concurrent file processing and parallel extraction
    #[test]
    fn test_39_concurrent_file_processing() {
        // RED PHASE: This test should FAIL initially

        // Create multiple test files for concurrent processing
        let test_files: Vec<_> = (0..5)
            .map(|i| {
                create_medium_cpinfo_file(50 * 1024 * 1024, &format!("file_{}", i))
                // 50MB each
            })
            .collect();

        let file_paths: Vec<_> = test_files.iter().map(|f| f.path()).collect();

        // Configure for concurrent processing
        let mut concurrent_config = default_performance_config();
        concurrent_config.max_memory_mb = 200; // Allow more memory for concurrent operations
        concurrent_config.enable_memory_monitoring = true;
        concurrent_config.enable_concurrent_processing = true;
        concurrent_config.max_concurrent_files = 3;

        let start_time = std::time::Instant::now();
        let result = CpinfoParser::parse_concurrent(file_paths, concurrent_config);
        let elapsed = start_time.elapsed();

        match result {
            Ok(concurrent_stats) => {
                // Verify all files were processed successfully
                assert_eq!(
                    concurrent_stats.files_processed, 5,
                    "Expected 5 files to be processed, got {}",
                    concurrent_stats.files_processed
                );

                // Verify concurrent processing was faster than sequential
                let total_size_mb = 5.0 * 50.0; // 250MB total
                let processing_speed = total_size_mb / elapsed.as_secs_f64();

                assert!(
                    processing_speed >= 100.0,
                    "Concurrent processing speed {} MB/s should be at least 100 MB/s",
                    processing_speed
                );

                // Verify memory usage remained reasonable
                assert!(
                    concurrent_stats.peak_memory_mb <= 200.0,
                    "Peak memory {} MB exceeded concurrent limit",
                    concurrent_stats.peak_memory_mb
                );

                println!("✅ Test 39: Concurrent processing successful");
                println!("   Files processed: {}", concurrent_stats.files_processed);
                println!("   Concurrent speed: {:.2} MB/s", processing_speed);
                println!("   Peak memory: {} MB", concurrent_stats.peak_memory_mb);
            }
            Err(e) => panic!("Concurrent processing test failed: {}", e),
        }
    }

    // Helper function to create large test cpinfo files
    fn create_large_cpinfo_file(size_bytes: usize) -> NamedTempFile {
        let mut temp_file = NamedTempFile::with_suffix(".info").unwrap();

        // Write cpinfo header
        writeln!(temp_file, "Check Point Support Information").unwrap();
        writeln!(temp_file, "==============================================").unwrap();
        writeln!(temp_file, "Generated: 2024-01-15 10:30:00").unwrap();
        writeln!(temp_file, "Version: R81.20 - Build 030").unwrap();
        writeln!(temp_file, "==============================================").unwrap();

        let mut bytes_written = 200; // Approximate header size
        let mut section_num = 1;

        // Generate sections until we reach the target size
        while bytes_written < size_bytes {
            let section_name = format!("Large Test Section {}", section_num);
            let section_content = generate_realistic_section_content(8192); // 8KB per section

            writeln!(temp_file, "{}", section_name).unwrap();
            writeln!(temp_file, "==============================================").unwrap();
            writeln!(temp_file, "{}", section_content).unwrap();
            writeln!(temp_file, "==============================================").unwrap();

            bytes_written += section_name.len() + section_content.len() + 100; // Overhead
            section_num += 1;

            // Safety check to prevent infinite loop
            if section_num > 100000 {
                break;
            }
        }

        temp_file.flush().unwrap();
        temp_file
    }

    // Helper function to create medium-sized test files
    fn create_medium_cpinfo_file(size_bytes: usize, identifier: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::with_suffix(".info").unwrap();

        // Write cpinfo header with identifier
        writeln!(
            temp_file,
            "Check Point Support Information - {}",
            identifier
        )
        .unwrap();
        writeln!(temp_file, "==============================================").unwrap();
        writeln!(temp_file, "Generated: 2024-01-15 10:30:00").unwrap();
        writeln!(temp_file, "Version: R81.20 - Build 030").unwrap();
        writeln!(temp_file, "File ID: {}", identifier).unwrap();
        writeln!(temp_file, "==============================================").unwrap();

        let mut bytes_written = 250; // Approximate header size
        let mut section_num = 1;

        // Generate sections for medium file
        while bytes_written < size_bytes {
            let section_name = format!("{} Section {}", identifier, section_num);
            let section_content = generate_realistic_section_content(4096); // 4KB per section

            writeln!(temp_file, "{}", section_name).unwrap();
            writeln!(temp_file, "==============================================").unwrap();
            writeln!(temp_file, "{}", section_content).unwrap();
            writeln!(temp_file, "==============================================").unwrap();

            bytes_written += section_name.len() + section_content.len() + 100;
            section_num += 1;

            if section_num > 50000 {
                break;
            }
        }

        temp_file.flush().unwrap();
        temp_file
    }

    // Test 40: Resource usage monitoring and performance metrics collection
    #[test]
    fn test_40_resource_usage_monitoring_and_metrics_collection() {
        // RED PHASE: This test should FAIL initially

        // Create a test file for resource monitoring
        let monitoring_test_file = create_medium_cpinfo_file(75 * 1024 * 1024, "monitoring");
        let file_path = monitoring_test_file.path();

        // Configure for comprehensive resource monitoring
        let mut monitoring_config = default_performance_config();
        monitoring_config.enable_memory_monitoring = true;
        monitoring_config.enable_speed_monitoring = true;
        monitoring_config.enable_resource_monitoring = true;
        monitoring_config.monitoring_interval_ms = 100; // Check every 100ms

        let result = CpinfoParser::parse_with_resource_monitoring(file_path, monitoring_config);

        match result {
            Ok(resource_stats) => {
                // Verify comprehensive resource tracking
                assert!(
                    resource_stats.cpu_samples.len() > 5,
                    "Should have multiple CPU samples, got {}",
                    resource_stats.cpu_samples.len()
                );

                assert!(
                    resource_stats.memory_samples.len() > 5,
                    "Should have multiple memory samples, got {}",
                    resource_stats.memory_samples.len()
                );

                assert!(
                    resource_stats.io_samples.len() > 5,
                    "Should have multiple I/O samples, got {}",
                    resource_stats.io_samples.len()
                );

                // Verify resource usage stayed within reasonable bounds
                let max_cpu_usage = resource_stats
                    .cpu_samples
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();

                assert!(
                    *max_cpu_usage <= 95.0,
                    "CPU usage {} % exceeded reasonable limit",
                    max_cpu_usage
                );

                // Verify memory tracking accuracy
                let max_memory_mb = resource_stats
                    .memory_samples
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();

                assert!(
                    *max_memory_mb <= 100.0,
                    "Memory usage {} MB exceeded configured limit",
                    max_memory_mb
                );

                // Verify I/O monitoring
                assert!(
                    resource_stats.total_bytes_read > 0,
                    "Should have tracked bytes read"
                );

                assert!(
                    resource_stats.total_bytes_written > 0,
                    "Should have tracked bytes written"
                );

                // Verify real-time metrics collection (< 1s intervals)
                let avg_interval_ms = resource_stats.total_monitoring_duration_ms as f64
                    / resource_stats.cpu_samples.len() as f64;

                assert!(
                    avg_interval_ms <= 1000.0,
                    "Average monitoring interval {} ms should be under 1 second",
                    avg_interval_ms
                );

                println!("✅ Test 40: Resource monitoring successful");
                println!("   CPU samples: {}", resource_stats.cpu_samples.len());
                println!("   Max CPU usage: {:.1}%", max_cpu_usage);
                println!("   Max memory: {:.1} MB", max_memory_mb);
                println!("   Bytes read: {}", resource_stats.total_bytes_read);
                println!("   Bytes written: {}", resource_stats.total_bytes_written);
            }
            Err(e) => panic!("Resource monitoring test failed: {}", e),
        }
    }

    // Test 41: Caching system for repeated section parsing and metadata
    #[test]
    fn test_41_caching_system_for_repeated_section_parsing() {
        // RED PHASE: This test should FAIL initially

        // Create test files with repeated content patterns
        let cache_test_file1 = create_cache_test_file("cache_test_1");
        let cache_test_file2 = create_cache_test_file("cache_test_2");
        let file_path1 = cache_test_file1.path();
        let file_path2 = cache_test_file2.path();

        // Configure with caching enabled
        let mut cache_config = default_performance_config();
        cache_config.max_memory_mb = 150;
        cache_config.enable_memory_monitoring = true;
        cache_config.enable_speed_monitoring = true;
        cache_config.enable_caching = true;
        cache_config.cache_size_mb = 50;

        // First parse - should populate cache
        let start_time1 = std::time::Instant::now();
        let result1 = CpinfoParser::parse_with_caching(file_path1, cache_config.clone());
        let duration1 = start_time1.elapsed();

        // Second parse of similar content - should benefit from cache
        let start_time2 = std::time::Instant::now();
        let result2 = CpinfoParser::parse_with_caching(file_path2, cache_config.clone());
        let duration2 = start_time2.elapsed();

        match (result1, result2) {
            (Ok(cache_stats1), Ok(cache_stats2)) => {
                // Verify caching improved performance on second file using microseconds for better precision
                let duration1_micros = duration1.as_micros() as f64;
                let duration2_micros = duration2.as_micros() as f64;

                // Guard against division by zero and very small durations
                let min_duration_micros = 100.0; // 0.1ms minimum
                let safe_duration2 = duration2_micros.max(min_duration_micros);

                let speedup_ratio = duration1_micros / safe_duration2;

                println!(
                    "Duration 1: {:.2}μs, Duration 2: {:.2}μs, Speedup: {:.2}x",
                    duration1_micros, duration2_micros, speedup_ratio
                );

                println!(
                    "Cache Stats 1: hits={}, misses={}, hit_rate={:.1}%",
                    cache_stats1.cache_hits,
                    cache_stats1.cache_misses,
                    cache_stats1.cache_hit_rate * 100.0
                );
                println!(
                    "Cache Stats 2: hits={}, misses={}, hit_rate={:.1}%",
                    cache_stats2.cache_hits,
                    cache_stats2.cache_misses,
                    cache_stats2.cache_hit_rate * 100.0
                );

                assert!(speedup_ratio >= 1.2,
                    "Second parse should be at least 20% faster due to caching, got {:.2}x speedup (Duration 1: {:.2}μs, Duration 2: {:.2}μs)",
                    speedup_ratio, duration1_micros, duration2_micros);

                // Verify cache hit rate is significant (>50% for second parse since first parse populates cache)
                assert!(
                    cache_stats2.cache_hit_rate >= 0.5,
                    "Cache hit rate {:.1}% should be over 50%",
                    cache_stats2.cache_hit_rate * 100.0
                );

                // Verify cache is being used effectively
                assert!(
                    cache_stats2.cache_hits > 0,
                    "Should have cache hits on second parse"
                );

                // Since cache statistics are global, the second call will have more total hits than misses
                // The logic should be that cache hits increased between first and second call
                assert!(
                    cache_stats2.cache_hits > cache_stats1.cache_hits,
                    "Second parse should have accumulated more cache hits"
                );

                // Verify memory usage is reasonable with caching
                assert!(
                    cache_stats2.peak_memory_mb <= 150.0,
                    "Memory usage with cache {} MB exceeded limit",
                    cache_stats2.peak_memory_mb
                );

                // Verify cache efficiency
                let cache_efficiency = cache_stats2.cache_hits as f64
                    / (cache_stats2.cache_hits + cache_stats2.cache_misses) as f64;

                assert!(
                    cache_efficiency >= 0.8,
                    "Cache efficiency {:.1}% should be over 80%",
                    cache_efficiency * 100.0
                );

                println!("✅ Test 41: Caching system successful");
                println!("   First parse: {:?}", duration1);
                println!("   Second parse: {:?}", duration2);
                println!("   Speedup: {:.2}x", speedup_ratio);
                println!(
                    "   Cache hit rate: {:.1}%",
                    cache_stats2.cache_hit_rate * 100.0
                );
                println!("   Cache efficiency: {:.1}%", cache_efficiency * 100.0);
            }
            (Err(e), _) => panic!("First caching test failed: {}", e),
            (_, Err(e)) => panic!("Second caching test failed: {}", e),
        }
    }

    // Test 42: Load balancing for batch processing operations across workers
    #[test]
    fn test_42_load_balancing_for_batch_processing() {
        // RED PHASE: This test should FAIL initially

        // Create files of varying sizes for load balancing test
        let small_file = create_medium_cpinfo_file(20 * 1024 * 1024, "small"); // 20MB
        let medium_file = create_medium_cpinfo_file(50 * 1024 * 1024, "medium"); // 50MB
        let large_file = create_medium_cpinfo_file(80 * 1024 * 1024, "large"); // 80MB
        let huge_file = create_medium_cpinfo_file(100 * 1024 * 1024, "huge"); // 100MB

        let file_paths = vec![
            small_file.path(),
            medium_file.path(),
            large_file.path(),
            huge_file.path(),
        ];

        // Configure load balancing with multiple workers
        let mut load_balance_config = default_performance_config();
        load_balance_config.max_memory_mb = 300;
        load_balance_config.enable_memory_monitoring = true;
        load_balance_config.enable_speed_monitoring = true;
        load_balance_config.enable_concurrent_processing = true;
        load_balance_config.enable_load_balancing = true;
        load_balance_config.max_concurrent_files = 4;
        load_balance_config.worker_count = 4;

        let start_time = std::time::Instant::now();
        let result = CpinfoParser::parse_with_load_balancing(file_paths, load_balance_config);
        let elapsed = start_time.elapsed();

        match result {
            Ok(load_balance_stats) => {
                // Verify all files were processed
                assert_eq!(
                    load_balance_stats.files_processed, 4,
                    "Should process all 4 files, got {}",
                    load_balance_stats.files_processed
                );

                // Verify load was distributed across workers
                assert!(
                    load_balance_stats.worker_utilization.len() >= 2,
                    "Should use at least 2 workers for load balancing"
                );

                // Check load distribution efficiency (no worker should handle >60% of total load)
                let total_load: f64 = load_balance_stats.worker_utilization.values().sum();
                let max_worker_load = load_balance_stats
                    .worker_utilization
                    .values()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();

                let max_worker_percentage = max_worker_load / total_load;
                assert!(
                    max_worker_percentage <= 0.6,
                    "Maximum worker load {:.1}% should not exceed 60% for good load balancing",
                    max_worker_percentage * 100.0
                );

                // Verify processing speed with load balancing
                let total_size_mb = 20.0 + 50.0 + 80.0 + 100.0; // 250MB total
                let processing_speed = total_size_mb / elapsed.as_secs_f64();

                assert!(
                    processing_speed >= 150.0,
                    "Load balanced processing speed {} MB/s should be at least 150 MB/s",
                    processing_speed
                );

                // Verify memory usage was managed across workers
                assert!(
                    load_balance_stats.peak_memory_mb <= 300.0,
                    "Peak memory {} MB should stay within limit",
                    load_balance_stats.peak_memory_mb
                );

                // Debug output to understand worker load distribution
                println!("Worker utilization breakdown:");
                for (worker_id, load) in &load_balance_stats.worker_utilization {
                    println!(
                        "   Worker {}: {:.2} MB ({:.1}%)",
                        worker_id,
                        load,
                        (load / total_load) * 100.0
                    );
                }
                println!("   Total load: {:.2} MB", total_load);
                println!(
                    "   Max worker percentage: {:.1}%",
                    max_worker_percentage * 100.0
                );
                println!(
                    "   Algorithm efficiency: {:.1}%",
                    load_balance_stats.load_balance_efficiency * 100.0
                );

                // Calculate coefficient of variation for debugging
                let mean_load = total_load / load_balance_stats.worker_utilization.len() as f64;
                let variance: f64 = load_balance_stats
                    .worker_utilization
                    .values()
                    .map(|load| (load - mean_load).powi(2))
                    .sum::<f64>()
                    / load_balance_stats.worker_utilization.len() as f64;
                let cv = variance.sqrt() / mean_load;
                println!("   Mean load: {:.2} MB, CV: {:.3}", mean_load, cv);

                // Use the actual load balancing efficiency from the algorithm
                // With highly varied file sizes (20MB, 50MB, 80MB, 100MB), perfect distribution is impossible
                // For heterogeneous workloads, 30% efficiency is reasonable
                assert!(load_balance_stats.load_balance_efficiency >= 0.3,
                    "Load balancing efficiency {:.1}% should be at least 30% for heterogeneous file sizes",
                    load_balance_stats.load_balance_efficiency * 100.0);

                println!("✅ Test 42: Load balancing successful");
                println!("   Processing speed: {:.2} MB/s", processing_speed);
                println!(
                    "   Workers used: {}",
                    load_balance_stats.worker_utilization.len()
                );
                println!("   Max worker load: {:.1}%", max_worker_percentage * 100.0);
                println!(
                    "   Load balance efficiency: {:.1}%",
                    load_balance_stats.load_balance_efficiency * 100.0
                );
            }
            Err(e) => panic!("Load balancing test failed: {}", e),
        }
    }

    // Test 43: Performance profiling and bottleneck identification tools
    #[test]
    fn test_43_performance_profiling_and_bottleneck_identification() {
        // RED PHASE: This test should FAIL initially

        // Create a complex file for profiling
        let profiling_test_file = create_large_cpinfo_file(150 * 1024 * 1024); // 150MB
        let file_path = profiling_test_file.path();

        // Configure for detailed profiling
        let mut profiling_config = default_performance_config();
        profiling_config.max_memory_mb = 200;
        profiling_config.enable_memory_monitoring = true;
        profiling_config.enable_speed_monitoring = true;
        profiling_config.enable_profiling = true;
        profiling_config.enable_bottleneck_detection = true;
        profiling_config.profiling_granularity = "detailed".to_string();

        let result = CpinfoParser::parse_with_profiling(file_path, profiling_config);

        match result {
            Ok(profiling_stats) => {
                // Verify profiling data was collected
                assert!(
                    !profiling_stats.operation_timings.is_empty(),
                    "Should have operation timing data"
                );

                assert!(
                    !profiling_stats.bottlenecks_identified.is_empty(),
                    "Should identify potential bottlenecks"
                );

                // Verify key operations were profiled
                let required_operations =
                    vec!["file_reading", "section_parsing", "content_extraction"];
                for operation in required_operations {
                    assert!(
                        profiling_stats.operation_timings.contains_key(operation),
                        "Should profile {} operation",
                        operation
                    );
                }

                // Verify bottleneck identification
                let bottlenecks = &profiling_stats.bottlenecks_identified;
                assert!(
                    bottlenecks.len() <= 3,
                    "Should identify top 3 bottlenecks, got {}",
                    bottlenecks.len()
                );

                // Verify bottleneck analysis includes actionable information
                for bottleneck in bottlenecks {
                    assert!(
                        !bottleneck.operation_name.is_empty(),
                        "Bottleneck should have operation name"
                    );

                    assert!(
                        bottleneck.time_percentage > 0.0,
                        "Bottleneck should have time percentage"
                    );

                    assert!(
                        !bottleneck.suggested_optimization.is_empty(),
                        "Bottleneck should have optimization suggestion"
                    );
                }

                // Verify performance insights
                assert!(
                    profiling_stats.performance_insights.len() >= 3,
                    "Should provide at least 3 performance insights"
                );

                // Verify profiling overhead is minimal (<5% of total time)
                let profiling_overhead_percent = profiling_stats.profiling_overhead_ms as f64
                    / profiling_stats.total_processing_time_ms as f64
                    * 100.0;

                assert!(
                    profiling_overhead_percent <= 5.0,
                    "Profiling overhead {:.2}% should be under 5%",
                    profiling_overhead_percent
                );

                println!("✅ Test 43: Performance profiling successful");
                println!(
                    "   Operations profiled: {}",
                    profiling_stats.operation_timings.len()
                );
                println!("   Bottlenecks identified: {}", bottlenecks.len());
                println!(
                    "   Performance insights: {}",
                    profiling_stats.performance_insights.len()
                );
                println!("   Profiling overhead: {:.2}%", profiling_overhead_percent);

                // Display top bottleneck
                if let Some(top_bottleneck) = bottlenecks.first() {
                    println!(
                        "   Top bottleneck: {} ({:.1}% of time)",
                        top_bottleneck.operation_name, top_bottleneck.time_percentage
                    );
                }
            }
            Err(e) => panic!("Performance profiling test failed: {}", e),
        }
    }

    // Test 44: Scalability testing with enterprise-scale file loads (10+ files)
    #[test]
    fn test_44_scalability_testing_with_enterprise_scale_loads() {
        // RED PHASE: This test should FAIL initially

        // Create 12 files of varying sizes for enterprise scale testing
        let enterprise_files: Vec<_> = (1..=12)
            .map(|i| {
                let size_mb = match i {
                    1..=4 => 30,  // 4 small files: 30MB each
                    5..=8 => 60,  // 4 medium files: 60MB each
                    9..=12 => 90, // 4 large files: 90MB each
                    _ => 30,
                };
                create_medium_cpinfo_file(size_mb * 1024 * 1024, &format!("enterprise_{}", i))
            })
            .collect();

        let file_paths: Vec<_> = enterprise_files.iter().map(|f| f.path()).collect();

        // Configure for enterprise-scale processing
        let mut enterprise_config = default_performance_config();
        enterprise_config.max_memory_mb = 500;
        enterprise_config.enable_memory_monitoring = true;
        enterprise_config.enable_speed_monitoring = true;
        enterprise_config.enable_concurrent_processing = true;
        enterprise_config.enable_load_balancing = true;
        enterprise_config.enable_scalability_optimization = true;
        enterprise_config.max_concurrent_files = 6;
        enterprise_config.worker_count = 6;
        enterprise_config.buffer_size = 16384; // Larger buffer for enterprise scale

        let worker_count = enterprise_config.worker_count;
        let start_time = std::time::Instant::now();
        let result = CpinfoParser::parse_enterprise_scale(file_paths, enterprise_config);
        let elapsed = start_time.elapsed();

        match result {
            Ok(enterprise_stats) => {
                // Verify all files were processed successfully
                assert_eq!(
                    enterprise_stats.files_processed, 12,
                    "Should process all 12 enterprise files, got {}",
                    enterprise_stats.files_processed
                );

                // Verify linear performance scaling
                let total_size_mb = (4.0 * 30.0) + (4.0 * 60.0) + (4.0 * 90.0); // 720MB total
                let processing_speed = total_size_mb / elapsed.as_secs_f64();

                assert!(
                    processing_speed >= 200.0,
                    "Enterprise processing speed {} MB/s should be at least 200 MB/s",
                    processing_speed
                );

                // Verify memory usage scaled appropriately
                assert!(
                    enterprise_stats.peak_memory_mb <= 500.0,
                    "Peak memory {} MB should stay within enterprise limit",
                    enterprise_stats.peak_memory_mb
                );

                // Verify fault tolerance - all files completed successfully
                assert_eq!(
                    enterprise_stats.failed_files, 0,
                    "No files should fail in enterprise processing"
                );

                // Verify resource utilization efficiency
                let cpu_efficiency = enterprise_stats.average_cpu_utilization;
                assert!(
                    cpu_efficiency >= 60.0 && cpu_efficiency <= 90.0,
                    "CPU utilization {:.1}% should be between 60-90% for efficiency",
                    cpu_efficiency
                );

                // Verify worker coordination
                assert!(
                    enterprise_stats.worker_coordination_overhead_ms < 1000,
                    "Worker coordination overhead {} ms should be under 1 second",
                    enterprise_stats.worker_coordination_overhead_ms
                );

                // Verify scalability metrics
                let throughput_per_worker = processing_speed / worker_count as f64;
                assert!(
                    throughput_per_worker >= 30.0,
                    "Throughput per worker {:.2} MB/s should be at least 30 MB/s",
                    throughput_per_worker
                );

                println!("✅ Test 44: Enterprise scalability successful");
                println!("   Files processed: {}", enterprise_stats.files_processed);
                println!("   Total size: {:.0} MB", total_size_mb);
                println!("   Processing speed: {:.2} MB/s", processing_speed);
                println!("   CPU efficiency: {:.1}%", cpu_efficiency);
                println!("   Throughput/worker: {:.2} MB/s", throughput_per_worker);
            }
            Err(e) => panic!("Enterprise scalability test failed: {}", e),
        }
    }

    // Test 45: Reliability testing with resource constraints and failure recovery
    #[test]
    fn test_45_reliability_testing_with_resource_constraints() {
        // RED PHASE: This test should FAIL initially

        // Create test files for reliability testing
        let reliability_files: Vec<_> = (1..=8)
            .map(|i| {
                create_medium_cpinfo_file(40 * 1024 * 1024, &format!("reliability_{}", i))
                // 40MB each
            })
            .collect();

        let file_paths: Vec<_> = reliability_files.iter().map(|f| f.path()).collect();

        // Configure with constrained resources for reliability testing
        let mut reliability_config = default_performance_config();
        reliability_config.max_memory_mb = 120; // Constrained memory
        reliability_config.enable_memory_monitoring = true;
        reliability_config.enable_speed_monitoring = true;
        reliability_config.enable_concurrent_processing = true;
        reliability_config.enable_failure_recovery = true;
        reliability_config.enable_graceful_degradation = true;
        reliability_config.max_concurrent_files = 3; // Limited concurrency
        reliability_config.worker_count = 3;
        reliability_config.buffer_size = 4096; // Smaller buffer
        reliability_config.memory_pressure_threshold = 0.85; // 85% memory pressure threshold

        let start_time = std::time::Instant::now();
        let result = CpinfoParser::parse_with_reliability_testing(file_paths, reliability_config);
        let elapsed = start_time.elapsed();

        match result {
            Ok(reliability_stats) => {
                // Verify reliability under constraints
                assert!(
                    reliability_stats.files_processed >= 6,
                    "Should process at least 6/8 files under constraints, got {}",
                    reliability_stats.files_processed
                );

                // Verify graceful degradation occurred when needed
                if reliability_stats.degradation_events > 0 {
                    assert!(
                        reliability_stats.files_processed >= 4,
                        "Even with degradation, should process at least 4 files"
                    );
                }

                // Verify memory constraints were respected
                assert!(
                    reliability_stats.peak_memory_mb <= 120.0,
                    "Peak memory {} MB should stay within constraint",
                    reliability_stats.peak_memory_mb
                );

                // Verify failure recovery mechanisms
                if reliability_stats.recovery_events > 0 {
                    assert!(
                        reliability_stats.successful_recoveries
                            >= reliability_stats.recovery_events / 2,
                        "Should recover from at least 50% of failures"
                    );
                }

                // Verify system maintained stability (99%+ uptime)
                let uptime_percentage = (1.0
                    - (reliability_stats.downtime_ms as f64 / elapsed.as_millis() as f64))
                    * 100.0;
                assert!(
                    uptime_percentage >= 99.0,
                    "System uptime {:.2}% should be at least 99%",
                    uptime_percentage
                );

                // Verify error handling
                assert!(
                    reliability_stats.handled_errors >= reliability_stats.unhandled_errors,
                    "Should handle more errors gracefully than crash"
                );

                // Verify partial processing capability
                if reliability_stats.files_processed < 8 {
                    assert!(
                        reliability_stats.partial_processing_enabled,
                        "Should enable partial processing when some files fail"
                    );
                }

                // Verify performance under constraints
                let total_processed_mb = reliability_stats.files_processed as f64 * 40.0;
                let constrained_speed = total_processed_mb / elapsed.as_secs_f64();

                assert!(
                    constrained_speed >= 50.0,
                    "Processing speed under constraints {} MB/s should be at least 50 MB/s",
                    constrained_speed
                );

                println!("✅ Test 45: Reliability testing successful");
                println!(
                    "   Files processed: {}/{}",
                    reliability_stats.files_processed, 8
                );
                println!("   Peak memory: {:.1} MB", reliability_stats.peak_memory_mb);
                println!("   System uptime: {:.2}%", uptime_percentage);
                println!("   Recovery events: {}", reliability_stats.recovery_events);
                println!(
                    "   Successful recoveries: {}",
                    reliability_stats.successful_recoveries
                );
                println!(
                    "   Degradation events: {}",
                    reliability_stats.degradation_events
                );
                println!("   Constrained speed: {:.2} MB/s", constrained_speed);
            }
            Err(e) => panic!("Reliability testing failed: {}", e),
        }
    }

    // Helper function to create cache test files with repeated patterns
    fn create_cache_test_file(identifier: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::with_suffix(".info").unwrap();

        // Write header
        writeln!(
            temp_file,
            "Check Point Support Information - {}",
            identifier
        )
        .unwrap();
        writeln!(temp_file, "==============================================").unwrap();

        // Create repeated sections that can benefit from caching
        let repeated_sections = vec![
            ("General Information", generate_general_info_content()),
            ("Network Configuration", generate_network_config_content()),
            ("Security Policy", generate_security_policy_content()),
            ("System Status", generate_system_status_content()),
        ];

        // Repeat each section type multiple times
        for _ in 0..5 {
            for (section_name, content) in &repeated_sections {
                writeln!(temp_file, "{}", section_name).unwrap();
                writeln!(temp_file, "==============================================").unwrap();
                writeln!(temp_file, "{}", content).unwrap();
                writeln!(temp_file, "==============================================").unwrap();
            }
        }

        temp_file.flush().unwrap();
        temp_file
    }

    fn generate_general_info_content() -> String {
        r#"Product: Check Point Security Gateway
Version: R81.20
Build: 914000250
Kernel: R81.20
Hostname: cp-gateway-01
Architecture: x86_64
CPU Cores: 8
Total Memory: 16 GB"#
            .to_string()
    }

    fn generate_network_config_content() -> String {
        r#"Interface eth0:
  IP Address: 192.168.1.100
  Subnet Mask: 255.255.255.0
  State: Up
  MTU: 1500

Interface eth1:
  IP Address: 10.0.0.1
  Subnet Mask: 255.0.0.0  
  State: Up
  MTU: 1500

Default Gateway: 192.168.1.1
DNS Servers: 8.8.8.8, 8.8.4.4"#
            .to_string()
    }

    fn generate_security_policy_content() -> String {
        r#"Total Rules: 1500
Allow Rules: 1200
Drop Rules: 250
Reject Rules: 50

Last Policy Install: 2024-01-15 09:15:30
Policy Package: Corporate_Security_Policy
Administrator: admin
Install Status: Success"#
            .to_string()
    }

    fn generate_system_status_content() -> String {
        r#"CPU Usage: 45.2%
Memory Usage: 67.8%
Disk Usage: 32.1%
Connections: 15000
Throughput: 890.5 Mbps
Uptime: 45 days, 12 hours
Load Average: 2.3, 2.1, 2.0"#
            .to_string()
    }

    // Generate realistic cpinfo section content
    fn generate_realistic_section_content(target_size: usize) -> String {
        let mut content = String::new();
        let base_lines = vec![
            "System Information:",
            "  Product: Check Point Security Gateway",
            "  Version: R81.20",
            "  Build: 914000250",
            "  Kernel: R81.20",
            "",
            "Network Configuration:",
            "  Interface eth0: 192.168.1.100/24",
            "  Interface eth1: 10.0.0.1/8",
            "  Gateway: 192.168.1.1",
            "",
            "Security Policy:",
            "  Rules: 1500",
            "  Objects: 2500",
            "  Last Policy Install: 2024-01-15 09:15:30",
            "",
            "Performance Metrics:",
            "  CPU Usage: 45.2%",
            "  Memory Usage: 67.8%",
            "  Connections: 15000",
            "  Throughput: 890.5 Mbps",
            "",
        ];

        let mut current_size = 0;
        let mut line_index = 0;

        while current_size < target_size {
            let line = &base_lines[line_index % base_lines.len()];
            content.push_str(line);
            content.push('\n');
            current_size += line.len() + 1;
            line_index += 1;
        }

        content
    }
}
