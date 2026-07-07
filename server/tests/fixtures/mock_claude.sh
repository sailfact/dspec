#!/usr/bin/env bash
# Mock of `claude -p`. Behavior selected via MOCK_MODE.
cat > /dev/null # consume stdin like the real CLI
case "${MOCK_MODE:-ok}" in
  ok)   echo "mock output" ;;
  json) echo '{"confidence": 85, "reasons": ["complete"]}' ;;
  fail) echo "boom" >&2; exit 2 ;;
  slow) sleep 5; echo "late" ;;
esac