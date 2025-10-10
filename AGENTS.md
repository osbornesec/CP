# AGENTS.md

## Setup
- Install the stable Rust toolchain: `rustup toolchain install stable` (crate targets edition 2021; see `Cargo.toml`).
- Ensure components: `rustup component add rustfmt clippy`; optional helpers: `cargo install rust-script` (for `scripts/quick_profiling.rs`), `jq` + `python3` (for lint scripts).
- Dependencies are locked in `Cargo.lock`; prefetch when offline: `cargo fetch`.
- Sanitized cpinfo fixtures live under `samples/`; keep them read-only and avoid uploading real customer data.

## Run
- CLI entrypoint: `cargo run --bin cpinfo-parser -- --help` to inspect flags (async tokio runtime).
- Typical end-to-end parse: `cargo run --release -- samples/fw-02_vs0.tgz.info tmp/output` (creates `sections/`, `commands/`, `files/` subdirs).
- Section extraction only: `cargo run -- --extract-only samples/FW1cpinfo.info tmp/sections`.
- Parse pre-extracted section: `cargo run -- --section-file tmp/sections/CP_Status.txt`.
- Feature gates: defaults enable `security` + `progress`; override with `cargo run --no-default-features --features security` when deterministic logs are needed.
- Quick perf profiling (optional): `rust-script scripts/quick_profiling.rs` uses `samples/fw-02_vs0.tgz.info`.

## Test
- Full suite: `cargo test` (unit + integration under `tests/unit/`); see [RUN_TESTS.md](RUN_TESTS.md) for targeted invocations.
- Doctests: `cargo test --doc`.
- Show all output: `cargo test -- --nocapture`; force serial runs when debugging concurrency: `cargo test -- --test-threads=1`.
- Property-based and async tests use large fixtures—prefer subset runs (e.g., `cargo test extraction_writer_tests`) during iteration.
- Optional coverage (requires `cargo tarpaulin`): `cargo tarpaulin --out Html`.

## Lint, Typecheck & Format
- Formatting: `cargo fmt` (or `cargo fmt --check` in CI scenarios).
- Fast compilation gate: `cargo check --all-targets --all-features`.
- Strict linting (pedantic + restriction lints, `print!`/`unwrap!` denied): `cargo clippy --all-targets --all-features`.
- Ordering audit helper: `./check_ordering.sh` (needs `jq`; reports `arbitrary_source_item_ordering` findings).
- Deep lint triage: `./lint_by_file.sh` regenerates `lint_report/` with categorized Clippy output (removes existing `lint_report/` before rebuilding).
- API docs sanity (respects no-std restrictions): `cargo doc --no-deps --all-features`.

## CI Parity
- No workflow files are committed; treat `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`, and `cargo doc --no-deps --all-features` as required gates before PRs.
- Automation scripts in `lint_reports_latest/` and `TEST_*.md` snapshots reflect the expected state after running the gates above.

## Repo Map
- `src/` — core crate modules (`extraction/`, `parser/`, `section/`, `security/`, `workflow/`, etc.) exposed via `lib.rs`; binary entry in `main.rs`.
- `tests/unit/` — integration-style tests grouped by capability (extraction, parser, security, workflow, sanitization, progress).
- `samples/` — sanitized cpinfo fixtures for manual runs and regression tests.
- `scripts/` — developer tooling (Rust-script profiler).
- `benches/`, `examples/` — placeholders reserved for Criterion benchmarks and runnable samples.
- `docs/` — generated reports (e.g., `implicit_return_configuration_conflict_report.md`).
- Root helpers: `RUN_TESTS.md`, `TEST_*SUMMARY.md`, `check_ordering.sh`, `lint_by_file.sh`.

## Contributing
- Use Conventional Commits (`feat:`, `fix:`, `chore:`); keep subject ≤72 chars and reference issues with `Refs #123` when relevant.
- Always run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`, and (when docs change) `cargo doc --no-deps --all-features` before pushing.
- Update or link to existing docs (e.g., `RUN_TESTS.md`) instead of duplicating instructions; keep new helpers in `docs/` or `scripts/` per project layout.
- Large fixture additions must be sanitized and stored in `samples/` with paths documented in PR descriptions.

## Env & Secrets
- No mandatory `.env` file

## Gotchas
- Clippy restrictions (`print!`, `dbg!`, `unwrap!`, `todo!`, `exit`, arithmetic lints, etc.) will turn into errors—mirror the existing patterns (`anyhow`, `Result`, explicit error handling).
- Progress reporter is rate-limited; tests that inspect logs expect deterministic order—avoid background threads writing to stdout/stderr.
- `lint_by_file.sh` deletes and recreates `lint_report/`; ensure nothing important is stored there before running.
- Large fixtures live outside Git LFS; avoid copying them outside the repo to keep diffs manageable.
- Tokio runtime powers the CLI; long-running operations should remain async-safe.

## Monorepo Notes
- Single-crate repository (`cpinfo-parser`); no Cargo workspace or Node/Python packages. Commands above operate at repo root.
