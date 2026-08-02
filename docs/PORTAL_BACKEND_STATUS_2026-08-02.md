# XDG Screenshot Portal Backend Status — 2026-08-02

**Repository:** `ekkus93/screenshot-ocr`  
**Branch:** `master`  
**Status:** Implemented; hosted validation pending

## Implemented behavior

The Rust capture layer now includes an XDG Screenshot Portal backend using `ashpd` 0.13.13.

The backend:

- probes the Screenshot portal with a bounded timeout;
- requires Screenshot portal interface version 3 or newer;
- requires the portal to advertise the `Area` capture target;
- requests an interactive, modal, single-area selection;
- maps cancellation, permission denial, timeout, unsupported portal versions, and general portal failures to stable application errors;
- accepts only absolute `file:` result URIs;
- rejects missing, empty, oversized, non-regular, and symlink capture results;
- bounds capture-file reads to 20 MiB;
- decodes the image through the existing bounded image pipeline;
- does not log the result URI, path, pixels, or recognized text.

## Backend selection contract

Selection happens before any selector is opened:

- `Auto` uses the portal only when version 3+ and `Area` support are proven;
- `Auto` otherwise falls back to the trusted GNOME screenshot helper;
- explicit `Portal` fails closed when area support is unavailable;
- explicit `Gnome` does not probe or invoke the portal.

There is no post-selector fallback. Once a portal selector has been requested, a portal failure is returned to the caller rather than opening a second unexpected selector.

## Automated tests added

- portal version and area-target requirements;
- backend preference and fallback policy;
- unsupported and non-file URI rejection;
- oversized result rejection;
- symlink result rejection;
- stable public portal error codes.

## Validation boundary

This document does not claim the portal backend is complete or release-ready until the same exact commit passes:

- Rustfmt;
- Clippy with `-D warnings`;
- Rust tests;
- Rust workspace build;
- frontend mandatory gates;
- Ubuntu 22.04 Debian package smoke.

Physical Ubuntu 24.04 GNOME Wayland validation is also required to prove the real portal selector, cancellation, permission handling, multiple monitors, and fractional scaling.

## Remaining portal work

- verify the exact `ashpd` API usage in hosted compilation;
- investigate explicit request closure on timeout/cancellation where the portal API exposes a request handle;
- surface safe portal capability details in diagnostics;
- validate cancellation and permission-denial behavior on a real GNOME session;
- update `docs/SCREENSHOT_OCR_V0_1_TODO.md` only after the required evidence exists.
