# Task 01 — `.gitignore` for a Rust cargo workspace

| field | value |
|---|---|
| id | `01-gitignore-rust` |
| category | mechanical |
| expected gate | high (≥ threshold) |
| expected outcome | accepted / patched |

**Why this task:** boilerplate with a single conventional right answer; the gate
should be confident and the verifier should accept it near-verbatim.

## Prompt

Paste everything below (verbatim, including the constraints) after `/spec`:

```
Write a .gitignore for a Rust cargo workspace. Constraints:
- The workspace has two members, server/ and xtask/, each a normal cargo crate.
- Ignore all cargo build output.
- Do NOT ignore Cargo.lock (this workspace ships applications, not a library).
- Ignore common editor/OS cruft: .vscode/, .idea/, *.swp, .DS_Store.
- Ignore a local ./scratch/ directory used for throwaway files.
Output only the .gitignore contents.
```

## Grading notes

A correct draft ignores `target/` (or `/target` plus `**/target/`), keeps
`Cargo.lock` un-ignored, and lists the editor/OS entries and `scratch/`. Accept
verbatim if all constraints are met. Patch only if an explicit constraint is
violated (e.g. it ignores `Cargo.lock`). Phrasing/ordering differences are **not**
grounds to patch.
