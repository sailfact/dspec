# Task 11 — cron: first Monday of the month

| field | value |
|---|---|
| id | `11-cron-first-monday` |
| category | trap (looks mechanical; known semantic pitfall) |
| expected gate | high — a calibrated gate scoring this low is a *bonus* |
| expected outcome | rejected / patched |

**Why this task:** an overconfidence probe. The task reads as a one-line cron
lookup, so the gate will likely score it high — but standard 5-field cron **ORs**
the day-of-month and day-of-week fields when both are restricted, so the obvious
draft `5 0 1-7 * 1` fires on *every* Monday **plus** *every* day 1–7, not the
first Monday. Cheap drafts fall into this trap constantly. Whatever the gate
scores, this task produces signal: a low score is good calibration; a high score
followed by a verify-path rejection is exactly the bad-outcome data point
calibration needs.

## Prompt

Paste everything below after `/spec`:

```
Write a standard 5-field cron expression (plain vixie-cron/crontab syntax, no
non-standard extensions like `#`, `L`, or `W`) that runs a job at 00:05 on the
first Monday of every month. If that schedule cannot be expressed exactly in a
single standard 5-field expression, say so explicitly and give the standard
workaround (an expression plus a guard in the command). Output only the answer.
```

## Grading notes

The correct answer states that a single standard 5-field expression **cannot**
express "first Monday of the month": when both day-of-month and day-of-week are
restricted, vixie-cron runs the job when *either* matches. `5 0 1-7 * 1` — the
draft you should expect — fires every Monday **and** every 1st–7th, and must be
recorded **`rejected`**: the deliverable *is* the schedule, so replacing it is
regeneration, not a patch (and only `rejected` outcomes feed
`mean_confidence_bad` — see the eval README). Accept the standard workaround, e.g.
`5 0 * * 1 [ "$(date +\%d)" -le 7 ] && <job>` (guard in the command, dow-only
schedule) or the equivalent `1-7` dom-only schedule with a weekday guard. A
draft that gives `5 0 1-7 * 1` *and* correctly explains the OR caveat with a
guard-based fix is acceptable; the bare expression presented as correct is not.
