# Task 09 — decide between SQLite and JSONL for telemetry

| field | value |
|---|---|
| id | `09-sqlite-vs-jsonl` |
| category | judgment (should gate **low**, or verify-with-patches) |
| expected gate | low / borderline |
| expected outcome | discarded / patched / rejected |

**Why this task:** a judgment call whose "right" answer depends on constraints and
values. A cheap draft may produce a plausible-but-unjustified recommendation, so
the gate should be cautious. Constraints are embedded so the decision is grounded
and the task is self-contained.

## Prompt

Paste everything below after `/spec`:

```
Decide whether this project's telemetry store should stay as an append-only JSONL
file or move to SQLite, and justify the choice. Constraints:
- Writes are append-only events; the only reads are periodic full-file aggregates
  ("stats"), never point queries.
- A write failure must never take down the calling task (fail-open); today a
  failed JSONL append is logged and swallowed.
- Corrupt/partial lines must be survivable — stats already skips unparseable rows.
- Volume is low: at most a few thousand events per user.
- Zero-dependency and human-greppable storage is currently considered a feature.
Give a recommendation with the two or three trade-offs that actually decide it.
```

## Grading notes

There is no single correct answer, but a good one is *grounded in the stated
constraints* (append-only + whole-file aggregates + low volume + fail-open +
grep-ability favor keeping JSONL; SQLite's wins — indexed point queries,
concurrent writers, transactional integrity — are mostly moot here). Discard is a
fine outcome. If verified: accept a well-argued recommendation verbatim; patch
only to fix a claim that contradicts the constraints (e.g. citing point-query
performance as decisive when there are no point queries); reject a recommendation
that ignores the constraints entirely.
