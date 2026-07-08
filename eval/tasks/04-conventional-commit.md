# Task 04 — conventional commit message for a diff

| field | value |
|---|---|
| id | `04-conventional-commit` |
| category | mechanical |
| expected gate | high (≥ threshold) |
| expected outcome | accepted / patched |

**Why this task:** summarizing a concrete, embedded diff into a conventional
commit is mechanical. The diff is included so the task is self-contained.

## Prompt

Paste everything below after `/spec`:

````
Write a single Conventional Commits message (type(scope): subject, then a body)
for the diff below. Subject in imperative mood, <= 72 chars. Output only the
commit message.

```diff
diff --git a/server/src/config.rs b/server/src/config.rs
--- a/server/src/config.rs
+++ b/server/src/config.rs
@@ -12,6 +12,7 @@ pub struct Config {
     pub threshold: u8,
     pub timeout_secs: u64,
     pub data_dir: PathBuf,
+    pub claude_bin: String,
 }

@@ -28,6 +29,9 @@ impl Config {
         let timeout_secs = lookup("DSPEC_TIMEOUT_SECS")
             .and_then(|v| v.parse().ok())
             .unwrap_or(120);
+        let claude_bin = lookup("DSPEC_CLAUDE_BIN")
+            .filter(|v| !v.is_empty())
+            .unwrap_or_else(|| "claude".to_string());
         Config { threshold, timeout_secs, data_dir, claude_bin }
     }
 }
```
````

## Grading notes

Accept if the type is appropriate (`feat`), the scope reasonable (`config`), the
subject imperative and within length (e.g. `feat(config): make claude binary path
configurable`), and the body mentions the new `DSPEC_CLAUDE_BIN` env var defaulting
to `claude`. Patch only for a factual error (wrong type, misdescribes the change).
Word choice in the subject is not grounds to patch.
