# dspec calibration eval

Gate calibration is dspec's make-or-break metric: if the gate can't separate good
drafts from bad, the whole draft-then-verify bet is noise. This directory is the
**self-contained** eval that measures it.

Every task under [`tasks/`](tasks/) embeds its full `/spec` prompt **and any input
material it needs** (source to document, JSON to convert, the diff to summarize,
the README to condense, the function to test). Nothing here reaches back into the
repo or the wider filesystem, so a run is reproducible on any checkout and the
result doesn't drift when the code changes. [`manifest.json`](manifest.json) is
the machine-readable index (id, category, expected gate, expected outcome band).

## The task set

The set spans the difficulty range on purpose, so calibration has something to
separate:

| # | Task | Category | Expected gate | Expected outcome |
|---|---|---|---|---|
| 01 | [`.gitignore` for a cargo workspace](tasks/01-gitignore-rust.md) | mechanical | high | accepted / patched |
| 02 | [rustdoc for a small module](tasks/02-rustdoc-comments.md) | mechanical | high | accepted / patched |
| 03 | [JSON → TOML](tasks/03-json-to-toml.md) | mechanical | high | accepted / patched |
| 04 | [conventional commit for a diff](tasks/04-conventional-commit.md) | mechanical | high | accepted / patched |
| 05 | [systemd unit, restart-on-failure](tasks/05-systemd-unit.md) | mechanical | high | accepted / patched |
| 06 | [summarize a README into 5 bullets](tasks/06-summarize-readme.md) | mechanical | high | accepted / patched |
| 07 | [bash one-liner: 10 largest files](tasks/07-bash-one-liner.md) | mechanical | high | accepted / patched |
| 08 | [design a directory-sync CLI](tasks/08-design-sync-cli.md) | novel | **low** | discarded / rejected |
| 09 | [SQLite vs JSONL for telemetry](tasks/09-sqlite-vs-jsonl.md) | judgment | low / borderline | discarded / patched / rejected |
| 10 | [proptests for `gate::decide`](tasks/10-proptest-gate-decide.md) | moderate | borderline | patched / accepted / rejected |
| 11 | [cron: first Monday of the month](tasks/11-cron-first-monday.md) | **trap** | high (low is a bonus) | patched / rejected |
| 12 | [GNU date: ISO week-date format](tasks/12-iso-week-date.md) | **trap** | high (low is a bonus) | patched / rejected |
| 13 | [bulk rename safe for hostile filenames](tasks/13-safe-bulk-rename.md) | **trap** | high (low is a bonus) | patched / rejected |
| 14 | [SQL: customers with no orders, NULL trap](tasks/14-sql-not-in-null.md) | **trap** | high (low is a bonus) | patched / rejected |
| 15 | [regex: validate SemVer 2.0.0](tasks/15-semver-regex.md) | **trap** | high (low is a bonus) | patched / rejected |
| 16 | [crash-consistent concurrent telemetry appends](tasks/16-concurrent-telemetry-design.md) | novel | **low** | discarded / rejected |

Tasks 08–09 and 16 are deliberate negative controls: a well-calibrated gate
should *not* wave open-ended design and judgment work through at high
confidence. They matter as much as the mechanical tasks — separation needs both
ends.

Tasks 11–15 are **trap tasks**, the intense end of the set: each one *looks*
mechanical (a cron line, a strftime string, a rename one-liner, an anti-join, a
validation regex) but contains a well-known correctness pitfall that cheap
drafts fall into — cron's day-of-month/day-of-week OR semantics, `%Y`/`%W`/`%w`
vs `%G`/`%V`/`%u`, shell word splitting, `NOT IN` against a NULL, SemVer's
leading-zero rules. They exist to feed the verify path bad drafts on purpose:
without verify-path rejections, `mean_confidence_bad` is null and calibration is
unmeasurable (a run of only tasks 01–10 can end with a 100% verify-path
acceptance rate and nothing to compare against). Each trap prompt embeds
concrete check inputs so "demonstrably wrong" is mechanical to establish, not a
judgment call — verify against them before accepting. A trap task succeeds
either way: gated low, it shows the gate smells danger under a mechanical
surface; gated high and then patched/rejected, it contributes the bad-outcome
confidence data the separation metric needs. What it must *not* do is end
`accepted` with the trap intact — that's a verification failure, not a gate
failure, and invalidates the data point.

## How to run

1. Build the server and load the plugin (see the repo [README](../README.md)).
2. **Isolate the telemetry log** so `/spec-stats` reflects only this eval.
   `spec_stats` reads the entire `<DSPEC_DATA_DIR>/events.jsonl` and joins *every*
   draft/outcome in it, so a pre-existing log from prior `/spec` use (the default
   is `~/.dspec/events.jsonl`) would pollute the aggregate and let the calibration
   verdict pass or fail independently of these ten runs. Before task 01, point the
   server at a fresh, empty data dir — set `DSPEC_DATA_DIR` to a clean path (e.g.
   `DSPEC_DATA_DIR=$(mktemp -d)/dspec`) in the plugin's MCP server env, or clear /
   move the existing log — and confirm `/spec-stats` reports **0 drafts** before
   you start.
3. For each task, open its file and paste the fenced **Prompt** block after `/spec`
   — verbatim, including the embedded material. Run one task per fresh session
   where practical, so context from one doesn't bleed into the next.
4. Follow the `/spec` flow: on `discard` the target does the task normally
   (outcome `discarded`); on `verify` apply verify-and-patch discipline and record
   `accepted` / `patched` / `rejected`. Each task file's **Grading notes** say what
   a correct draft looks like and when a patch is actually warranted.
5. After all tasks, run `/spec-stats` and fill in [`RESULTS.md`](RESULTS.md).

## Success criteria

- Mechanical tasks (01–07) mostly gate ≥ threshold and end **accepted** or **patched**.
- Novel/judgment tasks (08–09, 16) gate **low** or end **rejected**.
- Trap tasks (11–15) each yield a bad-outcome data point: gate **low**, or gate
  high and end **patched**/**rejected** on the verify path. None ends `accepted`
  with its embedded check inputs failing.
- `mean_confidence_bad` is **non-null** — the run produced actual bad-outcome
  confidences to compare against, not just discards.
- `mean_confidence_good` exceeds `mean_confidence_bad` by **≥ 10 points**.

If separation is under 10 points, the gate isn't doing useful work. The intended
fix is the **rubric wording** in `server/src/prompts.rs::gate_prompt` (a single
string constant — no structural change to `gate.rs`); adjust it and re-run this
eval.
