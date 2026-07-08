# Task 07 — bash one-liner: 10 largest files

| field | value |
|---|---|
| id | `07-bash-one-liner` |
| category | mechanical |
| expected gate | high (≥ threshold) |
| expected outcome | accepted / patched |

**Why this task:** a small, well-specified shell idiom.

## Prompt

Paste everything below after `/spec`:

```
Write a single bash one-liner that prints the 10 largest regular files anywhere
under a directory given as $1, recursively. Output human-readable sizes, largest
first. Assume GNU coreutils and find are available. Output only the one-liner.
```

## Grading notes

Accept any correct pipeline that recurses (`find "$1" -type f`), obtains sizes,
sorts descending, and takes the top 10 with human-readable sizes — e.g.
`find "$1" -type f -printf '%s\t%p\n' | sort -rn | head -n 10 | cut -f2- | xargs -d '\n' du -h`,
or the common `find … -exec du -h {} + | sort -rh | head -n 10`. Patch only a real
bug (not recursive, wrong sort order, includes directories, wrong count). Choice
among correct idioms is not grounds to patch.
