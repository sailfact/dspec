# Calibration eval — results

Fill this in after running every task in [`tasks/`](tasks/) through `/spec`, then
`/spec-stats`. Record the environment so the run is reproducible.

- **Date:** 2026-07-10
- **Draft model / gate model:** not surfaced by telemetry (server defaults)
- **`DSPEC_THRESHOLD`:** 60 (plugin manifest default)
- **`DSPEC_DATA_DIR`:** `~/.dspec` (unset; default — log wiped via `/spec-eval clean` immediately before this run)

## Per-task log

| # | id | confidence | decision | outcome | notes |
|---|---|---|---|---|---|
| 01 | `01-gitignore-rust`       | 82 | verify | accepted | all constraints met; `Cargo.lock` un-ignored |
| 02 | `02-rustdoc-comments`     | 88 | verify | accepted | all public items documented, code unchanged |
| 03 | `03-json-to-toml`         | 98 | verify | accepted | round-trips exactly, types preserved |
| 04 | `04-conventional-commit`  | 87 | verify | accepted | `feat(config)`, cites `DSPEC_CLAUDE_BIN` default |
| 05 | `05-systemd-unit`         | 87 | verify | accepted | all directives incl. `Restart=on-failure`/`RestartSec=5` |
| 06 | `06-summarize-readme`     | 87 | verify | accepted | exactly 5 accurate bullets |
| 07 | `07-bash-one-liner`       | 85 | verify | accepted | correct pipeline; unquoted `$1` out of scope here |
| 08 | `08-design-sync-cli`      | 68 | verify | rejected | negative control — gate over-scored open design; substantive gaps (symlinks, mirror-delete semantics, resume validation) |
| 09 | `09-sqlite-vs-jsonl`      | 80 | verify | accepted | constraint-grounded recommendation (keep JSONL) |
| 10 | `10-proptest-gate-decide` | 81 | verify | accepted | all three properties correct |
| 11 | `11-cron-first-monday`    | 89 | verify | accepted | trap avoided — guard workaround (`1-7` dom + `%u` weekday guard) |
| 12 | `12-iso-week-date`        | 97 | verify | accepted | trap avoided — `date +%G-W%V-%u`; verified `2021-01-01`→`2020-W53-5` |
| 13 | `13-safe-bulk-rename`     | 88 | verify | patched (0.15) | sound `-print0`/`read -d ''` traversal; patched dash-unsafe `find "$1"` start point |
| 14 | `14-sql-not-in-null`      | 90 | verify | accepted | trap avoided — LEFT JOIN … `WHERE o.id IS NULL` anti-join |
| 15 | `15-semver-regex`         | — | discard | discarded | gate timed out at 120s (fail-open); task done independently |
| 16 | `16-concurrent-telemetry-design` | 72 | verify | rejected | negative control — gate over-scored; wrong core (claims O_APPEND NFS-safe; PIPE_BUF/4KB conflation) |

## Aggregate (`/spec-stats`)

| Metric | Value |
|---|---|
| Total drafts | 16 |
| Accepted / Patched / Rejected / Discarded | 12 / 1 / 2 / 1 |
| Verify-path acceptance rate | 86.7% (13/15) |
| Mean patch ratio | 0.15 |
| Mean draft / gate latency (ms) | 27444 / 26851 |
| `mean_confidence_good` | 87.6 |
| `mean_confidence_bad` | 70.0 |
| Separation (good − bad) | 17.6 |
| **Calibration verdict** | Calibrated (17.6 ≥ 10 pts). Both rejections were open-design negative controls (08 @68, 16 @72) the gate over-scored just past the 60 threshold — separation holds only because mechanical work scores higher (≥80). Thin bad-side sample (2 rejections). Threshold ~75 would discard both while keeping all accepted drafts. |
