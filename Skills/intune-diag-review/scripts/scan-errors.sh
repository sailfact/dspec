#!/usr/bin/env bash
# scan-errors.sh — First-pass error triage over extracted diagnostic files.
#
# Usage: bash scripts/scan-errors.sh [workdir]
#   workdir defaults to /tmp/diagreview (must already contain the output of
#   extract-diag.sh)
#
# Surfaces:
#   1. Failed collections (non-zero HRESULT) from results.xml.
#   2. Error/warning lines from the IME logs (SCCM-style; type="3" = error).
set -euo pipefail

WORKDIR="${1:-/tmp/diagreview}"

echo "=== Failed collections (non-zero HRESULT in results.xml) ==="
if [[ -f "$WORKDIR/results.xml" ]]; then
  if grep -o 'HRESULT="[^"]*"' "$WORKDIR/results.xml" | grep -v 'HRESULT="0"' | sort | uniq -c; then
    :
  else
    echo "(all collections returned HRESULT 0)"
  fi
else
  echo "(results.xml not found — run extract-diag.sh first)"
fi

echo
echo "=== IME log errors ($WORKDIR/ime) ==="
if compgen -G "$WORKDIR/ime/*.log" > /dev/null; then
  grep -i "error\|failed\|0x8\|exception" "$WORKDIR"/ime/*.log 2>/dev/null | head -50 \
    || echo "(no matching error lines)"
else
  echo "(no IME logs extracted)"
fi