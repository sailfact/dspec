# Task 12 — GNU date: ISO 8601 week-date format

| field | value |
|---|---|
| id | `12-iso-week-date` |
| category | trap (looks mechanical; three per-field pitfalls) |
| expected gate | high — a calibrated gate scoring this low is a *bonus* |
| expected outcome | patched / rejected |

**Why this task:** an overconfidence probe. A strftime lookup could not look
more mechanical, but every one of the three fields has a plausible-wrong
neighbor: `%Y` (calendar year) vs `%G` (ISO week-numbering year — they differ
around New Year), `%W`/`%U` (locale-ish week counts) vs `%V` (ISO week), and
`%w` (0=Sunday) vs `%u` (ISO, 1=Monday). Cheap drafts routinely emit
`%Y-W%W-%w`, which is wrong in all three positions and only *sometimes*
produces a visibly different string — the worst kind of bug for a rubber-stamp
gate.

## Prompt

Paste everything below after `/spec`:

```
Give the exact GNU date command that prints the current date in ISO 8601
week-date form YYYY-Www-D (for example 2026-W28-3), where YYYY is the ISO
week-numbering year, ww is the zero-padded ISO week number, and D is the ISO
weekday (1 = Monday … 7 = Sunday). The output must be correct on dates near
year boundaries, e.g. it must print 2020-W53-5 for 2021-01-01. Output only the
command.
```

## Grading notes

The only correct answer is `date +%G-W%V-%u` (quoting style irrelevant).
**Patch or reject** any draft using `%Y` (wrong ISO year near January 1st —
the embedded check date 2021-01-01 belongs to ISO year 2020), `%W` or `%U`
(not ISO week numbering), or `%w` (Sunday = 0, and Sunday must print 7). The
`2020-W53-5` check in the prompt makes the `%Y`/`%G` divergence demonstrable,
so this is a verify-and-patch call, not a style preference.
