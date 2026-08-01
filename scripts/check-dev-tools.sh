#!/usr/bin/env bash
set -euo pipefail

missing=0
for command in node npm rustc cargo pkg-config; do
  if command -v "${command}" >/dev/null 2>&1; then
    printf '%-18s %s
' "${command}" "available"
  else
    printf '%-18s %s
' "${command}" "missing"
    missing=1
  fi
done

for package in webkit2gtk-4.1 ayatana-appindicator3-0.1; do
  if pkg-config --exists "${package}" 2>/dev/null; then
    printf '%-18s %s
' "${package}" "available"
  else
    printf '%-18s %s
' "${package}" "missing"
    missing=1
  fi
done

exit "${missing}"
