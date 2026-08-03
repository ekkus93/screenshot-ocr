#!/usr/bin/env bash
# PostToolUse (Write|Edit) hook: format the just-edited file in place.
set -uo pipefail

f=$(jq -r '.tool_input.file_path // empty')
[ -z "$f" ] && exit 0

case "$f" in
  *.ts | *.tsx) timeout 20 npx --no-install prettier --write "$f" >/dev/null 2>&1 || true ;;
  *.rs) timeout 20 rustfmt --edition 2021 "$f" >/dev/null 2>&1 || true ;;
esac

exit 0
