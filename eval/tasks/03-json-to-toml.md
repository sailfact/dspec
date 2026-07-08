# Task 03 — convert JSON to TOML

| field | value |
|---|---|
| id | `03-json-to-toml` |
| category | mechanical |
| expected gate | high (≥ threshold) |
| expected outcome | accepted / patched |

**Why this task:** a deterministic format conversion with one correct answer.

## Prompt

Paste everything below after `/spec`:

````
Convert the following JSON object to equivalent TOML. Preserve all keys, values,
types, and nesting. Output only the TOML.

```json
{
  "name": "dspec-server",
  "version": "0.1.0",
  "threshold": 60,
  "timeout_secs": 120,
  "fail_open": true,
  "models": {
    "draft": "haiku",
    "gate": "haiku"
  },
  "telemetry": {
    "dir": "~/.dspec",
    "file": "events.jsonl",
    "rotate": false
  },
  "outcomes": ["accepted", "patched", "rejected", "discarded"]
}
```
````

## Grading notes

Accept if the TOML round-trips to the same data: scalars keep their types
(`threshold = 60`, `fail_open = true`), `models` and `telemetry` become
`[models]` / `[telemetry]` tables (or inline tables), and `outcomes` is a string
array. Patch only on a real semantic difference (wrong type, dropped/renamed key,
broken nesting). Table-vs-inline-table style is **not** grounds to patch.
