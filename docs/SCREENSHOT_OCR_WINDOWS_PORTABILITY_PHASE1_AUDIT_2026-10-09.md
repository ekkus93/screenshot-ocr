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
- generic content-free platform diagnostics (Linux XDG desktop/session labels are
  normalized to a bounded allowlist, never copied verbatim);
- platform-provided shortcut text.

The repository-policy job runs `scripts/check-platform-boundaries.py`, which rejects known concrete Linux/Windows imports, Linux environment probes, Linux executable paths, and Windows API imports outside the platform tree.

The static boundary guard additionally rejects Windows registry bindings,
registry root constants, and Windows user-profile/environment-path probes in
shared Rust modules, including multiline environment reads. These checks are
synthetic regression protection, not a substitute for native loader review.

The portable settings store now delegates directory/file privacy protection
through two explicit `PlatformServices` methods. The Linux adapter retains
`0700` directory and `0600` file permissions; the Windows adapter preserves
the previous app-config ACL-inheritance behavior without claiming that native
ACL policy has been physically reviewed. The unsupported adapter fails closed.
The static boundary guard rejects production Unix/Windows filesystem permission
mechanisms in `settings.rs`, with synthetic regression tests. This narrows the
platform contract to actual settings-storage needs and does not complete the
Windows ACL, physical desktop, or native redistribution acceptance gates.

The shared `src-tauri/src/error.rs` error codes/variants and public error
mapping are now platform-neutral: Linux capture failure codes remain part of
the stable serialized domain contract even when compiled on Windows.
`UnsupportedEnvironment` is likewise a stable code constructed only by the
unsupported-platform adapter. Synthetic source guard tests reject any new
`target_os` gating in this public error contract. This is compile/test
coverage, not physical capture or system-error-path acceptance.

The W1.2 source-review process now also generates a content-free, per-commit
`platform-assumption-inventory.json` artifact in repository-policy CI.
`scripts/audit-platform-assumptions.py` categorizes OS vocabulary, desktop
environment reads, executable paths, shortcut labels, target conditionals,
error mappings, resource paths, bindings, and build metadata by
repository-relative filename, source line number, and responsible architectural
owner. The inventory deliberately contains no source lines, user paths,
screenshots, or OCR text; synthetic tests enforce this output contract.

The Linux environment adapter additionally normalizes public
distribution/version diagnostics to fixed distro labels and bounded numeric
versions, rather than forwarding arbitrary `PRETTY_NAME` content from
`/etc/os-release`. Synthetic parser tests exercise unrecognized and malformed
values; hosted exact-head CI must still qualify this change.

The inventory also scans Tauri build scripts, capability/configuration files,
installer `.nsh`/`.nsi` hooks, and shell scripts, rather than only Rust source.
It classifies Windows registry/environment references and Linux/Windows bundle
targets explicitly; synthetic fixtures check the additional coverage while
excluding generated `target/` output. The expanded scan must pass exact-head
CI before being treated as qualified evidence.

Additional deterministic inventory coverage includes Rust `cfg!` macros,
target-family/architecture predicates, native error probes, WinAPI/registry
bindings, absolute Windows executable paths, Windows batch scripts, native
resource scripts, and C/C++ source. A symlinked scan root now fails closed.
Synthetic fixtures cover these cases without serializing matched source text.
The generated inventory still requires category-by-category architectural
review before any W1.2 checkbox is closed.

A subsequent checkout-path guard also rejects symlinked ancestor directories
above the checkout root. The synthetic test distinguishes a real final
checkout directory reached through a linked parent from a directly linked
checkout root. This prevents a caller from treating a redirected tree as
trusted first-party input; the exact-head CI gate must still qualify it.

This is a deterministic **keyword inventory**, not proof that every OS
assumption has been semantically audited or that native loader/desktop behavior
passes. Review the artifact, the current code, and the remaining physical
acceptance matrix before checking the W1.2 and W1 acceptance boxes.

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

A subsequent portable-production boundary check rejects Rust `cfg`,
`cfg_attr`, and `cfg!` platform switches (including nested `all`/`any`/`not`)
outside platform adapters and bootstrap code. Test-only fixtures remain
permitted to use target conditionals. Synthetic regressions cover these
cases; this is architecture source-gating evidence, not physical or native
loader acceptance, and requires exact-head CI before qualification.

The runtime-model source-guard finding was addressed without a portable-core
lint exemption: Windows now uses the same `OcrRuntime::new` constructor as
Linux and then selects its own app-controlled working directory. This leaves
the shared descriptor free of `target_os` and blanket dead-code suppression.
CI on the exact head is still required; physical desktop and DLL/license review
are separate, unresolved acceptance gates.

The platform-assumption inventory now attributes `src-tauri/src/platform.rs`
(the actual composition root) to `platform_composition` rather than the
shared-core bucket. A synthetic classification fixture prevents regressions.
This fixes owner attribution only; it does not close W1.2 semantic-review
checkboxes or physical acceptance gates until exact-head CI qualifies it.
