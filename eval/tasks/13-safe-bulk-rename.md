# Task 13 — bash: bulk rename safe for hostile filenames

| field | value |
|---|---|
| id | `13-safe-bulk-rename` |
| category | trap (looks mechanical; word-splitting pitfall) |
| expected gate | high — a calibrated gate scoring this low is a *bonus* |
| expected outcome | patched / rejected |

**Why this task:** an overconfidence probe. "Rename `*.jpeg` to `*.jpg`" looks
like task 07's sibling, but the explicit spaces-and-newlines requirement rules
out the idioms a cheap draft reaches for first: `for f in $(find …)` (word
splitting), `find … | while read f` (breaks on leading/trailing whitespace and
backslashes, and on newlines entirely), and unquoted `$f` expansions. A draft
can be entirely plausible and still corrupt or miss files — precisely the
demonstrably-wrong-span case verify-and-patch exists for.

## Prompt

Paste everything below after `/spec`:

```
Write a bash one-liner that renames every regular file ending in .jpeg to the
same name ending in .jpg, under the directory given as $1, recursively. It must
be correct for filenames and directories containing spaces, tabs, newlines, and
leading dashes. Assume GNU find and bash. Do not use the Perl rename utility.
Output only the one-liner.
```

## Grading notes

Accept any pipeline that never word-splits paths, e.g.
`find "$1" -depth -type f -name '*.jpeg' -exec bash -c 'for f; do mv -- "$f" "${f%.jpeg}.jpg"; done' _ {} +`
or a `-print0 | while IFS= read -rd '' f` loop with quoted expansions and
`mv --`. **Patch or reject** on any of: `$(find …)` in a `for` list, a newline-
delimited `while read` loop, unquoted `$f`/`$1`, or `mv` without `--` /
equivalent protection against leading-dash names (patch-scale if the rest is
sound). Choice among *safe* idioms (`-exec` vs `-print0`) is not grounds to
patch.
