# AGENTS.md

## Project
`tagcat` — a fast CLI for tagging and querying local files. Single Rust binary.

## Stack
- Rust 2021, stable toolchain
- clap (derive) for arg parsing
- rusqlite (bundled) for the local index
- thiserror in library code, anyhow at the binary boundary

## Commands
- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy --all-targets -- -D warnings`
- Format: `cargo fmt`
- Run: `cargo run -- <args>`

## Conventions
- Run `cargo fmt && cargo clippy -- -D warnings` before every commit — CI fails on warnings.
- No `unwrap()`/`expect()` outside tests; propagate with `?`.
- Library functions return `thiserror` enums; only command handlers use `anyhow`.
- Don't add a dependency without justifying it in the PR description.

## Testing
- Every new subcommand gets an integration test in `tests/` using `assert_cmd` + `predicates`.
- Don't shell out manually in tests.

## Don't touch
- `src/migrations/` is append-only — never edit an existing migration, add a new one.