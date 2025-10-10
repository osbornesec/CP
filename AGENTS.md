# Repository Guidelines

## Project Structure & Module Organization
The Rust crate lives in `src/`, split by capability: `extraction/` for section writers and filters, `parser/` for monitoring and binary helpers, `section/` for metadata validation, and `security/` for logging. Shared utilities stay in `utils/`. High-level orchestration sits under `workflow/`. Integration tests reside in `tests/`, benchmarks in `benches/`, reusable samples and fixtures in `samples/`, and contributor tooling (lint, ordering checks) in `scripts/`. Reference material and architecture notes are stored in `docs/`.

## Build, Test, and Development Commands
- `cargo fmt` — format the codebase per `rustfmt.toml`.
- `cargo check` — fast compile-time validation while iterating.
- `cargo test` — run unit and integration tests in `src/` and `tests/`.
- `cargo test --doc` — ensure doctests (e.g., `write_section_with_progress`) compile.
- `cargo clippy --all-targets --all-features` — lint with repository rules; treat warnings as blockers.
- After each edit run `cargo fmt`, `cargo clippy --all-targets --all-features`, and `cargo test` before committing changes.
- `scripts/lint_by_file.sh <path>` — narrow lint runs when triaging CI failures.

## Rust 2024 Style Guide & Naming Conventions
Target the Rust 2024 edition (`cargo +nightly fmt --edition 2024` when previewing). Stick to four-space indentation, trailing commas on multi-line literals, and explicit `let else` + `if let` patterns where the new edition encourages them. Name modules with `snake_case`, types with `PascalCase`, and async helpers with the `_async` suffix. Prefer `dyn Trait` bounds, explicit lifetimes, and `#[non_exhaustive]` where future extension is planned; expose builder or `new` constructors (e.g., `SectionWriteParams::new`). Keep functions focused, add concise comments only for nuanced control flow like progress heuristics. Always rely on `cargo fmt` and `cargo clippy` for formatting and linting—never hand-tune whitespace.

## Testing Guidelines
Unit tests live beside modules; integration scenarios go to `tests/`. Mirror function names in the corresponding test modules (e.g., `mod write_section_with_progress_tests`). Doctests must compile without extra imports—export constructors or helper functions as needed. Add targeted cases when touching parsing, writer behavior, or validation rules, and prefer property-style coverage for boundary conditions.

## Commit & Pull Request Guidelines
Adopt Conventional Commit prefixes seen in history (`feat:`, `fix:`, `chore:`) and keep subject lines under 72 characters. Reference issues using `Refs #123` when applicable. Pull requests should include: summary of changes, validation steps (commands run), screenshots or sample outputs when behavior changes, and notes on backward compatibility. Ensure CI passes `cargo fmt`, `cargo clippy`, and the full test suite before requesting review.

## Security & Configuration Tips
Never commit real customer data; use sanitized fixtures from `samples/`. Scripts may rely on environment variables—document temporary overrides in PR descriptions. Review new dependencies for license compatibility and add them to the audit checklist maintained in `docs/`.
