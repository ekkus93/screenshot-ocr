#!/usr/bin/env bash
# PostToolUse (Write|Edit) hook: catch CI-fatal debug/logging leaks immediately
# after editing first-party source, instead of waiting for CI.
set -uo pipefail

f=$(jq -r '.tool_input.file_path // empty')
case "$f" in
  *.rs | *.ts | *.tsx | *.js | *.jsx) ;;
  *) exit 0 ;;
esac

out=$(python3 scripts/check-content-leakage.py 2>&1)
code=$?

if [ "$code" -ne 0 ]; then
  reason=$(printf '%s' "$out" | jq -Rs .)
  printf '{"decision":"block","reason":%s}\n' "$reason"
fi

exit 0
