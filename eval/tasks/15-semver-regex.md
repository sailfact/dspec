# Task 15 — regex: validate SemVer 2.0.0

| field | value |
|---|---|
| id | `15-semver-regex` |
| category | trap (looks mechanical; edge-case-dense spec) |
| expected gate | high — a calibrated gate scoring this low is a *bonus* |
| expected outcome | patched / rejected |

**Why this task:** an overconfidence probe. "Write a regex for semver" sounds
like a lookup, and drafts confidently produce something like
`^\d+\.\d+\.\d+(-[\w.]+)?(\+[\w.]+)?$` — which accepts leading zeros (`01.0.0`),
numeric pre-release identifiers with leading zeros (`1.0.0-01`), and empty
identifiers (`1.0.0-alpha..1`), all of which SemVer 2.0.0 explicitly forbids.
The prompt embeds accept/reject cases so a wrong draft is demonstrably wrong,
not stylistically different.

## Prompt

Paste everything below after `/spec`:

```
Write a single PCRE regular expression, anchored at both ends, that matches
exactly the strings that are valid Semantic Versioning 2.0.0 versions. It must
enforce: no leading zeros in the major/minor/patch numbers; pre-release
identifiers that are non-empty, dot-separated, alphanumeric-or-hyphen, with no
leading zeros in purely numeric identifiers; and build metadata identifiers that
are non-empty, dot-separated, alphanumeric-or-hyphen.

It must ACCEPT: 1.0.0, 0.1.0, 1.0.0-alpha, 1.0.0-alpha.1, 1.0.0-0.3.7,
1.0.0-x-y-z.--, 1.0.0-alpha+001, 1.0.0+20130313144700, 1.0.0-beta+exp.sha.5114f85
It must REJECT: 01.0.0, 1.02.0, 1.0.0-01, 1.0.0-alpha..1, 1.0.0-, 1.0.0+,
1.0.0+meta_data, v1.0.0, 1.0

Output only the regex.
```

## Grading notes

The reference is the official SemVer 2.0.0 regex (semver.org); any equivalent
formulation passing all embedded cases is acceptable verbatim — do not patch for
group naming, non-capturing groups, or factoring. **Patch or reject** a regex
that fails any embedded case; the usual draft failures are `\d+` cores that
accept `01.0.0`, pre-release classes that accept `1.0.0-01` or empty identifiers
(`alpha..1`), `\w` classes that accept underscores in build metadata, and
unanchored patterns that accept `v1.0.0`. Check the cases mechanically before
accepting — plausibility is not verification.
