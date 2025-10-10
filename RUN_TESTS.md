# Test Execution Guide

## Running All Tests

```bash
cargo test
```

## Running Specific Test Suites

### Extraction Tests
```bash
cargo test --test extraction_basic_tests
cargo test --test extraction_basic_extraction_tests
cargo test --test extraction_content_tests
cargo test --test extraction_writer_config_tests
cargo test --test extraction_writer_tests
cargo test --test extraction_organized_file_processing_tests
```

### Parser Tests
```bash
cargo test --test parser_utils_tests
cargo test --test parser_binary_extraction_tests
cargo test --test parser_recovery_backoff_tests
```

### Section Tests
```bash
cargo test --test section_types_tests
cargo test --test section_detector_tests
cargo test --test section_validation_tests
cargo test --test section_validation_artifact_tests
cargo test --test section_validation_content_tests
cargo test --test section_validation_pattern_tests
```

### Security Tests
```bash
cargo test --test security_event_logger_tests
```

## Running Tests with Options

### Show output from passing tests
```bash
cargo test -- --nocapture
```

### Run tests in single thread (for debugging)
```bash
cargo test -- --test-threads=1
```

### Show only failed tests
```bash
cargo test -- --quiet
```

### Run specific test function
```bash
cargo test test_extract_sections_basic
```

## Test Coverage Report

To generate a coverage report (requires cargo-tarpaulin):

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Continuous Integration

These tests are designed to run in CI/CD pipelines. Example GitHub Actions workflow:

```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all-features
```