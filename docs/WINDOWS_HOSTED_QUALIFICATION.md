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

## WebView2 provisioning policy

The experimental NSIS bundle uses
`bundle.windows.webviewInstallMode.type = offlineInstaller` in
`src-tauri/tauri.nsis.conf.json`, rather than Tauri's default
`downloadBootstrapper`. The bundler includes Microsoft's offline WebView2
installer, adding approximately 127 MB according to Tauri's Windows packaging
documentation. Installer construction may fetch the WebView2 distribution
**during the build**; this is not a runtime OCR network dependency.

The NSIS archive verifier also requires exactly one embedded
`MicrosoftEdgeWebView2RuntimeInstaller.exe` payload with a plausible standalone
installer size, and records its SHA-256 in hosted CI logs. This static archive
inspection does not validate Microsoft's installer signature, native
redistribution obligations, execution, or disconnected installation.

Bundling the offline installer is only a packaging design decision. CI does
not execute NSIS setup on a clean disconnected Windows desktop. Physical
WebView2 provisioning, first launch, clean uninstall, and both Windows 10/11
platform acceptance remain **unverified**; no redistribution approval or
signed installer is implied.

## Developer-only Windows build

On a Windows x86-64 development machine, install the pinned Node and Rust
toolchains, MSVC C++ build tools, Windows SDK, WebView2/Tauri prerequisites,
Python 3, and PowerShell. These are development dependencies; Windows users
must not install Linux GNOME/XDG Portal or Debian packages.

From the repository root in PowerShell:

```powershell
npm ci
npm run tauri -- icon src-tauri/icons/128x128.png
./scripts/audit-windows-ocr-runtime.ps1 -StageRuntime
python scripts/audit-windows-pe-imports.py src-tauri/windows-ocr-runtime
npm run tauri -- build --bundles nsis --config src-tauri/tauri.nsis.conf.json
./scripts/verify-windows-nsis-payload.ps1
```

Staging fetches checksum-pinned OCR inputs at **build time**; the app must
not fetch OCR models at runtime. This procedure creates an unapproved NSIS
test candidate, **not** a redistributable release. The installer remains
unpublished pending license/NOTICE and physical Windows acceptance.

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
