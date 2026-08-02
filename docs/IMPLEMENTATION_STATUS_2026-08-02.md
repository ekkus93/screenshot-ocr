# Screenshot OCR Implementation Status — 2026-08-02

**Repository:** `ekkus93/screenshot-ocr`  
**Branch:** `master`  
**Authoritative plan:** `docs/SCREENSHOT_OCR_V0_1_TODO.md`  
**Status:** Active implementation; hosted build/package baseline is green; physical desktop validation is still required

## 1. Current accepted source baseline

The latest fully proven implementation commit is:

```text
ACCEPTED_SOURCE_SHA=2fdce81e5163c5234515fc26efeea48544077c88
CI_RUN_ID=30731505719
```

All permanent read-only workflow jobs passed on that exact source commit:

- repository policy;
- frontend formatting, lint, typecheck, tests, and production build;
- Rustfmt, Clippy with `-D warnings`, Rust tests, and workspace build;
- Ubuntu 22.04 Debian package build, inspection, checksum, and artifact upload.

Documentation-only evidence commits after the accepted source SHA do not change the validated binary behavior.

## 2. Current application architecture

The repository contains a Tauri 2 Linux desktop application with:

- React, TypeScript, Vite, and Tailwind CSS frontend;
- Rust application services and typed public errors;
- a capture-backend abstraction;
- GNOME screenshot-helper capture;
- XDG Screenshot Portal area capture using `ashpd` 0.13.13;
- local Tesseract OCR;
- bounded image decoding and preprocessing;
- preview and immediate-copy clipboard policies;
- persistent versioned settings;
- startup capture through `screenshot-ocr capture`;
- caller-owned capture job identifiers and cancellation tokens;
- privacy-safe in-memory runtime diagnostics;
- permanent Ubuntu 22.04 source-quality and Debian-package CI;
- ChatGPT-readable CI status issue `#1`.

The application remains local-first. Captured pixels and recognized text are not intentionally sent to a network service or persisted as history.

## 3. Capture backends

### 3.1 GNOME helper backend

Implemented properties:

- trusted executable discovery from absolute system paths;
- executable must be a regular non-symlink file;
- direct argument-vector invocation without a shell;
- `--area` and output-file capture;
- private per-capture temporary directory;
- bounded helper lifetime;
- explicit kill and reap on application cancellation or timeout;
- typed cancellation and helper-failure results;
- nonempty regular non-symlink output validation;
- bounded image decode;
- cleanup after success, cancellation, timeout, helper failure, and decode failure;
- cleanup failures surfaced instead of silently ignored.

### 3.2 XDG Screenshot Portal backend

Implemented properties:

- bounded portal capability probe;
- Screenshot interface version 3 or newer required;
- advertised `Area` target required;
- interactive modal single-area request;
- `Auto` selects the portal only when support is proven;
- `Auto` falls back to GNOME only before a selector is opened;
- explicit `Portal` selection fails closed when capability is unavailable;
- no second selector after a portal request fails;
- cancellation and timeout checks around portal operations;
- typed cancellation, permission-denial, unsupported, timeout, and general failures;
- absolute `file:` URI requirement;
- regular, non-symlink, nonempty, size-bounded portal result validation;
- bounded, content-free portal version and area-capability reporting for diagnostics.

Still open:

- explicit portal request-object closure where supported by the `ashpd` lifecycle;
- real GNOME/Wayland portal selection, permission, cancellation, monitor, and scaling validation.

## 4. OCR pipeline

Implemented properties:

- Tesseract executable discovery and English-language probe;
- terminal, document, and single-line page-segmentation modes;
- PNG input through stdin;
- bounded OCR output;
- bounded OCR process lifetime;
- explicit kill and reap on cancellation or timeout;
- cancellation propagated rather than treated as a failed candidate;
- whitespace-preserving cleanup;
- several preprocessing variants;
- best-candidate selection;
- typed empty-result and OCR failures;
- no OCR output included in public errors or panic messages.

Still open:

- deterministic real-Tesseract fixture coverage;
- confidence data from Tesseract rather than `None`;
- stronger terminal/code accuracy evidence;
- additional image variants and evidence-based scoring calibration.

## 5. Capture job lifecycle and cancellation

Implemented properties:

- the frontend creates a UUID job ID before invoking Rust;
- the Rust state machine owns one active job at a time;
- a second request while active returns `CaptureAlreadyActive`;
- cancellation signals a shared token without clearing ownership early;
- start/cancel/start cannot acquire a second selector until the first command finishes;
- stale job IDs are rejected;
- cancellation is checked before backend selection, capture, OCR variants, and immediate clipboard copy;
- GNOME and Tesseract subprocesses are killed and reaped on cancellation;
- the main window is restored after success, cancellation, capture failure, OCR failure, and immediate-copy clipboard failure;
- frontend request tokens reject stale completion results;
- the UI has distinct `cancelling` and `cancelled` states.

Important limitation:

- the main window is hidden during the selector and OCR command, so the React Cancel button is not directly reachable during the normal hidden interval;
- reachable application cancellation still needs a tray command, global shortcut, or visible progress window;
- physical process-orphan verification remains open.

Detailed evidence is in `docs/CANCELLATION_STATUS_2026-08-02.md`.

## 6. Clipboard and editor behavior

Implemented properties:

- preview mode writes only after explicit user action;
- immediate mode writes after OCR only if the job is still active and not cancelled;
- empty and oversized clipboard writes are rejected;
- code whitespace is preserved;
- a cancelled replacement capture does not overwrite prior editor text;
- capture/editor actions are disabled while a capture job is busy;
- clipboard failure no longer prevents window restoration.

Still open:

- physical clipboard ownership and paste validation;
- notification behavior after copy;
- stronger large-text UI responsiveness tests;
- explicit preservation/recovery UX for immediate-copy clipboard failure.

## 7. Settings, diagnostics, and lifecycle

Implemented:

- versioned settings schema;
- default settings fallback for invalid settings;
- capture backend preference;
- text mode, preview policy, whitespace, notification, start-at-login, close-to-tray, and shortcut fields;
- startup capture intent consumed once;
- safe diagnostics DTO with application, OS, desktop, session, clipboard, tray, and schema fields;
- bounded XDG Screenshot Portal version and area-target capability summary;
- GNOME helper and Tesseract availability plus installed OCR language codes;
- privacy-safe in-memory retention of the most recent stable public error code;
- monotonic cleanup-failure counting without paths;
- tests that reject executable paths from serialized diagnostics.

Still open:

- corrupt-settings warning surfaced to the UI;
- stronger settings migration and interrupted-write tests;
- GNOME screenshot helper version reporting;
- Tesseract version reporting;
- last completed stage duration;
- **Copy diagnostics** with guaranteed redaction;
- a complete deterministic diagnostics-report snapshot test;
- real tray and close-to-tray behavior;
- single-instance behavior;
- reachable global shortcut behavior;
- start-at-login implementation or removal of that setting from v0.1.

Detailed diagnostics evidence is in `docs/DIAGNOSTICS_STATUS_2026-08-02.md`.

## 8. Debian package evidence

Accepted package built from `2fdce81e5163c5234515fc26efeea48544077c88`:

```text
PACKAGE_FILE=Screenshot OCR_0.1.0_amd64.deb
PACKAGE_NAME=screenshot-ocr
PACKAGE_VERSION=0.1.0
ARCHITECTURE=amd64
PACKAGE_SIZE_BYTES=5826646
INSTALLED_SIZE_KIB=16898
PACKAGE_SHA256=b079ee4bd5a2ea64c91b51248105c15cf6cc3da14a4b2f643f0f664f29ecf180
```

Artifact:

```text
ARTIFACT_ID=8828666725
ARTIFACT_NAME=screenshot-ocr-deb-2fdce81e5163c5234515fc26efeea48544077c88
ARTIFACT_ARCHIVE_SIZE_BYTES=5794424
ARTIFACT_ARCHIVE_SHA256=971cf67e5b70bf0e47fc7a54454507e74955d3e6ad0bb5c0291c488174680351
ARTIFACT_EXPIRES=2026-10-31T03:58:59Z
```

The package declares:

```text
gnome-screenshot
tesseract-ocr
tesseract-ocr-eng
libwebkit2gtk-4.1-0
libgtk-3-0
```

## 9. Physical validation still mandatory

Hosted CI cannot establish release readiness for:

- Ubuntu 22.04 GNOME Wayland and X11 capture;
- Ubuntu 24.04 GNOME Wayland and X11 capture;
- real portal capability discovery and area selection;
- permission denial and portal cancellation;
- multiple monitors and fractional scaling;
- selector dismissal and process-orphan behavior;
- OCR cancellation through a reachable application control;
- real clipboard ownership and paste behavior;
- real diagnostics values and redaction on a GNOME/Wayland session;
- dark/light terminal OCR quality;
- launcher, icon, dependency installation, upgrade, and uninstall behavior;
- tray, shortcut, single-instance, and start-at-login behavior.

No physical-validation checkbox should be marked complete from hosted compilation alone.

## 10. Highest-priority next product work

1. Make cancellation reachable while the main window is hidden, preferably through a tray action or global shortcut integrated with single-instance behavior.
2. Surface corrupt-settings recovery instead of silently presenting defaults.
3. Add deterministic OCR integration fixtures and accuracy evidence.
4. Complete missing diagnostics fields and **Copy diagnostics**.
5. Prepare and execute the Ubuntu 24.04 GNOME Wayland physical validation matrix.
6. Update the authoritative TODO only where the required evidence exists.

## 11. Quality-policy compliance

- Development remains on `master`.
- No branch or pull request was created.
- No first-party lint or Clippy warning was suppressed or downgraded.
- Permanent CI is read-only.
- No temporary validation workflow remains in the repository.
- The accepted package is tied to one exact source SHA and workflow run.
- Documentation distinguishes hosted proof from physical desktop proof.
