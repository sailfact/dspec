# CLAUDE.md

@AGENTS.md

## Claude Code notes
The shared rules are in AGENTS.md (imported above). Everything below is Claude-specific.

- When chasing a failing test, run it alone first (`cargo test <name>`) — the full suite is slow.
- Keep diffs small and reviewable; don't refactor unrelated code in the same change.
- Path-scoped rule: in `src/cli/`, every public command must have a `--help` example in its doc comment.