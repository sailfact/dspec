---
description: Speculative draft-then-verify — a cheap model drafts, a gate scores, you verify only the delta
argument-hint: <task>
---

# /spec — speculative execution

Task: $ARGUMENTS

Follow this procedure exactly.

## 1. Assemble minimal context
Gather ONLY what the drafter strictly needs: relevant file excerpts, constraints,
naming conventions. Hard budget: ~200 lines of context. Context transfer is the
hidden cost of speculation — when in doubt, send less. Do not read files that
are not clearly required.

## 2. Draft
Call the `draft_task` MCP tool with the task and that minimal context.

## 3. If decision is "discard"
Perform the task yourself, normally, at full quality. Then call `record_outcome`
with the returned draft_id and outcome "discarded". Tell the user in one line
that speculation was skipped and why (gate reasons or error).

## 4. If decision is "verify" — VERIFY-AND-PATCH DISCIPLINE
You are a verifier, not an author. The economics of this entire workflow depend
on you NOT rewriting acceptable work.
- Read the draft against the task. Accept it verbatim unless a span is
  demonstrably wrong: factually incorrect, broken code, missing an explicit
  requirement, or violating a stated constraint.
- Style, phrasing, or structure you would merely have done differently is NOT
  grounds to change anything.
- If patching: change ONLY the divergent spans, preserve everything else
  byte-for-byte, and estimate patch_ratio (fraction of the draft you changed,
  0.0–1.0).
- If the draft's approach is fundamentally wrong, reject it and produce your
  own solution from scratch.
- Deliver the final result to the user, briefly noting what (if anything) you
  changed and why.
- Call `record_outcome` with outcome "accepted", "patched" (with patch_ratio),
  or "rejected".

## 5. Summary line
End with exactly one status line:
`spec: <decision> conf=<confidence> outcome=<outcome> draft=<draft_ms>ms gate=<gate_ms>ms`