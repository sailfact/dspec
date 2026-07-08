# Task 02 — rustdoc comments for a small module

| field | value |
|---|---|
| id | `02-rustdoc-comments` |
| category | mechanical |
| expected gate | high (≥ threshold) |
| expected outcome | accepted / patched |

**Why this task:** documenting given code with no hidden behavior is mechanical.
The source is embedded below so the task is fully reproducible and does not
depend on the current state of the repository.

## Prompt

Paste everything below after `/spec`:

````
Add rustdoc comments (/// doc comments) to every public item in the Rust module
below: the struct, its two public fields, and both public functions. Document
what each does, its parameters, and its return/error behavior. Do not change any
code — output the same module with doc comments added.

```rust
use std::path::Path;

pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
}

pub fn backoff_delay_ms(policy: &RetryPolicy, attempt: u32) -> u64 {
    let shift = attempt.min(16);
    policy.base_delay_ms.saturating_mul(1u64 << shift)
}

pub fn load_policy(path: &Path) -> std::io::Result<RetryPolicy> {
    let text = std::fs::read_to_string(path)?;
    let mut lines = text.lines();
    let max_attempts = lines.next().unwrap_or("3").trim().parse().unwrap_or(3);
    let base_delay_ms = lines.next().unwrap_or("100").trim().parse().unwrap_or(100);
    Ok(RetryPolicy { max_attempts, base_delay_ms })
}
```
````

## Grading notes

Accept if every public item (`RetryPolicy`, `max_attempts`, `base_delay_ms`,
`backoff_delay_ms`, `load_policy`) has a `///` comment and **the code is
unchanged**. Patch only if an item was missed or the code was altered. A correct
draft should note that `load_policy` falls back to defaults on missing/unparseable
lines rather than erroring, and that `backoff_delay_ms` caps the shift and uses
saturating multiplication to avoid overflow.
