# Phase 1 portability audit

**Date:** 2026-10-09  
**Branch:** `ralph/windows-support`  
**Baseline:** `bbfd605d5c65b83735a9795622d92062c6c9c59d`  
**Companion documents:** `SCREENSHOT_OCR_WINDOWS_PORTABILITY_SPEC_2026-10-09.md` and `SCREENSHOT_OCR_WINDOWS_PORTABILITY_TODO_2026-10-09.md`

## Purpose

This note records the Phase 1 inventory and the evidence used to decide whether the Linux application has a portable application core with operating-system mechanisms behind a narrow platform boundary. It does not claim Windows feature completion or physical desktop validation.

## Module classification

| Area                   | Classification              | Notes                                                                         |
| ---------------------- | --------------------------- | ----------------------------------------------------------------------------- |
| `app.rs`               | portable application policy | Owns capture/OCR sequencing and consumes only `PlatformServices`              |
| `actions.rs`           | portable application policy | Constrained action router shared by desktop activation sources                |
| `cancellation.rs`      | portable core               | Platform-neutral cancellation token                                           |
| `capture/mod.rs`       | portable contract           | Defines only `CaptureBackend`                                                 |
| `commands.rs`          | shared Tauri adapter        | Shared DTO, clipboard, and window-lifecycle wiring                            |
| `desktop.rs`           | shared Tauri adapter        | Tray/action policy is shared; shortcut selection comes from platform services |
| `diagnostics.rs`       | portable aggregation        | Accepts content-free `PlatformDiagnostics`                                    |
| `error.rs`             | portable error policy       | Stable public failures; unsupported-OS handling is only a fallback target     |
| `image_pipeline/`      | portable core               | Bounded decode and deterministic preprocessing                                |
| `models.rs`            | portable domain             | Stable backend identity includes `windows_native` without implementation code |
| `ocr/`                 | portable core               | Bounded Tesseract execution, cleanup/scoring, and `OcrRuntime`                |
| `settings.rs`          | portable core               | Shared schema, persistence, quarantine, and recovery                          |
| `state.rs`             | portable core               | One-active-capture state machine                                              |
| `platform/services.rs` | platform contract           | Narrow capability surface used by shared policy                               |
| `platform/linux/`      | Linux adapter               | Environment probing, backend selection, GNOME, portal, Linux OCR discovery    |
| `platform/windows/`    | Windows adapter             | Phase 2 composition point; native mechanisms are intentionally incomplete     |
| `platform/mod.rs`      | composition root            | The only target selector for platform services                                |
| `lib.rs`, `main.rs`    | bootstrap                   | Shared Tauri application entry points                                         |

## Platform-assumption inventory

Phase 1 moved these Linux mechanisms under `platform/linux/`:

- GNOME Screenshot helper discovery and capture;
- XDG Desktop Portal and `ashpd`;
- `DISPLAY`, `WAYLAND_DISPLAY`, XDG desktop/session probing;
- Linux capture backend preference resolution;
- Linux direct-shortcut selection;
- Linux Tesseract executable discovery.

`ashpd` and `url` are Linux-target Cargo dependencies. No Windows API dependency is added until Phase 2 selects a capture mechanism.

Shared code intentionally retains only platform-neutral concepts:

- a stable capture backend ID;
- a generic captured image;
- a generic OCR runtime descriptor containing an internally resolved executable and optional tessdata directory;
- generic content-free platform diagnostics;
- platform-provided shortcut text.

The repository-policy job runs `scripts/check-platform-boundaries.py`, which rejects known concrete Linux/Windows imports, Linux environment probes, Linux executable paths, and Windows API imports outside the platform tree.

## Frontend inventory

OCR review, copy recovery, job state, cancellation, and the general settings flow remain shared. Linux-only backend choices and shortcut copy are Phase 2 truthfulness work for the Windows UI; they do not justify a second frontend.

## Settings migration decision

The settings schema remains shared for compatibility. Existing `auto`, `gnome`, and `portal` values remain parseable and valid if a profile is moved between operating systems.

The Windows platform adapter does not execute a Linux backend choice. It owns a single Windows-native capture mechanism regardless of a migrated Linux preference. Phase 2 must filter Linux-only backend choices and show the Windows shortcut truthfully in the UI.

This avoids quarantining an otherwise valid Linux-origin settings file merely because it is opened on Windows.

## Preserved non-negotiable behavior

Phase 1 preserves:

- one-active-capture ownership;
- Rust-authoritative cancellation;
- no shell invocation;
- bounded Tesseract subprocesses;
- no OCR history;
- content-free diagnostics;
- no broad Tauri filesystem/shell capability;
- recoverable clipboard failure;
- settings quarantine/recovery;
- GNOME Screenshot and XDG Portal Linux capture.

## Validation evidence

The baseline master commit `bbfd605d5c65b83735a9795622d92062c6c9c59d` passed CI run `33818790571`.

The structural Phase 1 refactor at `914061ff7973a215deb690ae838022e22df07a35` passed CI run `38019029823`, including:

- repository policy and the platform-boundary guard;
- frontend format, lint, typecheck, tests, and build;
- Playwright layout regression;
- Rust format, Clippy with warnings denied, tests, and build;
- Debian package build, metadata inspection, checksum generation, and artifact upload.

Portable orchestration fake-platform tests were subsequently added on the same branch. Their final qualified SHA/run is recorded in the authoritative TODO after exact-head CI completes.

## Physical evidence boundary

No new physical Linux desktop acceptance is asserted by this audit. The TODO's GNOME Wayland/X11, tray, shortcut, clipboard, and display checks remain unchecked unless they are actually re-run. They are not treated as hosted-CI evidence.
