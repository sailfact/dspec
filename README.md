# dspec

**DSpark-inspired speculative draft-then-verify for Claude Code.** A cheap draft model attempts a task, an independent confidence gate scores it, weak drafts are discarded before the target model ever sees them, and the target model verifies-and-patches only the drafts that survive — spending expensive tokens only on the parts that actually diverge.

---

## Read this first: what dspec is *not*

**dspec is a DSpark-*inspired* workflow optimization, not a reimplementation of speculative decoding, and it is not lossless.**

Real [DSpark](https://arxiv.org/) verifies draft tokens against the target model's probability distribution via rejection sampling, which guarantees output identical to target-only decoding. That requires logit access *inside* the inference engine. dspec runs at the API/CLI layer, where no logits exist. Here the target model **judges** a finished draft against a rubric; it does **not** verify it against logits.

Concretely:

- Output is **approximate-to-target**, not distribution-exact.
- The result is **not guaranteed identical** to what the target model would have produced on its own.
- The documentation, this README included, will never claim otherwise.

What you get instead is a *measurable bet*: when the gate is well-calibrated, mechanical work is drafted cheaply and rubber-stamped, while genuinely hard work falls through to normal full-quality execution. The telemetry (`/spec-stats`) tells you whether that bet is paying off. If the gate can't separate good drafts from bad, the whole scheme is noise — and the calibration metric is designed to make that failure obvious rather than silent.

---

## How it works

```
task ──▶ draft model (cheap) ──▶ confidence gate (independent) ──▶ threshold
                                                                      │
                                              confidence ≥ threshold  │  confidence < threshold
                                                       ▼              │            ▼
                                          target model verifies       │      discard draft;
                                          & patches only the delta    │      target does the
                                                                      │      task normally
                                                       └──────────────┴──▶ record outcome + telemetry
```

The pipeline maps onto DSpark's structure as follows:

| DSpark concept | dspec implementation |
|---|---|
| Draft model | `claude -p --model sonnet` subprocess producing a full candidate deliverable |
| Trained per-token confidence head | A second, independent `haiku` call scoring the whole draft 0–100 against a rubric |
| Discard low-confidence drafts pre-verification | Server-side threshold comparison; discarded drafts never reach the target model |
| Target verification (rejection sampling) | Target model verify-and-patch prompt: accept verbatim unless demonstrably wrong |
| Acceptance rate / accepted length | JSONL telemetry: accepted / patched / rejected / discarded + gate calibration |
| Fall back to normal decoding | Target model does the task itself whenever speculation is discarded, errors, or times out |

**Fail-open is a hard rule.** Any error — CLI missing, timeout, non-zero exit, unparseable gate output — short-circuits to `decision: "discard"` with the error recorded, and the task falls through to normal execution. Degradation means losing the *speedup*, never the *answer*. The server never blocks your task.

---

## Requirements

- **Rust** (2021 edition) and `cargo` — to build the MCP server.
- The **`claude` CLI**, installed and authenticated. dspec shells out to `claude -p` subprocesses, so it rides your existing Claude Code auth; there are no separate API keys to configure.
- **Claude Code**, to load the plugin.

---

## Install

dspec ships as source; you build the server once, then load the directory as a local plugin. Prebuilt binaries and build-on-install hooks are out of scope for v1.

**1. Build the server:**

```bash
cd dspec/server
cargo build --release
# binary lands at dspec/server/target/release/dspec-server
```

The plugin manifest points at `${CLAUDE_PLUGIN_ROOT}/server/target/release/dspec-server`, so the release binary must exist before the plugin will start.

**2. Install as a local plugin.**

The local-plugin loading mechanism has changed across Claude Code releases, so don't guess it — check the current help:

```bash
claude plugin --help
claude plugin validate .   # run from the dspec/ plugin root
```

Install `dspec/` as a local plugin per whatever your version documents, then restart Claude Code.

**3. Verify:**

- `/spec` appears in the slash-command list.
- `/mcp` shows the `dspec` server connected with three tools: `draft_task`, `record_outcome`, `spec_stats`.

---

## Usage

### `/spec <task>`

Runs the full speculative pipeline on a task.

```
/spec write a conventional commit message for: renamed telemetry field ts to ts_ms across the dspec server
```

What happens:

1. The command assembles **minimal** context (a hard budget of ~200 lines — context transfer is the hidden cost of speculation).
2. `draft_task` drafts and gates the work server-side.
3. If the decision is **discard**, the target model just does the task normally at full quality and records the outcome as `discarded`.
4. If the decision is **verify**, the target model applies *verify-and-patch discipline*: accept the draft verbatim unless a span is demonstrably wrong, patch only the divergent spans, and never restyle acceptable content.
5. A one-line status is always printed:

   ```
   spec: <decision> conf=<confidence> outcome=<outcome> draft=<draft_ms>ms gate=<gate_ms>ms
   ```

### `/spec-stats`

Reports the telemetry conversationally: total drafts, per-outcome counts, verify-path acceptance rate, mean patch ratio, mean draft/gate latency, and — most importantly — **gate calibration**: the mean gate confidence of drafts that ended accepted/patched versus those that ended rejected. If those two numbers aren't separated by a comfortable margin, the gate isn't predictive and the threshold is meaningless.

---

## The MCP tools

The server exposes three tools; the slash commands orchestrate them, but they're documented here for completeness.

- **`draft_task(task, context?)`** — the core pipeline. Returns JSON: `draft_id`, `decision` (`verify`/`discard`), `confidence`, `reasons`, `draft`, `draft_ms`, `gate_ms`, `error`.
- **`record_outcome(draft_id, outcome, patch_ratio?)`** — appends the final outcome for a speculation.
- **`spec_stats()`** — aggregates the telemetry, joining draft events to outcome events by `draft_id`.

### Outcome vocabulary

Exactly four values, no others:

| Outcome | Meaning |
|---|---|
| `accepted` | Draft used verbatim |
| `patched` | Draft used with targeted edits; include `patch_ratio` (0.0–1.0, fraction of the draft changed) |
| `rejected` | Target regenerated the work from scratch |
| `discarded` | Gate or an error killed the draft; the target did the task normally |

---

## Configuration

All configuration is via environment variables set in the plugin manifest's MCP server entry, each with a default:

| Variable | Default | Meaning |
|---|---|---|
| `DSPEC_DRAFT_MODEL` | `sonnet` | Model alias passed to `claude --model` for drafting |
| `DSPEC_GATE_MODEL` | `haiku` | Model for the gate pass |
| `DSPEC_THRESHOLD` | `60` | Minimum confidence (inclusive) to reach verification |
| `DSPEC_TIMEOUT_SECS` | `120` | Per-subprocess timeout, in seconds |
| `DSPEC_DATA_DIR` | `~/.dspec` | Telemetry location |
| `DSPEC_CLAUDE_BIN` | `claude` | CLI binary; overridden in tests to a mock script |

Unparseable numeric values (e.g. a non-numeric `DSPEC_THRESHOLD`) silently fall back to the default rather than failing.

---

## Telemetry

One JSON object per line, appended to `<DSPEC_DATA_DIR>/events.jsonl` (default `~/.dspec/events.jsonl`). Two event shapes — `draft` and `outcome` — joined by `id`. `spec_stats` reads this file; corrupt lines are skipped rather than causing stats to fail, and telemetry write failures are logged to stderr and swallowed so stats can degrade without ever taking down a task.

---

## Development & testing

```bash
cd server
cargo test -- --test-threads=1
```

The `--test-threads=1` flag matters: a few tests in the `claude_cli` and `server` suites mutate process-level environment variables (`MOCK_MODE`) to drive the mock `claude` fixture, so they must not run concurrently. The subprocess wrapper uses `kill_on_drop(true)`, so a timed-out speculation cannot leak a running `claude` process.

The test suite covers config defaults/overrides, gate JSON extraction (clean, prose-wrapped, garbage, out-of-range), telemetry append/join/calibration math, the CLI wrapper against a mock shell fixture (success / non-zero exit / timeout), the prompts, and the fail-open pipeline paths.

---

## Calibration eval

Gate calibration is the make-or-break metric. The eval below runs a spread of tasks — from mechanical (should gate high, end accepted/patched) to novel design (should gate low, end discarded/rejected) — and checks that `mean_confidence_good` separates cleanly from `mean_confidence_bad`.

> **Fill this in after running the eval.** Run each task through `/spec` (one per fresh session where practical), then `/spec-stats`. **Success criteria:** tasks 1–7 mostly gate ≥60 and end accepted/patched; tasks 8–9 gate low or end rejected; `mean_confidence_good` exceeds `mean_confidence_bad` by **≥10 points**. If calibration fails, iterate the gate rubric wording in `prompts.rs` (a single constant, no structural change) and re-run.

**Eval task set:**

1. Write a `.gitignore` for a Rust cargo workspace. *(mechanical)*
2. Write rustdoc comments for `telemetry.rs`'s public items. *(mechanical)*
3. Convert a 20-line JSON object to TOML. *(mechanical)*
4. Write a conventional commit message for a described diff. *(mechanical)*
5. Write a systemd unit file with restart-on-failure. *(mechanical)*
6. Summarize a 100-line README into 5 bullet points. *(mechanical)*
7. Write a bash one-liner to find the 10 largest files under a directory. *(mechanical)*
8. Design the module structure for a new directory-sync CLI. *(novel — should gate low)*
9. Decide between SQLite and JSONL for telemetry and justify. *(judgment — low or verify-with-patches)*
10. Write property-based tests for `gate::decide`. *(moderately novel)*

**Results:**

| Metric | Value |
|---|---|
| Total drafts | _TBD_ |
| Accepted / Patched / Rejected / Discarded | _TBD_ |
| Verify-path acceptance rate | _TBD_ |
| Mean patch ratio | _TBD_ |
| Mean draft / gate latency (ms) | _TBD_ |
| `mean_confidence_good` | _TBD_ |
| `mean_confidence_bad` | _TBD_ |
| Separation (good − bad) | _TBD_ |
| **Calibration verdict** | _TBD (≥10 pts = calibrated)_ |

---

## License

_TBD — add your license of choice._