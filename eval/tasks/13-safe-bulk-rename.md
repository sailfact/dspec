# Task 13 — bash: bulk rename safe for hostile filenames

| field | value |
|---|---|
| id | `13-safe-bulk-rename` |
| category | trap (looks mechanical; word-splitting pitfall) |
| expected gate | high — a calibrated gate scoring this low is a *bonus* |
| expected outcome | rejected / patched |

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

Accept any pipeline that never word-splits paths **and** survives `$1` itself
starting with a dash — GNU find parses a leading-dash start point as a
predicate (`find "-foo"` fails with `unknown predicate '-foo'`), so a bare
`find "$1" …` does not meet the prompt. The canonical safe form runs find from
inside the directory:
`(cd -- "$1" && find . -depth -type f -name '*.jpeg' -exec bash -c 'for f; do mv -- "$f" "${f%.jpeg}.jpg"; done' _ {} +)`
— every result then starts with `./`, which also neutralizes leading-dash
entries. A `-print0 | while IFS= read -rd '' f` loop with quoted expansions,
`mv --`, and an equivalent dash-safe start point is equally fine.

**Record `rejected`** for a wrong traversal core: `$(find …)` in a `for` list
or a newline-delimited `while read` loop — the unsafe idiom *is* the answer,
so fixing it is regeneration, not a patch (and only `rejected` outcomes feed
`mean_confidence_bad` — see the eval README). **Patch-scale** if the traversal
is sound but a peripheral protection is missing: an unquoted `$f`/`$1`, `mv`
without `--`, or a bare `find "$1"` start point. Choice among *safe* idioms
(`cd`-subshell vs `-print0`) is not grounds to patch.
