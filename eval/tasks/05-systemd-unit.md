# Task 05 — systemd unit with restart-on-failure

| field | value |
|---|---|
| id | `05-systemd-unit` |
| category | mechanical |
| expected gate | high (≥ threshold) |
| expected outcome | accepted / patched |

**Why this task:** a systemd unit from a fixed spec is boilerplate with
well-known directives.

## Prompt

Paste everything below after `/spec`:

```
Write a systemd service unit file for a long-running daemon. Spec:
- Description: "dspec telemetry sync".
- Runs /usr/local/bin/dspec-sync as user "dspec", group "dspec".
- Working directory /var/lib/dspec.
- Restart on failure only (not on clean exit), with a 5-second restart delay.
- Start after the network is online, and want network-online.target.
- Enable under the multi-user target.
Output only the unit file contents.
```

## Grading notes

Accept if it has `[Unit]`, `[Service]`, `[Install]` sections with
`After=network-online.target` + `Wants=network-online.target`, `ExecStart` at the
given path, `User=dspec`/`Group=dspec`, `WorkingDirectory=/var/lib/dspec`,
`Restart=on-failure`, `RestartSec=5`, and `WantedBy=multi-user.target`. Patch only
a missing/incorrect directive (e.g. `Restart=always` instead of `on-failure`).
