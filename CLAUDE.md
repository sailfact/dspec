# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

dspec is a Claude Code **plugin**: a slash-command layer (`/spec`, `/spec-stats`) backed by a local Rust MCP server (`server/`). It implements a DSpark-*inspired* (not a true reimplementation of) speculative draft-then-verify workflow: a cheap `claude -p --model sonnet` subprocess drafts a task, an independent `haiku` call gates the draft's confidence 0-100, and only drafts above `DSPEC_THRESHOLD` (default 60) get verified/patched by the target model. See [README.md](README.md) for the full rationale, including the "what dspec is *not*" section — don't describe this as distribution-exact or lossless.

## Build, test, run

```bash
cd server
cargo build --release   # binary must exist at server/target/release/dspec-server for the plugin to load
cargo test -- --test-threads=1   # REQUIRED: -- --test-threads=1, several tests mutate the MOCK_MODE env var
cargo test <test_name> -- --test-threads=1   # single test
```

Plugin-level check: `claude plugin validate .` from the repo root.

There is no separate lint step beyond `cargo build`/`cargo test`; no JS/TS tooling in this repo.

## Architecture

The plugin has two halves that must be read together to understand a change:

- **`commands/*.md`** — slash-command prompts (`/spec`, `/spec-stats`) that run in the *calling* Claude Code session. They instruct that session on: assembling a ~200-line context budget, calling the MCP tools, and enforcing "verify-and-patch discipline" (accept the draft verbatim unless a span is demonstrably wrong — style/phrasing preference is never grounds to change it). This discipline is documented in the prompt, not enforced in code.
- **`server/src/`** — the Rust MCP server (via `rmcp`) exposing three tools consumed by the commands:
  - `draft_task` → `server.rs::run_draft_pipeline` — the actual pipeline: draft call, gate call, threshold decision, telemetry write. This is the one function to read to understand the whole system.
  - `record_outcome` — appends an `Outcome` telemetry event (outcome ∈ `accepted | patched | rejected | discarded`, validated against a fixed 4-value list).
  - `spec_stats` → `telemetry.rs::stats` — joins `Draft` and `Outcome` events by id and computes gate calibration (`mean_confidence_good` vs `mean_confidence_bad`).

Module responsibilities in `server/src/`:
- `config.rs` — env-var config (`DSPEC_*`), all with defaults; unparseable numeric values silently fall back rather than erroring.
- `claude_cli.rs` — subprocess wrapper around `claude -p --model <model> --output-format text`, piping the prompt over stdin. Uses `kill_on_drop(true)` so a timed-out call can't leak a process, and `tokio::time::timeout` for `DSPEC_TIMEOUT_SECS`.
- `gate.rs` — parses the gate model's JSON response out of possible prose wrapping (finds first `{`/last `}`), validates confidence is 0-100, and applies the inclusive `confidence >= threshold` decision.
- `prompts.rs` — the drafter and gate prompt templates as plain string constants (no templating engine). The gate rubric here is what you iterate on if calibration (see below) fails — the README explicitly calls this out as "a single constant, no structural change."
- `telemetry.rs` — JSONL append/read. One `Event` enum with `Draft`/`Outcome` variants (serde-tagged), appended to `<DSPEC_DATA_DIR>/events.jsonl` (default `~/.dspec/events.jsonl`). Corrupt lines are skipped, not fatal.
- `error.rs` — `DraftError` (thiserror), one variant per failure mode surfaced above.

**Fail-open is the core invariant of this codebase.** Any error in the pipeline (CLI missing, non-zero exit, timeout, unparseable gate JSON, out-of-range confidence) collapses to `Decision::Discard` with the error string recorded — never a panic, never a blocked task. When touching `run_draft_pipeline` or `claude_cli.rs`, preserve this: a new failure mode must resolve to discard, not propagate.

**Test doubles**: `server/tests/fixtures/mock_claude.sh` stands in for the real `claude` CLI, driven by the `MOCK_MODE` env var (`ok` / `json` / `fail` / `slow`) read inside individual `#[cfg(test)]` blocks via `DSPEC_CLAUDE_BIN`/`cfg.claude_bin`. Because `MOCK_MODE` is a process-level env var, tests that set it cannot run concurrently — this is why `cargo test` requires `--test-threads=1`.

## Configuration surface

All config is env vars on the MCP server process, set in `.claude-plugin/plugin.json`'s `mcpServers.dspec.env` (currently only `DSPEC_THRESHOLD` is set there) or the shell: `DSPEC_DRAFT_MODEL`, `DSPEC_GATE_MODEL`, `DSPEC_THRESHOLD`, `DSPEC_TIMEOUT_SECS`, `DSPEC_DATA_DIR`, `DSPEC_CLAUDE_BIN`. Defaults live in `config.rs::Config::from_lookup`.

## Gate calibration

`mean_confidence_good` (accepted+patched) should exceed `mean_confidence_bad` (rejected) by ≥10 points, or the gate/threshold isn't doing useful work — see the eval task list and results table in [README.md](README.md). If asked to improve calibration, the intended lever is the rubric text in `prompts.rs::gate_prompt`, not a structural change to `gate.rs`.
