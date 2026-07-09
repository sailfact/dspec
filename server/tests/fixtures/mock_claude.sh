#!/usr/bin/env bash
# Mock of `claude -p`. Behavior selected via MOCK_MODE.
cat > /dev/null # consume stdin like the real CLI
case "${MOCK_MODE:-ok}" in
  ok)   echo "mock output" ;;
  json) echo '{"confidence": 85, "reasons": ["complete"]}' ;;
  stream)
    echo '{"type":"system","subtype":"init"}'
    echo '{"type":"stream_event","event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"hel"}}}'
    echo '{"type":"stream_event","event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"lo"}}}'
    echo '{"type":"result","subtype":"success","result":"hello"}'
    ;;
  fail) echo "boom" >&2; exit 2 ;;
  slow) sleep 5; echo "late" ;;
esac