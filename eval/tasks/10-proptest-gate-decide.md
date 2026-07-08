# Task 10 — property-based tests for `gate::decide`

| field | value |
|---|---|
| id | `10-proptest-gate-decide` |
| category | moderately novel |
| expected gate | borderline |
| expected outcome | patched / accepted / rejected |

**Why this task:** writing tests for a tiny, fully-specified function is bounded
enough that a draft can be close, but choosing meaningful properties takes some
judgment — a good mid-scale calibration probe. The function under test is embedded
so the task does not depend on the current repo source.

## Prompt

Paste everything below after `/spec`:

````
Write property-based tests (using the `proptest` crate) for the `decide` function
below. Cover the properties that actually matter: the threshold is inclusive
(confidence == threshold verifies), monotonicity (raising confidence never turns a
Verify into a Discard for a fixed threshold), and the boundary between Verify and
Discard sits exactly at the threshold. Output only the test module.

```rust
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Decision {
    Verify,
    Discard,
}

/// Verify iff confidence is at or above the threshold (inclusive).
pub fn decide(confidence: u8, threshold: u8) -> Decision {
    if confidence >= threshold {
        Decision::Verify
    } else {
        Decision::Discard
    }
}
```
````

## Grading notes

Accept if the tests compile-plausibly and encode the three stated properties with
`proptest!` over `u8` inputs (inclusive boundary at `confidence == threshold`,
`confidence < threshold` ⇒ `Discard`, and monotonicity in `confidence`). Patch a
genuinely wrong assertion (e.g. testing an exclusive boundary, which contradicts
the spec) or a property that doesn't hold. Reject only if the approach is
fundamentally off (e.g. not property-based at all). Different-but-valid property
formulations are not grounds to patch.
