---
description: Open the dspec live tmux dashboard (draft, gate, and events panes)
---

# /spec-watch

Run the dashboard launcher with the Bash tool:

    "${CLAUDE_PLUGIN_ROOT}/bin/dspec-watch"

If `CLAUDE_PLUGIN_ROOT` is not set in your environment, locate the installed
dspec plugin directory and run `bin/dspec-watch` from there.

- If the script exits with "tmux not found", tell the user to install tmux
  (>= 3.1) and stop.
- If the script prints an attach hint (it always will from this non-TTY
  context when the user is not already inside tmux), relay it verbatim:
  the user should run `tmux attach -t dspec` in their terminal.
- If the user is already inside tmux, tell them a new window named `dspec`
  has been added to their current session.
- Do not tail or read the log files yourself; the dashboard is for the
  user's eyes, not context for you.