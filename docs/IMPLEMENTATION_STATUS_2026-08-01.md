# Screenshot OCR Implementation Status — 2026-08-01

**Repository:** `ekkus93/screenshot-ocr`  
**Branch:** `master`  
**Authoritative plan:** `docs/SCREENSHOT_OCR_V0_1_TODO.md`  
**Status:** Active implementation; not release-complete

## 1. Current repository state

The project has been scaffolded as a Tauri 2 application with:

- React, TypeScript, Vite, and Tailwind CSS frontend;
- Rust backend and reusable application services;
- committed npm and Cargo lockfiles;
- Ubuntu 22.04 hosted frontend, Rust, repository-policy, and Debian-package gates;
- ChatGPT-readable CI status issue `#1`;
- all first-party source files below 800 lines;
- zero-warning lint and Clippy policies with no suppressions added for first-party code.

The current `master` candidate at the time of this report is:

```text
LATEST_MASTER_SHA=e12214738212ed4ec71b561a207bff8b76c642e2
```

## 2. Last fully proven automated baseline

Before the runtime-hardening and startup-capture batches, commit:

```text
83f7841377e59e13d4a756d8dd92a13e263cb660
```

proved all of the following on Ubuntu 22.04 hosted runners:

- repository policy and required-file validation;
- first-party source files at or below 800 lines;
- no wildcard Tauri capabilities;
- frontend formatting;
- frontend ESLint with no warnings accepted;
- frontend TypeScript checking;
- frontend tests;
- frontend production build;
- Rust formatting;
- Rust Clippy with `-D warnings`;
- Rust unit tests;
- Rust workspace build.

The Debian package job had entered its build phase but was superseded by later implementation commits before a package artifact could be accepted as release evidence.

## 3. Implemented after the proven baseline

### 3.1 Capture security and cleanup

- Added a `CaptureBackend` abstraction.
- Added trusted executable discovery restricted to absolute system directories.
- Rejects helper names containing path separators.
- Requires discovered helpers to be executable regular files rather than symlinks.
- GNOME capture uses null stdin/stdout/stderr and `kill_on_drop`.
- Region capture is bounded by a 120-second timeout.
- Capture output must be a nonempty regular non-symlink file.
- Capture artifacts are placed in a private per-capture directory.
- Cleanup runs after success, cancellation, process failure, timeout, and decode failure.
- Cleanup failure is returned as `TemporaryCleanupFailed` instead of being hidden.
- Added tests for executable discovery, nested cleanup, and symlink rejection.

### 3.2 OCR hardening

- Tesseract stderr is not retained or exposed.
- OCR stdout is bounded while being read.
- OCR output is limited to 1,000,000 bytes.
- OCR is bounded by a 30-second timeout.
- Language discovery output is bounded and filtered.
- English language data remains mandatory.

### 3.3 Backend selection and clipboard behavior

- Capture now honors the saved backend preference.
- `Auto` and `Gnome` use the trusted GNOME helper when available.
- Explicit portal selection fails closed until an actual portal backend is implemented.
- Immediate-copy policy now writes recognized text to the clipboard and marks the result copied.
- Window hide, show, and focus errors are checked.
- Clipboard input rejects empty and oversized text while preserving code whitespace.

### 3.4 Startup capture command

The documented command is now wired through the application:

```text
screenshot-ocr capture
```

Implementation properties:

- only the exact first argument `capture` is accepted;
- the Rust backend stores a one-shot startup intent;
- the Tauri command consumes the intent atomically;
- the React controller waits for settings and diagnostics to load;
- the normal configured capture path is then invoked;
- the intent cannot be replayed by frontend rerenders;
- argument parsing and one-shot consumption have Rust tests;
- frontend tests mock the startup-intent command explicitly.

## 4. Current CI blocker

The CI status bridge is currently stale.

Issue `#1` still reports:

```text
RUN_ID=30724012865
HEAD_SHA=68f7f6c4579028ed8ca3389b8b62aca96ad974ad
STATUS=queued
JOBS_VISIBLE=0
```

That run was created for an intermediate commit and no longer represents `master`.

Multiple later commits, including the current workflow-trigger commit, have not appeared in issue `#1`. The GitHub connector's commit-run lookup only discovers pull-request-triggered runs, so it cannot independently discover the missing ordinary push run. Therefore:

- the runtime-hardening batch is implemented but not yet accepted as CI-green;
- the startup-capture batch is implemented but not yet accepted as CI-green;
- no Debian package artifact is accepted for the latest SHA;
- no TODO parent milestone depending on those proofs should be marked complete.

## 5. Exact next automated actions

1. Restore reliable push-run discovery or allow the current `CI` push run to be created and published.
2. Require issue `#1` to report the exact current `master` SHA.
3. Fix any formatter, frontend lint/typecheck/test/build, Rustfmt, Clippy, Rust test, or Rust build failure without suppression.
4. Require the Debian package smoke job to produce a `.deb` and SHA-256 artifact on the same exact SHA.
5. Update the TODO only with tasks proven by that run.
6. Continue with the remaining product work below.

## 6. Remaining implementation work

### Automated/product work still open

- Real XDG Screenshot Portal area-capture implementation and capability detection.
- Cancellation that terminates the active helper/OCR process rather than only clearing state.
- Single-instance behavior for repeated shortcut launches.
- Tray menu and close-to-tray behavior.
- Start-at-login behavior or removal of the unimplemented setting from v0.1.
- Cleanup-failure counters and last safe error code in diagnostics.
- Corrupt-settings warning surfaced to the frontend.
- Stronger settings migration and interrupted-write tests.
- Deterministic OCR fixtures and real Tesseract integration tests.
- Additional image preprocessing variants and confidence-based selection evidence.
- Notification behavior after copy.
- Package install/uninstall/upgrade validation.
- TODO checkbox and evidence updates.

### Physical desktop validation still mandatory

Hosted CI cannot honestly prove:

- Ubuntu 22.04 GNOME Wayland capture;
- Ubuntu 22.04 GNOME X11 capture;
- Ubuntu 24.04 GNOME Wayland capture;
- Ubuntu 24.04 GNOME X11 capture;
- custom GNOME shortcut behavior on a real session;
- multi-monitor and fractional-scaling coordinate behavior;
- real clipboard ownership and paste behavior;
- dark/light terminal OCR quality;
- selector cancellation and timeout behavior on the desktop;
- installed `.deb` launcher, icon, dependency, upgrade, and uninstall behavior.

These remain open and must not be replaced by hosted compile-only evidence.

## 7. Quality-policy compliance

- Development remained on `master`.
- No branch or pull request was created.
- No first-party lint warning was hidden, suppressed, downgraded, or ignored.
- Meaningless constant-only tests flagged by Clippy were removed rather than allowed.
- All current first-party code files remain below 800 lines according to the repository policy gate.
- Third-party dependency warnings were not treated as first-party defects.
