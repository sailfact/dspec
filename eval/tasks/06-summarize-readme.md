# Task 06 — summarize a README into 5 bullets

| field | value |
|---|---|
| id | `06-summarize-readme` |
| category | mechanical |
| expected gate | high (≥ threshold) |
| expected outcome | accepted / patched |

**Why this task:** faithful extractive summarization of provided text is
low-risk. The source document is embedded so the task is self-contained and does
not depend on any external file.

## Prompt

Paste everything below after `/spec`:

````
Summarize the README below into exactly 5 bullet points. Each bullet one line,
covering the most important facts a new user needs. Output only the 5 bullets.

```markdown
# ledgerd

A tiny append-only ledger daemon for single-host bookkeeping.

## What it is

ledgerd stores immutable financial entries in a local append-only log and serves
them over a small HTTP API. It is built for a single machine and a single writer;
it is not a distributed database and makes no attempt at multi-node consensus.

## Why append-only

Every entry is written once and never mutated or deleted. Corrections are made by
appending a compensating entry that references the original by id. This gives a
complete, tamper-evident audit trail: the full history of every balance can be
reconstructed by replaying the log from the beginning.

## Storage

Entries are serialized as JSON, one object per line, to `ledger.log`. On startup
ledgerd replays the whole file into an in-memory index of balances by account.
The log is the source of truth; the in-memory index is a cache that is rebuilt on
every boot. There is no separate database.

## HTTP API

- `POST /entries` — append a new entry; returns the assigned id.
- `GET  /entries` — list entries, newest first, with optional `?account=` filter.
- `GET  /balances/:account` — current balance for one account.
- `GET  /health` — liveness probe; returns 200 once the log has been replayed.

All money amounts are integer minor units (cents); ledgerd never uses floating
point for money.

## Concurrency

A single writer lock serializes appends so the log is never interleaved. Reads
are lock-free against the in-memory index and may lag the log by microseconds.

## Configuration

Configured entirely by environment variables: `LEDGERD_ADDR` (default
`127.0.0.1:8080`), `LEDGERD_LOG` (default `./ledger.log`), and `LEDGERD_FSYNC`
(`always` or `interval`, default `always`) controlling how aggressively appends
are flushed to disk.

## Durability

With `LEDGERD_FSYNC=always` each append is fsynced before the id is returned, so a
successful response guarantees the entry survived a crash. `interval` batches
fsyncs once per second for higher throughput at the cost of a small window of
possible loss on power failure.

## Non-goals

No authentication (run it behind a trusted proxy), no multi-currency conversion,
no horizontal scaling. These are deliberately out of scope for v1.
```
````

## Grading notes

Accept if there are exactly 5 bullets that faithfully capture the core facts
(append-only/immutable + compensating entries, JSON-lines log replayed into an
in-memory index, the HTTP API, integer-cents money, env-var config /
fsync durability). Patch only for an inaccuracy or a missing critical point, or if
the count isn't 5. Do not patch for choosing different (but accurate) bullets.
