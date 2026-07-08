# Calibration eval — results

Fill this in after running every task in [`tasks/`](tasks/) through `/spec`, then
`/spec-stats`. Record the environment so the run is reproducible.

- **Date:** _TBD_
- **Draft model / gate model:** _TBD_ / _TBD_
- **`DSPEC_THRESHOLD`:** _TBD_
- **`DSPEC_DATA_DIR`:** _TBD (use a fresh/empty log for this run — see README step 2)_

## Per-task log

| # | id | confidence | decision | outcome | notes |
|---|---|---|---|---|---|
| 01 | `01-gitignore-rust`       | _TBD_ | _TBD_ | _TBD_ | |
| 02 | `02-rustdoc-comments`     | _TBD_ | _TBD_ | _TBD_ | |
| 03 | `03-json-to-toml`         | _TBD_ | _TBD_ | _TBD_ | |
| 04 | `04-conventional-commit`  | _TBD_ | _TBD_ | _TBD_ | |
| 05 | `05-systemd-unit`         | _TBD_ | _TBD_ | _TBD_ | |
| 06 | `06-summarize-readme`     | _TBD_ | _TBD_ | _TBD_ | |
| 07 | `07-bash-one-liner`       | _TBD_ | _TBD_ | _TBD_ | |
| 08 | `08-design-sync-cli`      | _TBD_ | _TBD_ | _TBD_ | |
| 09 | `09-sqlite-vs-jsonl`      | _TBD_ | _TBD_ | _TBD_ | |
| 10 | `10-proptest-gate-decide` | _TBD_ | _TBD_ | _TBD_ | |

## Aggregate (`/spec-stats`)

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
