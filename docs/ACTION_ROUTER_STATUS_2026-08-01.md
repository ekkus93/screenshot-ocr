# Cross-platform Action Router Status — 2026-08-01

**Repository:** `ekkus93/screenshot-ocr`  
**Branch:** `master`  
**Decision record:** `docs/adr/0002-cross-platform-activation-and-cancellation.md`  
**Status:** Implemented; exact-SHA hosted validation pending

## Implemented behavior

The application now has one Rust-owned action router for:

- toggling capture;
- starting capture;
- cancelling the active capture;
- showing the main window;
- quitting after active capture cleanup.

The following activation sources route through that same controller:

- initial command-line invocation;
- a second process handled by Tauri single-instance support;
- a directly registered global shortcut where supported;
- the tray or future macOS menu-bar menu;
- the existing React main-window controls.

External start actions reserve a Rust capture job before notifying React. React consumes the reserved job identifier and source through a typed Tauri command, then invokes the existing `start_capture` command. A bounded reservation timeout prevents an unresponsive webview from retaining capture ownership indefinitely.

Toggle and cancel actions signal the active Rust cancellation token directly. They therefore remain reachable while the main window is hidden. Job ownership is not released until the normal capture or OCR process termination and cleanup path finishes.

## Platform behavior

- GNOME Wayland: use a custom shortcut invoking `screenshot-ocr toggle`; direct registration is deliberately deferred.
- Linux X11: attempt direct registration of `Super+Shift+O`.
- Windows future build: use `Control+Shift+O` through the same action router.
- macOS future build: use `Command+Shift+O` through the same action router.

The tray or menu-bar menu exposes Capture text, Cancel capture, Show Screenshot OCR, and Quit. Tray creation or shortcut registration failure is nonfatal and is reflected in safe diagnostics.

## Safety properties

- No activation adapter launches capture or OCR independently.
- Only the documented action vocabulary is accepted from command-line or second-instance input.
- Repeated start requests cannot create overlapping selectors.
- Cancellation does not clear job ownership early.
- Quit waits for active cancellation and cleanup before exiting.
- Pending actions contain only a random job identifier, a constrained action enum, and a constrained source enum.
- No screenshot, OCR text, clipboard text, path, URI, or subprocess output is added to diagnostics.

## Automated coverage added

- supported and unsupported command parsing;
- second-instance fallback to showing the existing window;
- activation-source mapping;
- capture reservation consumption and expiration;
- repeated start and cancellation ownership behavior;
- typed action serialization;
- pending-action queue consumption and overwrite rejection;
- Linux direct-shortcut policy;
- React consumption of a reserved shortcut-originated capture request.

## Validation boundary

This document does not claim the batch is accepted until one exact clean `master` SHA passes:

- repository policy and the 800-line first-party source limit;
- frontend formatting, zero-warning lint, typecheck, tests, and production build;
- Rustfmt, Clippy with warnings denied, Rust tests, and workspace build;
- Ubuntu 22.04 Debian package build, inspection, checksum, and artifact upload.

Physical validation of tray availability, GNOME custom shortcuts, direct X11 shortcuts, repeated invocation, hidden-window cancellation, and process cleanup remains required on supported Ubuntu desktop sessions.
