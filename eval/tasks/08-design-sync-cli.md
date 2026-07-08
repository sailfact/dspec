# Task 08 — design the module structure for a directory-sync CLI

| field | value |
|---|---|
| id | `08-design-sync-cli` |
| category | novel (should gate **low**) |
| expected gate | low (< threshold) → discarded |
| expected outcome | discarded / rejected |

**Why this task:** open-ended architecture with many defensible answers and
unstated trade-offs. A cheap draft cannot be confidently rubber-stamped; the gate
*should* score this low and the task should fall through to normal execution. This
is a deliberate negative control for calibration — a well-calibrated gate is as
important here as on the mechanical tasks.

## Prompt

Paste everything below after `/spec`:

```
Design the module/crate structure for a new Rust CLI called `dsync` that does
one-way mirroring of a source directory to a destination. Requirements it must
eventually support: content-hash-based change detection, dry-run mode, include/
exclude globs, resumable transfers after interruption, and a progress display.
Propose the module breakdown, the key types and their responsibilities, and where
the trait boundaries should be. Explain the main design trade-offs.
```

## Grading notes

This is the calibration counter-example: **you expect the gate to discard it.**
If it discards, the target does the design work normally and the outcome is
`discarded` — that is the success case. If the gate lets it through to verify, the
draft will almost always need substantial rework: reject or heavily patch, and
note that the gate over-scored a genuinely open design task (a calibration
concern worth recording).
