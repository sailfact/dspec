---
description: Report speculative decoding telemetry — acceptance rates and gate calibration
---

# /spec-stats

Call the `spec_stats` MCP tool, then present the results conversationally (no
tables of raw JSON): total drafts, outcome counts, verify-path acceptance rate,
mean patch ratio, mean draft/gate latency.

Then assess gate calibration: compare mean_confidence_good vs
mean_confidence_bad. If they are separated by less than ~10 points (or bad is
higher than good), say plainly that the gate is not predictive at the current
prompt/threshold and suggest either raising DSPEC_THRESHOLD or iterating the
gate rubric. If there are fewer than ~10 verified outcomes, note the sample is
too small to judge.