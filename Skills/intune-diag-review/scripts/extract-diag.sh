#!/usr/bin/env bash
# extract-diag.sh — Orient and extract an Intune "Collect Diagnostics" package.
#
# Usage: bash scripts/extract-diag.sh <DiagLogs-*.zip> [workdir]
#   workdir defaults to /tmp/diagreview
#
# Performs the deterministic setup steps from SKILL.md Steps 1-2:
#   1. Print the results.xml manifest (non-zero HRESULT = failed collection).
#   2. Extract text-readable files (.log, .reg, results.xml) to <workdir>.
#   3. Extract IME logs to <workdir>/ime/.
#   4. Decode every .reg file from UTF-16LE to UTF-8 *in place*. Windows registry
#      exports are UTF-16LE; cat/grep on them silently returns nothing, so this
#      script normalises them once and all later greps work directly.
set -euo pipefail

ZIP="${1:?Usage: bash scripts/extract-diag.sh <zip> [workdir]}"
WORKDIR="${2:-/tmp/diagreview}"

if [[ ! -f "$ZIP" ]]; then
  echo "Error: zip not found: $ZIP" >&2
  exit 1
fi

mkdir -p "$WORKDIR" "$WORKDIR/ime"

echo "=== Manifest (results.xml) ==="
unzip -p "$ZIP" results.xml 2>/dev/null || echo "(results.xml not found in package)"

echo
echo "=== Extracting readable files to $WORKDIR ==="
unzip -j -o "$ZIP" "*.log" "*.reg" "results.xml" -d "$WORKDIR" 2>/dev/null || true

echo "=== Extracting IME logs to $WORKDIR/ime ==="
unzip -j -o "$ZIP" "*IntuneManagementExtension_Logs/*" -d "$WORKDIR/ime" 2>/dev/null || true

echo "=== Decoding .reg files (UTF-16LE -> UTF-8) ==="
shopt -s nullglob
for reg in "$WORKDIR"/*.reg; do
  tmp="$(mktemp)"
  if iconv -f UTF-16LE -t UTF-8 "$reg" > "$tmp" 2>/dev/null; then
    mv "$tmp" "$reg"
    echo "  decoded: $(basename "$reg")"
  else
    rm -f "$tmp"
    echo "  WARN: could not decode $(basename "$reg")" >&2
  fi
done

echo
echo "Done. Extracted files in: $WORKDIR"
echo "Next: bash scripts/scan-errors.sh $WORKDIR"