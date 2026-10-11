# Windows hosted qualification matrix

**Branch:** `ralph/windows-support`  
**Status:** Windows candidate, not approved for distribution

## Exact-head qualification

Record the complete commit SHA and passing GitHub Actions run IDs for that
same SHA. Passing earlier commits do not qualify later changes. The `CI`
workflow runs on `master` and `ralph/windows-support`. Branch concurrency
may cancel superseded runs; inspect the newest exact head instead.

## Linux CI (Ubuntu 22.04)

- Repository policy: file limits, synthetic PE parser tests, content-leakage
  and platform-boundary guards, and Tauri capability checks.
- Frontend: locked npm install, formatting, lint, typecheck, tests, build,
  and Chromium/Playwright no-scroll layout tests.
- Rust: formatting, Clippy, tests, and workspace build.
- Debian packaging smoke on the Windows branch or tags: `.deb` build,
  package metadata, SHA-256, and CI artifact upload.

Linux hosted CI does not prove physical GNOME Wayland/X11 behavior.

## Windows CI (Windows Server 2022)

- Locked frontend build and pinned Rust formatting, Clippy, tests,
  workspace build, and Tauri link.
- Branch-only NSIS smoke: checksum-verified Tesseract/English data staging,
  static PE dependency inventory, synthetic English OCR execution,
  NSIS build, staged resource verification, and installer SHA-256.
- Separate Windows OCR runtime input audit workflow.

The NSIS smoke does **not** publish or upload a Windows installer.
The checksum is printed in CI logs, not distributed as a Windows asset.

## Outstanding evidence

Hosted CI cannot prove clean installation/uninstallation, fully offline
installation, WebView2 provisioning, tray/shortcut behavior, selector focus,
Escape cancellation, clipboard behavior, or physical mixed-DPI capture.
The [Windows acceptance TODO](SCREENSHOT_OCR_WINDOWS_PORTABILITY_TODO_2026-10-09.md)
requires physical Windows evidence tied to an exact commit.

Before distributing an installer, complete the native DLL provenance,
license, and NOTICE review in
[ADR 0005](adr/0005-windows-tesseract-provenance.md).
Do not upload private screenshots, OCR output, clipboard text, user paths,
or raw diagnostics. Final integration to `master` requires owner approval.
