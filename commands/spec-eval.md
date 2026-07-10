---
description: Run the calibration eval — all tasks, a selection (numbers/ranges), or `clean` to wipe telemetry
argument-hint: [task numbers | ranges | clean]
---

# /spec-eval — run the calibration eval

Arguments: $ARGUMENTS

Batch-runs tasks from `${CLAUDE_PLUGIN_ROOT}/eval/tasks/` through the `/spec`
pipeline and reports calibration. Follow this procedure exactly.

## 1. Parse the arguments

Split `$ARGUMENTS` on whitespace and interpret:

- **empty** → run every task listed in `${CLAUDE_PLUGIN_ROOT}/eval/manifest.json`, in manifest order.
- **`clean`** (the only argument) → go to step 5; do not run any tasks.
- **integers** (e.g. `1`, `3 2`) → run exactly those tasks, **in the order given** (`3 2` runs 03 then 02).
- **ranges** `a-b` (e.g. `3-7`) → expand inclusive (`3 4 5 6 7`); if `a > b`, expand descending. Ranges and integers can mix (`1 3-5 12`).
- Drop duplicate task numbers after the first occurrence.
- Any other token, or a number with no matching task in the manifest → print the valid task ids from the manifest and **stop without running anything**.

## 2. Resolve the tasks

Read `${CLAUDE_PLUGIN_ROOT}/eval/manifest.json`. Task number N maps to the entry
whose id starts with the two-digit prefix (1 → `01-…`). Each entry's `file` is
relative to `${CLAUDE_PLUGIN_ROOT}/eval/`.

Before a **full** run, remind the user (one line) that a pre-existing telemetry
log pollutes the calibration verdict and that `/spec-eval clean` wipes it —
then proceed; do not block on it.

## 3. Run each task

For each selected task, in order:

1. Read the task file. Extract the fenced **Prompt** block — verbatim,
   including all embedded material — and note its **Grading notes**.
2. Call `draft_task` with the prompt as the task and **no additional context**:
   every eval prompt is deliberately self-contained; adding repo context breaks
   reproducibility. Do not let drafts or verdicts from earlier tasks in the
   batch influence this one.
3. Follow the `/spec` flow on the result:
   - **discard** → do the task yourself normally, then `record_outcome` with
     `discarded`.
   - **verify** → apply verify-and-patch discipline (accept verbatim unless a
     span is demonstrably wrong; never restyle), **using the task's Grading
     notes as the rubric** — check any embedded inputs (test cases, check
     dates, sample data) mechanically before accepting. Per the eval README:
     if a trap task's core artifact is wrong, record `rejected`, not
     `patched`; `patched` (with patch_ratio) is for a correct core with a
     peripheral flaw.
4. Print the task's one-line status:
   `spec-eval <NN-id>: <decision> conf=<confidence> outcome=<outcome> draft=<draft_ms>ms gate=<gate_ms>ms`

Task failures fail open, matching the server: if one task errors, record what
happened, skip to the next, and say so in the report.

## 4. Report

After the last task, call `spec_stats` and present:

- A per-task table: `# | id | confidence | decision | outcome | notes` (the
  same columns as `eval/RESULTS.md` — on a full run, offer to fill that file in).
- The aggregate: outcome counts, verify-path acceptance rate, mean patch ratio,
  mean latencies.
- The calibration verdict: `mean_confidence_good` vs `mean_confidence_bad`,
  flagging separation < 10 points, a null `mean_confidence_bad` (no verify-path
  rejections — calibration unmeasurable), or a sample too small to judge.

If the log contained drafts from before this batch, say the aggregate covers
them too.

## 5. `clean` — wipe telemetry

Resolve the data dir: `$DSPEC_DATA_DIR` if set (check the shell and the plugin
manifest's `mcpServers.dspec.env`), otherwise `~/.dspec`. Then:

1. Show what exists: `events.jsonl` (with its line count) and `live/*.log`.
2. Delete `events.jsonl` and the `live/` logs from that directory. Do not
   remove the directory itself or anything else in it. No server restart is
   needed — the next append recreates the files.
3. Confirm in one line what was removed, e.g.
   `spec-eval clean: removed events.jsonl (42 events) and 2 live logs from ~/.dspec`.
   If there was nothing to remove, say so instead.
