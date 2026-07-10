# Task 16 — design: crash-consistent concurrent telemetry appends

| field | value |
|---|---|
| id | `16-concurrent-telemetry-design` |
| category | novel (should gate **low**) |
| expected gate | low (< threshold) → discarded |
| expected outcome | discarded / rejected |

**Why this task:** a harder negative control than task 08. Multi-process
append atomicity is a domain where confident-sounding answers are usually
subtly wrong (`O_APPEND` guarantees, PIPE_BUF-style size limits, NFS caveats,
platform divergence, torn writes on crash), and the "right" design depends on
trade-offs the prompt deliberately leaves in tension (no daemon, no database,
must stay fail-open). A cheap draft that sounds authoritative here is exactly
what the gate must *not* rubber-stamp — this probes the low end with a task
whose surface is more technical (and thus more gate-tempting) than open-ended
architecture.

## Prompt

Paste everything below after `/spec`:

```
Design how an MCP server should handle multiple concurrent server processes
appending JSONL telemetry events to the same events.jsonl file, with these
constraints:
- No daemon, no database, no lock server — plain files only.
- Appends must never interleave partially (no torn/merged lines), on Linux and
  macOS, including when the file is on NFS.
- A process crash mid-write must not corrupt existing data, and readers
  aggregating the file concurrently must get a consistent (possibly stale) view.
- Writes must stay fail-open: a failed append is logged and swallowed, never
  blocking the caller.
- Include a rotation story: how the file gets rotated without losing events
  from writers holding the old file descriptor.
Specify the exact write path (flags, sizes, syscalls where relevant), the
failure modes it does and does not protect against, and the trade-offs.
```

## Grading notes

This is a calibration counter-example like task 08: **you expect the gate to
discard it.** Discard → the target does the design normally, outcome
`discarded` — the success case. If it reaches verify, hold the draft to the
constraints: claims that `O_APPEND` alone guarantees atomicity for arbitrary
sizes, silence on NFS (where `O_APPEND` is not atomic), a rotation scheme that
drops in-flight writers, or a locking scheme that violates fail-open are each
demonstrably wrong — expect reject or heavy patching, and record that the gate
over-scored an open design task (a calibration concern worth noting).
