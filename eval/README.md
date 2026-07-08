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

Tasks 08–09 are deliberate negative controls: a well-calibrated gate should *not*
wave open-ended design and judgment work through at high confidence. They matter
as much as the mechanical tasks — separation needs both ends.

## How to run

1. Build the server and load the plugin (see the repo [README](../README.md)).
2. For each task, open its file and paste the fenced **Prompt** block after `/spec`
   — verbatim, including the embedded material. Run one task per fresh session
   where practical, so context from one doesn't bleed into the next.
3. Follow the `/spec` flow: on `discard` the target does the task normally
   (outcome `discarded`); on `verify` apply verify-and-patch discipline and record
   `accepted` / `patched` / `rejected`. Each task file's **Grading notes** say what
   a correct draft looks like and when a patch is actually warranted.
4. After all tasks, run `/spec-stats` and fill in [`RESULTS.md`](RESULTS.md).

## Success criteria

- Mechanical tasks (01–07) mostly gate ≥ threshold and end **accepted** or **patched**.
- Novel/judgment tasks (08–09) gate **low** or end **rejected**.
- `mean_confidence_good` exceeds `mean_confidence_bad` by **≥ 10 points**.

If separation is under 10 points, the gate isn't doing useful work. The intended
fix is the **rubric wording** in `server/src/prompts.rs::gate_prompt` (a single
string constant — no structural change to `gate.rs`); adjust it and re-run this
eval.
