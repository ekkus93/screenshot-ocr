# Screenshot OCR v0.1 Implementation TODO

**Repository:** `ekkus93/screenshot-ocr`  
**Companion specification:** `docs/SCREENSHOT_OCR_V0_1_SPEC.md`  
**Document status:** Initial implementation plan  
**Date:** 2026-08-01  
**Target release:** v0.1

## 1. Purpose

This file is the authoritative implementation checklist for Screenshot OCR v0.1. It converts the product and technical specification into ordered, testable work.

The core release goal is:

> On Ubuntu 22.04 or 24.04, under GNOME Wayland or X11, a user can invoke Screenshot OCR, select visible terminal text, recognize it locally with Tesseract, optionally review it, and copy it to the clipboard without retaining or uploading the screenshot or recognized text.

## 2. Task-state rules

Use these checkbox states consistently:

- `[ ]` Not started or not proven.
- `[x]` Implemented and proven by the required evidence.

Do not mark a parent task complete merely because code exists. A task is complete only when its implementation, tests, documentation, and required validation evidence are complete.

When a task is blocked, add a dated note directly beneath it with:

- blocker;
- evidence;
- owner or required environment;
- exact next action.

Do not delete incomplete tasks to make a milestone appear finished.

## 3. Global definition of done

Every implementation task must satisfy all applicable items:

- [ ] Behavior conforms to `SCREENSHOT_OCR_V0_1_SPEC.md`.
- [ ] Failure paths are explicit and typed.
- [ ] No captured pixels, OCR text, clipboard text, or sensitive temporary paths are logged.
- [ ] Cleanup executes on success, cancellation, timeout, and failure.
- [ ] Unit or integration tests cover the behavior.
- [ ] User-facing behavior has accessible UI states.
- [ ] Documentation is updated when behavior or setup changes.
- [ ] Rust formatting, Clippy, and tests pass.
- [ ] Frontend typecheck, lint, tests, and production build pass.
- [ ] No new P0 or P1 issue is introduced.

## 4. Milestone dependency map

```text
M0 Decisions and repository policy
 └─ M1 Project scaffolding
     ├─ M2 Domain models, errors, and state machine
     ├─ M3 Settings and diagnostics foundation
     └─ M4 Frontend shell and design system

M2 + M3
 └─ M5 Capture capability detection
     ├─ M6 GNOME screenshot backend
     └─ M7 Portal area-capture backend

M2
 ├─ M8 Image preprocessing
 ├─ M9 Tesseract OCR
 └─ M10 Clipboard service

M4 + M6/M7 + M8 + M9 + M10
 └─ M11 End-to-end capture orchestration and UI
     ├─ M12 Shortcut, single-instance, tray, and autostart
     └─ M13 Privacy, cleanup, and security hardening

M11 + M12 + M13
 ├─ M14 Automated test and CI completion
 ├─ M15 Ubuntu platform validation
 └─ M16 Packaging, documentation, and release signoff
```

## 5. M0 — Repository baseline and explicit decisions

### M0.1 Repository governance

- [ ] Add a root `README.md` with the product summary, current status, supported platforms, development prerequisites, and links to the spec and TODO.
- [ ] Select and add a project license.
- [ ] Add `.gitignore` entries for Node, Vite, Rust, Tauri, IDE, test-output, package-output, screenshots, OCR fixtures generated locally, and temporary capture artifacts.
- [ ] Add `.editorconfig`.
- [ ] Add contribution guidance in `CONTRIBUTING.md`.
- [ ] Add a security policy stating how to report screenshot leakage, command execution, path traversal, and clipboard defects.
- [ ] Add a code of conduct if the repository will accept public contributions.
- [ ] Define a semantic versioning policy for application and configuration schema versions.
- [ ] Decide whether `master` remains the default branch or migrate intentionally to `main` before development begins.
- [ ] Configure branch protection after initial CI exists.

### M0.2 Product decisions

- [ ] Confirm the production application name and binary name.
  - Proposed display name: `Screenshot OCR`.
  - Proposed binary: `screenshot-ocr`.
- [ ] Confirm the application identifier, for example `io.github.ekkus93.screenshot-ocr` or another stable reverse-domain identifier.
- [ ] Confirm English as the only required bundled/declared OCR language for v0.1.
- [ ] Confirm preview-before-copy as the default.
- [ ] Confirm `Super+Shift+O` as the documented default shortcut.
- [ ] Confirm that persistent history is omitted or disabled in v0.1.
- [ ] Confirm `.deb` as the mandatory release package.
- [ ] Record whether AppImage is deferred or included only as an experimental artifact.
- [ ] Decide whether Tesseract integration uses a library binding or a subprocess after a time-boxed prototype.
- [ ] Record the decision and rationale in an architecture decision record under `docs/adr/`.

### M0.3 Threat model

- [ ] Create `docs/THREAT_MODEL.md`.
- [ ] Identify protected assets:
  - screenshot pixels;
  - recognized text;
  - clipboard contents;
  - terminal secrets;
  - filesystem integrity;
  - command execution boundary;
  - settings integrity.
- [ ] Model threats from malicious screen content.
- [ ] Model threats from malicious or replaced `gnome-screenshot`/`tesseract` executables.
- [ ] Model temporary-file races, symlinks, path traversal, and stale artifact recovery.
- [ ] Model untrusted frontend-to-Rust command payloads.
- [ ] Model oversized or malformed images.
- [ ] Model subprocess hangs and output flooding.
- [ ] Model accidental content disclosure through logs, panic messages, diagnostics, notifications, crash reports, and test artifacts.
- [ ] Define mitigations and map them to TODO task identifiers.

### M0 acceptance gate

- [ ] Product, license, identifier, history, OCR integration, and packaging decisions are recorded.
- [ ] Threat model exists and has no unowned P0/P1 mitigation.
- [ ] Root documentation clearly labels the project as pre-release.

## 6. M1 — Tauri, React, TypeScript, and Tailwind scaffolding

### M1.1 Toolchain pinning

- [ ] Add `rust-toolchain.toml` with a pinned stable toolchain and required components.
- [ ] Select Node package manager and commit its lockfile.
- [ ] Pin a supported Node major version in `.nvmrc`, `.node-version`, Volta metadata, or an equivalent repository policy.
- [ ] Record Ubuntu 22.04 development packages required by Tauri 2.
- [ ] Add a script that checks required development tools without modifying the host.
- [ ] Ensure all setup documentation distinguishes build dependencies from runtime dependencies.

### M1.2 Application scaffold

- [ ] Initialize Tauri 2 with React, TypeScript, and Vite.
- [ ] Enable TypeScript strict mode.
- [ ] Configure Tailwind CSS.
- [ ] Remove starter/demo components and assets.
- [ ] Add production application metadata and placeholder icons.
- [ ] Add a root frontend error boundary.
- [ ] Add a Rust library entry point so core logic is testable independently of the Tauri executable.
- [ ] Keep Tauri command registration in a thin adapter module.
- [ ] Add a minimal application launch smoke test.

### M1.3 Formatting and linting

- [ ] Configure Rust formatting.
- [ ] Configure Clippy with warnings denied in CI.
- [ ] Configure frontend formatting.
- [ ] Configure ESLint for React and TypeScript.
- [ ] Reject unused variables, unsafe `any`, and floating promises unless explicitly justified.
- [ ] Add repository scripts for `format`, `format:check`, `lint`, `typecheck`, `test`, and `build`.
- [ ] Add a combined local validation script that fails on the first failed mandatory gate.

### M1.4 Initial Tauri security configuration

- [ ] Define a restrictive Content Security Policy.
- [ ] Use only bundled local frontend assets.
- [ ] Disable or omit unrestricted shell access.
- [ ] Create minimal Tauri capabilities rather than broad defaults.
- [ ] Document every enabled Tauri plugin and permission.
- [ ] Add a test or review check that prevents accidental wildcard capability expansion.

### M1 acceptance gate

- [ ] Clean clone builds on Ubuntu 22.04.
- [ ] Development window launches.
- [ ] Production frontend and Tauri build complete.
- [ ] Formatting, linting, typecheck, and starter tests pass.
- [ ] No broad shell or filesystem capability is enabled.

## 7. M2 — Rust domain models, error taxonomy, and state machine

### M2.1 Core models

- [ ] Implement strongly typed identifiers for capture jobs.
- [ ] Implement `CaptureRequest`.
- [ ] Implement `CapturedImage` without serializable image bytes or source paths.
- [ ] Implement `OcrOptions`.
- [ ] Implement `OcrCandidate`.
- [ ] Implement `OcrResult`.
- [ ] Implement capture backend and OCR engine identifiers.
- [ ] Implement `TextMode`, `CopyPolicy`, `CaptureSource`, and language selection enums.
- [ ] Implement warning types for low confidence, suspicious text, fallback backend, and cleanup concerns.
- [ ] Add serialization DTOs separate from internal domain types.

### M2.2 Error taxonomy

- [ ] Implement internal typed errors for capture, image processing, OCR, clipboard, settings, desktop integration, cleanup, and orchestration.
- [ ] Implement stable public error codes listed in the specification.
- [ ] Implement safe error-to-DTO mapping.
- [ ] Ensure source errors and raw subprocess output are not automatically serialized to the frontend.
- [ ] Add redaction tests for paths, OCR strings, command content, and image metadata.
- [ ] Provide actionable user messages for every expected public error.
- [ ] Preserve an internal error chain for local debugging without content-bearing fields.

### M2.3 State machine

- [ ] Implement the authoritative capture job state machine in Rust.
- [ ] Reject invalid transitions.
- [ ] Associate all events and results with a job identifier.
- [ ] Implement cancellation tokens.
- [ ] Distinguish user cancellation from failure.
- [ ] Prevent two capture jobs from simultaneously owning a selector.
- [ ] Define behavior for a second capture request while busy.
- [ ] Define and implement stale frontend event rejection.
- [ ] Add exhaustive transition tests.
- [ ] Add race tests for start/cancel/start sequences.

### M2.4 Application service boundary

- [ ] Define the orchestration service API independently of Tauri.
- [ ] Inject capture, OCR, clipboard, settings, notification, and diagnostics dependencies.
- [ ] Provide test doubles for each dependency.
- [ ] Keep process spawning and platform detection outside domain logic.
- [ ] Add an integration harness that runs orchestration entirely with fakes.

### M2 acceptance gate

- [ ] Domain crate/module compiles without the React frontend.
- [ ] State machine and error-redaction tests pass.
- [ ] A fake end-to-end job can transition from Idle to Copied and from Idle to Cancelled.

## 8. M3 — Settings, diagnostics, and logging foundation

### M3.1 Settings model

- [ ] Implement schema version 1 settings.
- [ ] Implement defaults from the specification.
- [ ] Validate maximum image dimensions and decoded-pixel limits against hard bounds.
- [ ] Validate language and mode enum values.
- [ ] Prevent frontend requests from supplying executable paths or raw OCR arguments.
- [ ] Implement atomic writes using a temporary settings file plus rename where supported.
- [ ] Apply restrictive file permissions.
- [ ] Detect corrupt settings.
- [ ] Preserve or quarantine corrupt settings with a visible warning rather than silently discarding them.
- [ ] Implement reset-to-defaults.
- [ ] Add migration framework even though only schema v1 initially exists.
- [ ] Test interrupted write recovery.
- [ ] Test unknown fields and unknown enum values.

### M3.2 Logging

- [ ] Configure `tracing` with release-appropriate levels.
- [ ] Define a content-free structured logging schema.
- [ ] Add field wrappers or lint/review conventions for sensitive values.
- [ ] Ensure job identifiers are random and not derived from captured content.
- [ ] Log stage durations and safe codes only.
- [ ] Do not log process argument values that contain temporary paths.
- [ ] Do not log Tesseract stdout.
- [ ] Sanitize stderr classifications before logging.
- [ ] Add automated tests that scan captured test logs for known secret fixtures.

### M3.3 Diagnostics model

- [x] Implement safe diagnostics DTO.
- [x] Include application version.
- [x] Include OS release, desktop environment, and session type.
- [x] Include portal presence and safe capability summary.
- [ ] Include GNOME screenshot helper presence/version.
- [ ] Include Tesseract presence/version and installed language codes.
- [x] Include clipboard and tray status.
- [x] Include settings schema version.
- [ ] Include last safe error code and stage duration.
- [x] Include cleanup failure counts without paths.
- [ ] Implement **Copy diagnostics** with guaranteed redaction.
- [ ] Add snapshot tests for the diagnostics report.

  - Evidence (2026-08-02): source `2fdce81e5163c5234515fc26efeea48544077c88`; CI run `30731505719`; `docs/DIAGNOSTICS_STATUS_2026-08-02.md`.
  - Combined presence/version and error-code/stage-duration items remain unchecked because helper versions and stage duration are not implemented.

### M3 acceptance gate

- [ ] Settings persist atomically and recover visibly from corruption.
- [ ] Diagnostics are useful without exposing captured content or temporary paths.
- [ ] Logging redaction tests pass.

## 9. M4 — Frontend shell, navigation, and design system

### M4.1 Application shell

- [ ] Implement responsive application layout.
- [ ] Add Capture navigation.
- [ ] Add Settings navigation.
- [ ] Add Diagnostics navigation or settings subsection.
- [ ] Omit History or display an honest disabled/no-history explanation.
- [ ] Do not ship mock OCR history entries.
- [ ] Add frontend route/state restoration that contains no recognized text beyond the active session.

### M4.2 Capture screen static states

- [ ] Implement Idle state.
- [ ] Implement Preparing state.
- [ ] Implement Selecting region state.
- [ ] Implement Processing state with stage text.
- [ ] Implement Reviewing state.
- [ ] Implement Copied confirmation.
- [ ] Implement Cancelled state without alarm styling.
- [ ] Implement recoverable Error state.
- [ ] Implement low-confidence warning.
- [ ] Implement empty-result warning that explicitly says clipboard was unchanged.
- [ ] Implement clipboard-failure state retaining editable OCR text.

### M4.3 Preview editor

- [ ] Use a monospace font stack.
- [ ] Preserve line breaks and spaces in the editor.
- [ ] Add **Copy text**.
- [ ] Add **Capture again**.
- [ ] Add **Clear**.
- [ ] Prevent accidental loss of edited text during an unrelated settings refresh.
- [ ] Decide whether a new capture replaces current text immediately or only after capture succeeds.
- [ ] Preserve prior text after capture cancellation.
- [ ] Add text-length handling that remains responsive for unusually large OCR results.

### M4.4 Settings UI

- [ ] Add OCR language selection.
- [ ] Add text-mode selection.
- [ ] Add preview-before-copy toggle.
- [ ] Add preserve-whitespace control with clear terminal-mode semantics.
- [ ] Add notification toggle.
- [ ] Add start-at-login toggle.
- [ ] Add close-to-tray toggle where supported.
- [ ] Add capture backend display with `Auto` as the default.
- [ ] Add default shortcut display.
- [ ] Add GNOME custom-shortcut instructions placeholder.
- [ ] Add reset settings action with confirmation.
- [ ] Show save failures without losing unsaved values.

### M4.5 Accessibility

- [ ] Use native controls where possible.
- [ ] Associate every label and form control.
- [ ] Provide visible keyboard focus.
- [ ] Add ARIA live regions for capture state and copy result.
- [ ] Avoid color-only status communication.
- [ ] Validate keyboard-only operation.
- [ ] Validate screen scaling up to 200%.
- [ ] Validate narrow window width.
- [ ] Run automated accessibility checks on primary states.
- [ ] Perform a manual screen-reader smoke test where practical.

### M4.6 Frontend tests

- [ ] Test all capture states.
- [ ] Test busy-state button disabling.
- [ ] Test job-ID stale event rejection.
- [ ] Test preview editing and copy command payload.
- [ ] Test empty-result behavior.
- [ ] Test settings validation and failed save.
- [ ] Test diagnostics rendering.
- [ ] Test keyboard navigation of primary workflow.

### M4 acceptance gate

- [ ] The mockup workflow exists as a functional frontend using fake backend events.
- [ ] Primary frontend tests and accessibility checks pass.
- [ ] The production UI contains no fake history or fake successful diagnostics.

## 10. M5 — Runtime environment and capture capability detection

### M5.1 Environment detection

- [ ] Detect Linux and reject unsupported operating systems with a stable error.
- [ ] Read Ubuntu release information from safe system sources.
- [ ] Detect GNOME from standard environment variables and corroborating evidence.
- [ ] Detect Wayland versus X11.
- [ ] Handle missing or contradictory environment variables.
- [ ] Keep detection results content-free and safe for diagnostics.
- [ ] Add fixtures for Ubuntu 22.04/24.04, Wayland/X11, unknown desktop, and malformed environment data.

### M5.2 Executable discovery

- [ ] Locate `gnome-screenshot` without invoking a shell.
- [ ] Locate `tesseract` if subprocess integration is selected.
- [ ] Reject untrusted per-request executable paths.
- [ ] Define trust rules for discovered executable locations.
- [ ] Record safe version information.
- [ ] Provide installation guidance when required executables are absent.

### M5.3 Portal probing

- [ ] Detect whether the XDG Screenshot Portal is reachable.
- [ ] Query screenshot interface version.
- [ ] Query advertised targets when the interface supports them.
- [ ] Detect explicit area-target capability.
- [ ] Ensure probing does not show a capture dialog.
- [ ] Bound D-Bus calls with timeouts.
- [ ] Handle absent, old, partially implemented, and failing portal backends.
- [ ] Add mocked portal capability tests.

### M5.4 Backend selection policy

- [ ] Implement `auto` selection.
- [ ] Prefer portal area capture only when advertised and validated.
- [ ] Fall back to GNOME helper on supported Ubuntu GNOME environments.
- [ ] Return `unsupported_environment` when no safe backend is available.
- [ ] Attach safe fallback reason to diagnostics and OCR result warnings.
- [ ] Never silently use an unvalidated backend.
- [ ] Add table-driven tests covering the full platform matrix and failure combinations.

### M5 acceptance gate

- [ ] Capability probing produces correct backend selection in tests.
- [ ] Probe operations are noninteractive and bounded.
- [ ] Unsupported environments produce corrective guidance.

## 11. M6 — GNOME screenshot compatibility backend

### M6.1 Temporary capture directory

- [ ] Create an application-owned temporary/runtime directory.
- [ ] Apply restrictive permissions.
- [ ] Generate unpredictable owned filenames.
- [ ] Define a recognizable ownership marker that does not contain user content.
- [ ] Reject symlink targets.
- [ ] Reject existing non-owned files.
- [ ] Enforce expected regular-file type after capture.
- [ ] Implement cleanup guard.
- [ ] Implement safe stale-artifact scavenging limited to application-owned files.
- [ ] Test symlink and race scenarios.

### M6.2 Process invocation

- [ ] Spawn `gnome-screenshot` with an argument vector, never a shell.
- [ ] Use `--area` and `--file`/`-f` correctly on Ubuntu 22.04 and 24.04.
- [ ] Minimize inherited environment where practical without breaking GNOME integration.
- [ ] Hide the Screenshot OCR window before selection.
- [ ] Restore window state correctly after success, cancellation, and failure.
- [ ] Capture exit status.
- [ ] Bound process lifetime.
- [ ] Implement cancellation behavior without leaving an orphan helper.
- [ ] Distinguish user cancellation from helper failure using tested evidence.
- [ ] Avoid returning raw helper stderr to the UI.

### M6.3 Result validation

- [ ] Require a successful, decodable image file.
- [ ] Reject zero-byte files.
- [ ] Reject unsupported formats.
- [ ] Enforce byte-size and decoded-pixel limits.
- [ ] Load pixels into application-controlled memory.
- [ ] Delete the source file immediately after decode.
- [ ] Return cleanup evidence without returning the path.
- [ ] Treat cleanup failure as visible diagnostic/error according to severity policy.

### M6.4 Backend tests

- [ ] Unit-test command construction.
- [ ] Test successful fake-helper execution.
- [ ] Test cancellation.
- [ ] Test nonzero exit.
- [ ] Test missing output.
- [ ] Test malformed image.
- [ ] Test oversized image.
- [ ] Test cleanup after decode failure.
- [ ] Test cleanup after downstream OCR failure through orchestration integration.
- [ ] Perform manual capture on all four required Ubuntu/session combinations.

### M6 acceptance gate

- [ ] GNOME region selection works on Ubuntu 22.04 Wayland/X11 and Ubuntu 24.04 Wayland/X11.
- [ ] No known path leaves an owned temporary capture behind without a visible cleanup warning.
- [ ] No shell invocation exists.

## 12. M7 — XDG Screenshot Portal area backend

### M7.1 D-Bus client

- [ ] Select a maintained Rust D-Bus/portal integration approach.
- [ ] Implement screenshot request object-path/token management.
- [ ] Query version and available target data.
- [ ] Request target `Area` only when advertised.
- [ ] Set appropriate parent-window information when available.
- [ ] Handle response signals and request completion.
- [ ] Implement timeout and cancellation.
- [ ] Close/cancel the portal request when appropriate.

### M7.2 Returned URI handling

- [ ] Validate portal response code.
- [ ] Distinguish user dismissal from permission denial and backend failure.
- [ ] Accept only supported URI schemes.
- [ ] Decode or safely copy the portal result into application-controlled memory.
- [ ] Enforce image limits.
- [ ] Avoid logging URI/path values.
- [ ] Release temporary/document portal resources where supported.
- [ ] Guarantee no raw URI is serialized to the frontend.

### M7.3 Fallback behavior

- [ ] If area target is unavailable during probe, select GNOME backend before capture.
- [ ] If portal capability changes between probe and capture, fail explicitly or perform one policy-approved fallback.
- [ ] Do not display two sequential selectors without telling the user.
- [ ] Record safe fallback reason.
- [ ] Add tests for old portal, unavailable area target, request failure, cancellation, and malformed result.

### M7.4 Manual validation

- [ ] Validate portal behavior on a system that advertises explicit area target.
- [ ] Validate fallback on Ubuntu 22.04.
- [ ] Validate that portal permission UI and cancellation are understandable.
- [ ] Validate multiple monitors and fractional scaling.

### M7 acceptance gate

- [ ] Portal backend is used only when capability is proven.
- [ ] Cancellation and permission failures are typed.
- [ ] Fallback cannot cause duplicate unexpected selectors.

## 13. M8 — Image decoding and preprocessing pipeline

### M8.1 Safe decoding

- [ ] Select maintained Rust image library/libraries.
- [ ] Decode into a normalized pixel representation.
- [ ] Reject unsupported or malformed input.
- [ ] Enforce encoded-byte limit before decode.
- [ ] Enforce width, height, and pixel-count limits.
- [ ] Avoid unbounded allocation from attacker-controlled dimensions.
- [ ] Strip metadata not required for OCR.
- [ ] Add malformed-image corpus tests.

### M8.2 Image analysis

- [ ] Compute luminance statistics.
- [ ] Detect likely light-on-dark versus dark-on-light text.
- [ ] Estimate glyph scale or image conditions that warrant upscaling.
- [ ] Record only aggregate, content-free metrics.
- [ ] Add deterministic tests for analysis decisions.

### M8.3 Preprocessing variants

- [ ] Implement original normalized variant.
- [ ] Implement grayscale contrast-normalized variant.
- [ ] Implement inverted grayscale variant.
- [ ] Implement 2× high-quality upscale.
- [ ] Implement 3× upscale for small text.
- [ ] Evaluate and implement thresholding only if fixture evidence shows benefit.
- [ ] Bound total variant count and memory use.
- [ ] Avoid writing intermediate variants to disk in production.
- [ ] Add benchmark coverage for common image sizes.

### M8.4 Variant policy

- [ ] Implement fast path.
- [ ] Define confidence/structure threshold for trying additional variants.
- [ ] Define maximum OCR attempts per capture.
- [ ] Make policy deterministic under tests.
- [ ] Expose selected variant in safe result metadata.
- [ ] Add a debug-only fixture tool that never accepts or stores real user captures by default.

### M8.5 Fixtures

- [ ] Add synthetic dark terminal fixture.
- [ ] Add synthetic light terminal fixture.
- [ ] Add small-font fixture.
- [ ] Add antialiased/fractional-scaling fixture.
- [ ] Add Rust error fixture.
- [ ] Add shell command fixture.
- [ ] Add JSON/TOML/YAML/Markdown fixture set.
- [ ] Add punctuation ambiguity fixture containing `0 O 1 l I | - _ : ; ' \` { } [ ] ( ) / \\`.
- [ ] Document fixture generation so no proprietary or private screenshot is committed.

### M8 acceptance gate

- [ ] All variants are bounded and deterministic.
- [ ] Dark and light terminal fixtures improve or preserve OCR accuracy relative to original-only baseline.
- [ ] Production pipeline leaves no intermediate images on disk.

## 14. M9 — Tesseract OCR engine

### M9.1 Integration prototype and decision

- [ ] Prototype direct library binding.
- [ ] Prototype subprocess integration if still under consideration.
- [ ] Compare Ubuntu 22.04/24.04 packaging, cancellation, timeout, thread safety, and error behavior.
- [ ] Record the selected integration in an ADR.
- [ ] Remove abandoned prototype code and dependencies.

### M9.2 Capability probing

- [ ] Detect Tesseract availability.
- [ ] Detect safe version information.
- [ ] Detect English language data.
- [ ] Enumerate installed language codes safely.
- [ ] Avoid triggering OCR during probe.
- [ ] Provide exact corrective instructions when English data is missing.
- [ ] Add tests for absent engine, broken engine, missing language, and unsupported version policy.

### M9.3 OCR execution

- [ ] Implement terminal-mode Tesseract configuration.
- [ ] Implement normal-document mode.
- [ ] Implement single-line mode.
- [ ] Use bounded timeout.
- [ ] Support cancellation where technically possible.
- [ ] Prevent concurrent unsafe access if the selected binding is not thread-safe.
- [ ] Capture confidence data where available.
- [ ] Return typed engine failures.
- [ ] Never log OCR output.
- [ ] Never include OCR text in panic messages.

### M9.4 Text preservation and cleanup

- [ ] Normalize line endings to `\n`.
- [ ] Preserve leading spaces.
- [ ] Preserve blank lines.
- [ ] Preserve repeated interior spaces where returned.
- [ ] Avoid smart-quote conversion.
- [ ] Avoid dash substitution.
- [ ] Avoid spelling correction.
- [ ] Define trailing-space policy.
- [ ] Ensure cleanup is idempotent.
- [ ] Add exact tests for developer punctuation.
- [ ] Add tests proving secrets in OCR fixtures do not enter logs.

### M9.5 Candidate scoring

- [ ] Implement confidence contribution.
- [ ] Penalize replacement/control characters.
- [ ] Penalize implausibly empty results.
- [ ] Detect pathological unbroken output.
- [ ] Preserve a reasonable punctuation ratio for terminal mode.
- [ ] Select the best bounded candidate deterministically.
- [ ] Report low confidence as a warning, not silent failure.
- [ ] Document why no language model or cloud correction is used.

### M9.6 OCR regression suite

- [ ] Establish baseline expected outputs for all fixtures.
- [ ] Separate exact assertions from tolerant structural assertions.
- [ ] Record Tesseract version in test evidence.
- [ ] Add regression tests for every fixed OCR fidelity defect.
- [ ] Add performance measurements for representative fixtures.

### M9 acceptance gate

- [ ] English Tesseract OCR works on both Ubuntu releases.
- [ ] Terminal fixtures preserve required whitespace and punctuation within documented tolerances.
- [ ] OCR timeout/failure never overwrites clipboard or leaves images behind.

## 15. M10 — Clipboard service

### M10.1 Backend implementation

- [ ] Select Tauri clipboard plugin or Rust clipboard implementation based on Wayland/X11 reliability tests.
- [ ] Implement `ClipboardService` abstraction.
- [ ] Write UTF-8 plain text.
- [ ] Return explicit failure.
- [ ] Do not read unrelated clipboard contents.
- [ ] Ensure clipboard ownership/lifetime behavior is valid after capture completion.
- [ ] Test multiline text, leading spaces, large text, and non-ASCII text.

### M10.2 Clipboard policy

- [ ] Do not write on cancellation.
- [ ] Do not write on OCR failure.
- [ ] Do not write whitespace-only output.
- [ ] In preview mode, write only on explicit user action.
- [ ] In immediate-copy mode, write the exact returned result.
- [ ] On clipboard failure, preserve OCR text in the UI.
- [ ] Add retry action.
- [ ] Add tests proving prior clipboard remains unchanged on all failure paths where test infrastructure permits.

### M10.3 Manual validation

- [ ] Paste into GNOME Terminal.
- [ ] Paste into a browser text field.
- [ ] Paste into a graphical text editor.
- [ ] Validate on Wayland and X11 for Ubuntu 22.04 and 24.04.
- [ ] Validate clipboard content remains after the capture helper exits.
- [ ] Validate application quit behavior according to chosen clipboard backend semantics.

### M10 acceptance gate

- [ ] Clipboard writes are reliable across the required platform matrix.
- [ ] Empty/error/cancel paths do not mutate clipboard.

## 16. M11 — End-to-end capture orchestration and functional UI

### M11.1 Tauri command layer

- [ ] Implement typed `start_capture` command.
- [ ] Implement typed `cancel_capture` command.
- [ ] Implement typed `copy_text` command.
- [ ] Implement `get_settings` and `update_settings`.
- [ ] Implement `get_diagnostics`.
- [ ] Validate all frontend payloads in Rust.
- [ ] Reject arbitrary executable paths, command arguments, file paths, and OCR config.
- [ ] Keep commands thin and delegate to application services.
- [ ] Define stable event names and DTOs.

### M11.2 Orchestration pipeline

- [ ] Probe or load cached safe capabilities.
- [ ] Choose capture backend.
- [ ] Hide window if appropriate.
- [ ] Launch selector.
- [ ] Handle cancellation.
- [ ] Decode and validate image.
- [ ] Delete source capture immediately after decode.
- [ ] Analyze and preprocess.
- [ ] Run bounded OCR candidate policy.
- [ ] Apply terminal/document cleanup.
- [ ] Evaluate preview/copy policy.
- [ ] Write clipboard only when allowed.
- [ ] Notify only when allowed.
- [ ] Restore/focus window according to workflow.
- [ ] Emit final result and safe metrics.
- [ ] Release job ownership in all exit paths.

### M11.3 Frontend integration

- [ ] Connect Capture button to Rust command.
- [ ] Connect state events to UI using job IDs.
- [ ] Ignore stale events.
- [ ] Show selected backend and fallback warning when relevant.
- [ ] Show confidence and warnings.
- [ ] Preserve recognized whitespace in editor.
- [ ] Connect manual copy action.
- [ ] Connect retry and capture-again actions.
- [ ] Preserve prior preview text on cancellation.
- [ ] Ensure immediate-copy mode can operate without showing the main window.
- [ ] Show background failures by notification plus recoverable app state.

### M11.4 End-to-end tests with fakes

- [ ] Happy path with preview.
- [ ] Happy path with immediate copy.
- [ ] User cancellation.
- [ ] Capture backend failure.
- [ ] Invalid image.
- [ ] OCR timeout.
- [ ] OCR empty result.
- [ ] Clipboard failure.
- [ ] Notification failure without core operation failure.
- [ ] Settings load failure.
- [ ] Two simultaneous capture requests.
- [ ] Stale event arrival.
- [ ] Cleanup failure surfacing.

### M11 acceptance gate

- [ ] A user can complete the full workflow through the production UI.
- [ ] Every expected failure has a visible, recoverable state.
- [ ] Fake end-to-end suite covers all major stages.

## 17. M12 — Shortcut, single-instance, tray, and startup integration

### M12.1 Command-line capture entry point

- [ ] Implement `screenshot-ocr capture`.
- [ ] Implement `screenshot-ocr show` if useful.
- [ ] Implement `screenshot-ocr diagnostics` only if output is guaranteed redacted.
- [ ] Define exit codes.
- [ ] Ensure command-line help contains no unsupported promises.
- [ ] Ensure `capture` starts the app if no primary instance exists.
- [ ] Ensure `capture` signals the existing primary instance when one exists.

### M12.2 Single-instance routing

- [ ] Select and configure Tauri single-instance support.
- [ ] Route second-instance arguments to the authoritative application service.
- [ ] Authenticate or constrain local IPC as appropriate.
- [ ] Prevent malformed local messages from invoking arbitrary commands.
- [ ] Handle primary-instance startup races.
- [ ] Serialize repeated shortcut presses.
- [ ] Test rapid repeated invocation.
- [ ] Test invocation while preview is open.
- [ ] Test invocation while selector is active.

### M12.3 GNOME shortcut setup

- [ ] Display the exact installed capture command.
- [ ] Document Ubuntu 22.04 GNOME custom-shortcut setup.
- [ ] Document Ubuntu 24.04 GNOME custom-shortcut setup.
- [ ] Add a copy-command button.
- [ ] Detect and explain command-path changes after nonstandard installation where possible.
- [ ] Do not edit GNOME settings automatically unless a separate reviewed task proves the operation safe and reversible.
- [ ] Validate `Super+Shift+O` does not conflict in clean test systems.
- [ ] Document conflict resolution.

### M12.4 Direct shortcut registration

- [ ] Evaluate Tauri global shortcut behavior on X11.
- [ ] Evaluate behavior on supported Wayland sessions.
- [ ] Enable only where proven reliable.
- [ ] Treat GNOME custom shortcut invocation as the required compatibility path.
- [ ] Avoid presenting direct registration as active when registration failed.

### M12.5 System tray

- [ ] Add tray icon.
- [ ] Add **Capture text**.
- [ ] Add **Show Screenshot OCR**.
- [ ] Add **Settings**.
- [ ] Add **Quit**.
- [ ] Route tray capture through the same orchestration service.
- [ ] Detect tray unavailability.
- [ ] Keep main workflow usable without tray.
- [ ] Ensure quit performs cancellation and cleanup.
- [ ] Test close-to-tray behavior.
- [ ] Test explicit quit.

### M12.6 Start at login

- [ ] Add autostart integration.
- [ ] Default it off.
- [ ] Ensure enabling/disabling is reversible.
- [ ] Ensure autostart launches idle and does not trigger capture.
- [ ] Surface autostart failures.
- [ ] Test across both Ubuntu releases.

### M12 acceptance gate

- [ ] GNOME custom shortcut successfully triggers capture on all required environments.
- [ ] Rapid repeated invocations cannot launch overlapping selectors.
- [ ] Tray absence does not break the app.
- [ ] Quit and autostart behavior are correct and documented.

## 18. M13 — Privacy, cleanup, and security hardening

### M13.1 Content leakage audit

- [ ] Search code for logging of OCR text.
- [ ] Search code for image-byte debug formatting.
- [ ] Search code for temporary-path logging.
- [ ] Search panic and error messages for source content.
- [ ] Search frontend console logging.
- [ ] Disable production frontend debug logs containing job payloads.
- [ ] Review notifications for content leakage.
- [ ] Review diagnostics export.
- [ ] Review test failure snapshots and CI artifacts.
- [ ] Add automated forbidden-content test using a distinctive synthetic secret.

### M13.2 Cleanup fault injection

- [ ] Inject image decode failure after capture.
- [ ] Inject preprocessing failure.
- [ ] Inject OCR failure.
- [ ] Inject OCR timeout.
- [ ] Inject clipboard failure.
- [ ] Inject frontend disconnect.
- [ ] Inject application shutdown during each stage.
- [ ] Inject temporary-file deletion failure.
- [ ] Verify cleanup or visible recovery evidence for each case.
- [ ] Verify startup scavenger removes only owned stale artifacts.

### M13.3 Subprocess hardening

- [ ] Confirm no shell execution.
- [ ] Confirm fixed argument construction.
- [ ] Confirm executable discovery trust policy.
- [ ] Bound stdout/stderr collection.
- [ ] Bound runtime.
- [ ] Kill and reap timed-out child processes.
- [ ] Prevent orphan capture or OCR processes.
- [ ] Review inherited environment variables.
- [ ] Test paths containing spaces and unusual characters.

### M13.4 Input and resource limits

- [ ] Set maximum encoded image bytes.
- [ ] Set maximum dimensions.
- [ ] Set maximum decoded pixels.
- [ ] Set maximum OCR attempts.
- [ ] Set OCR timeout.
- [ ] Set maximum recognized text returned to frontend, with safe handling if exceeded.
- [ ] Set maximum diagnostics size.
- [ ] Test all limits and boundary values.

### M13.5 Tauri capability audit

- [ ] Enumerate every capability and plugin permission.
- [ ] Remove unused permissions.
- [ ] Verify no arbitrary filesystem access from frontend.
- [ ] Verify no arbitrary shell command access.
- [ ] Verify no remote content navigation.
- [ ] Verify CSP in production bundle.
- [ ] Verify updater is disabled or explicitly secured if not implemented.
- [ ] Review dependency advisories.

### M13.6 Privacy documentation

- [ ] Add `docs/PRIVACY.md`.
- [ ] State that OCR is local.
- [ ] State what temporary files may exist and when they are deleted.
- [ ] State that history is not retained in v0.1.
- [ ] State what diagnostics contain.
- [ ] State that clipboard contents are written only according to policy.
- [ ] Document known limitations and operating-system facilities involved.

### M13 acceptance gate

- [ ] Content-leakage audit finds no unresolved P0/P1 issue.
- [ ] Cleanup fault-injection matrix is complete.
- [ ] Tauri capability audit is documented.
- [ ] Privacy documentation matches implementation.

## 19. M14 — Automated testing and continuous integration

### M14.1 Rust tests

- [ ] Reach meaningful coverage of domain/state/error logic.
- [ ] Complete capture policy tests.
- [ ] Complete temporary-file tests.
- [ ] Complete image limit tests.
- [ ] Complete preprocessing tests.
- [ ] Complete OCR cleanup/scoring tests.
- [ ] Complete settings persistence/migration tests.
- [ ] Complete redaction tests.
- [ ] Complete orchestration integration tests.
- [ ] Ensure tests are deterministic and do not require the real desktop except explicitly marked manual/system tests.

### M14.2 Frontend tests

- [ ] Complete component tests for primary screens.
- [ ] Complete state-event tests.
- [ ] Complete settings tests.
- [ ] Complete accessibility tests.
- [ ] Complete error rendering tests.
- [ ] Ensure production build contains no test fixtures or mock history.

### M14.3 CI workflow

- [ ] Add frontend install with frozen lockfile.
- [ ] Add formatting check.
- [ ] Add TypeScript typecheck.
- [ ] Add ESLint.
- [ ] Add frontend tests.
- [ ] Add production frontend build.
- [ ] Add `cargo fmt --check`.
- [ ] Add Clippy with `-D warnings`.
- [ ] Add Rust tests.
- [ ] Add release Tauri build on Ubuntu 22.04.
- [ ] Cache dependencies without caching generated sensitive test outputs.
- [ ] Upload only reviewed, non-sensitive build artifacts.
- [ ] Set least-privilege workflow permissions.
- [ ] Pin third-party actions by immutable commit SHA.
- [ ] Add dependency review/security tooling appropriate to public repositories.

### M14.4 Packaging smoke CI

- [ ] Build `.deb`.
- [ ] Inspect package metadata and dependency list.
- [ ] Install package into a clean Ubuntu 22.04 CI/container/VM environment where feasible.
- [ ] Run `--help` and safe diagnostics smoke tests.
- [ ] Launch UI under virtual display where feasible.
- [ ] Do not claim Wayland region-capture validation from a headless CI test.
- [ ] Retain manual platform evidence as a separate release gate.

### M14.5 Quality gates

- [ ] No ignored failing test without linked issue and rationale.
- [ ] No broad `allow` annotations hiding Clippy defects.
- [ ] No snapshot update accepted without review.
- [ ] No CI path silently skips mandatory jobs.
- [ ] Document exact local commands equivalent to CI.

### M14 acceptance gate

- [ ] CI passes on a clean commit.
- [ ] Release build and `.deb` artifact are produced.
- [ ] Workflow permissions and action pins are reviewed.

## 20. M15 — Ubuntu 22.04/24.04 platform validation

### M15.1 Test environment records

For each environment, record:

- Ubuntu version and patch level;
- GNOME version;
- Wayland or X11;
- portal versions/backends;
- Tesseract version;
- display count, resolutions, and scaling;
- package commit SHA;
- selected capture backend.

Required environments:

- [ ] Ubuntu 22.04 GNOME Wayland.
- [ ] Ubuntu 22.04 GNOME X11.
- [ ] Ubuntu 24.04 GNOME Wayland.
- [ ] Ubuntu 24.04 GNOME X11.

### M15.2 Core workflow matrix

On every required environment:

- [ ] Install `.deb` on a clean system.
- [ ] Launch main window.
- [ ] Capture from main button.
- [ ] Cancel selection.
- [ ] Capture dark terminal text.
- [ ] Capture light terminal text.
- [ ] Review and edit result.
- [ ] Copy and paste into terminal.
- [ ] Copy and paste into browser.
- [ ] Enable immediate copy.
- [ ] Confirm clipboard unchanged after cancellation.
- [ ] Confirm clipboard unchanged after empty result.
- [ ] Trigger missing/failing OCR scenario where practical.
- [ ] Confirm prior text remains available after clipboard failure simulation.
- [ ] Capture repeatedly.
- [ ] Press shortcut repeatedly during active capture.
- [ ] Quit cleanly.
- [ ] Restart and verify settings.

### M15.3 Shortcut/tray/autostart matrix

- [ ] Configure GNOME custom shortcut.
- [ ] Trigger capture with `Super+Shift+O`.
- [ ] Trigger shortcut while app is closed.
- [ ] Trigger shortcut while app is running.
- [ ] Trigger shortcut while preview is open.
- [ ] Trigger shortcut while selector is active.
- [ ] Validate tray actions where supported.
- [ ] Validate close-to-tray.
- [ ] Validate explicit quit.
- [ ] Validate start-at-login enable/disable.

### M15.4 Display matrix

Across the available environments:

- [ ] Single monitor at 100%.
- [ ] Single monitor at 200%.
- [ ] Fractional scale 125%.
- [ ] Fractional scale 150%.
- [ ] Multiple monitors with equal scaling.
- [ ] Multiple monitors with mixed scaling if GNOME configuration supports it.
- [ ] Selection crossing no monitor boundary.
- [ ] Selection on secondary monitor.
- [ ] Small region.
- [ ] Large but allowed region.
- [ ] Oversized region/resource-limit behavior.

### M15.5 Privacy and cleanup inspection

On every required environment:

- [ ] Inspect application temp/runtime directories after success.
- [ ] Inspect after cancellation.
- [ ] Inspect after OCR failure.
- [ ] Inspect after forced app termination during OCR.
- [ ] Restart and verify scavenger behavior.
- [ ] Inspect logs for synthetic secret fixture.
- [ ] Copy diagnostics and inspect for paths/content.
- [ ] Confirm no network traffic is required for capture/OCR using an offline test.

### M15.6 Evidence files

- [ ] Create `docs/validation/UBUNTU_22_04_WAYLAND.md`.
- [ ] Create `docs/validation/UBUNTU_22_04_X11.md`.
- [ ] Create `docs/validation/UBUNTU_24_04_WAYLAND.md`.
- [ ] Create `docs/validation/UBUNTU_24_04_X11.md`.
- [ ] Include exact commit SHA, package checksum, commands, results, known limitations, and defect links.
- [ ] Do not include real private screenshots or OCR content in evidence.

### M15 acceptance gate

- [ ] All four required environments pass the mandatory workflow.
- [ ] Any platform-specific exception is explicitly approved in the specification; undocumented exceptions block release.
- [ ] No P0/P1 issue remains.

## 21. M16 — Packaging, user documentation, and release signoff

### M16.1 Debian package

- [ ] Configure application metadata.
- [ ] Configure desktop file.
- [ ] Configure icons at required sizes.
- [ ] Configure MIME/URL handlers only if genuinely needed; otherwise omit them.
- [ ] Declare validated runtime dependencies.
- [ ] Include AppIndicator dependency only when required by actual tray implementation.
- [ ] Include Tesseract and English trained-data dependencies according to chosen packaging strategy.
- [ ] Include GNOME screenshot dependency for supported fallback.
- [ ] Validate package install, upgrade, reinstall, and removal.
- [ ] Verify uninstall does not delete unrelated user data.
- [ ] Verify package does not leave executable temp helpers.
- [ ] Generate package checksum.

### M16.2 User documentation

- [ ] Expand root README with installation.
- [ ] Document Ubuntu 22.04 support.
- [ ] Document Ubuntu 24.04 support.
- [ ] Document Wayland and X11 behavior.
- [ ] Document GNOME custom shortcut setup.
- [ ] Document tray behavior and fallback.
- [ ] Document Tesseract/language dependencies.
- [ ] Document capture, preview, immediate-copy, and cancellation workflows.
- [ ] Document privacy guarantees and limitations.
- [ ] Document troubleshooting by stable error code.
- [ ] Document diagnostic report generation.
- [ ] Document complete uninstall.
- [ ] Document unsupported desktop environments honestly.

### M16.3 Developer documentation

- [ ] Document architecture and module boundaries.
- [ ] Document Tauri command/event contracts.
- [ ] Document capture backend extension process.
- [ ] Document OCR engine extension process.
- [ ] Document fixture generation.
- [ ] Document local test commands.
- [ ] Document release build procedure on Ubuntu 22.04.
- [ ] Document manual validation procedure.
- [ ] Document version and settings migration policy.

### M16.4 Release audit

- [ ] Review all TODO checkboxes and evidence.
- [ ] Review unresolved issues by severity.
- [ ] Run secret scan.
- [ ] Run dependency/license review.
- [ ] Run content-leakage grep/audit.
- [ ] Run clean release build.
- [ ] Run exact CI gates on release commit.
- [ ] Install exact artifact on all required validation environments.
- [ ] Verify artifact checksum matches tested package.
- [ ] Verify release notes contain known limitations.
- [ ] Tag only after all gates pass.

### M16.5 Final evidence report

- [ ] Create `docs/SCREENSHOT_OCR_V0_1_IMPLEMENTATION_REPORT.md`.
- [ ] Record final commit SHA.
- [ ] Record CI run links/identifiers.
- [ ] Record package filename and checksum.
- [ ] Summarize automated test results.
- [ ] Link all four manual validation records.
- [ ] List all deferred work.
- [ ] State explicitly whether each v0.1 acceptance criterion passed.
- [ ] Avoid unsupported claims such as “works on all Linux distributions.”

### M16 acceptance gate

- [ ] Exact release artifact is proven on the required matrix.
- [ ] All v0.1 acceptance criteria in the specification pass.
- [ ] Release documentation and implementation report are complete.
- [ ] No known P0/P1 defect remains.

## 22. Required test fixtures and expected properties

The fixture suite must include at least the following. Exact source text should be synthetic and safe to publish.

### FIX-001 Dark terminal paragraph

- [ ] Light monospace text on near-black background.
- [ ] Multiple paragraphs and blank lines.
- [ ] Expected: line ordering and blank lines preserved.

### FIX-002 Shell command

- [ ] Paths, pipes, redirects, long options, underscores, hyphens, quotes, and backslashes.
- [ ] Expected: no smart punctuation or spelling correction.

### FIX-003 Rust compiler diagnostic

- [ ] Error code, file path, line/column, carets, braces, lifetimes, and generic types.
- [ ] Expected: structurally recognizable output and preserved line breaks.

### FIX-004 JSON/TOML/YAML/Markdown

- [ ] Representative punctuation and indentation.
- [ ] Expected: indentation and delimiters preserved within documented OCR tolerance.

### FIX-005 Small antialiased text

- [ ] Simulated fractional scaling.
- [ ] Expected: upscaled variant outperforms or matches original.

### FIX-006 Light theme

- [ ] Dark text on light background.
- [ ] Expected: no unnecessary inversion.

### FIX-007 Ambiguous glyphs

- [ ] `0/O`, `1/l/I`, pipe, backtick, apostrophe, colon/semicolon, braces/parentheses, hyphen/underscore.
- [ ] Expected: warnings or measured accuracy; no silent generative correction.

### FIX-008 Empty/non-text region

- [ ] Background with no readable text.
- [ ] Expected: empty warning and clipboard unchanged.

### FIX-009 Oversized dimensions

- [ ] Synthetic header or mocked decoder metadata only; avoid committing huge binary files.
- [ ] Expected: bounded rejection before dangerous allocation.

### FIX-010 Synthetic secret

- [ ] Distinctive fake token included in a test image.
- [ ] Expected: recognized result may contain token in memory, but logs, diagnostics, panic output, and CI artifacts do not.

## 23. Public error-code verification checklist

For each error code, add at least one test and verify its user guidance:

- [ ] `capture_already_active`
- [ ] `capture_cancelled`
- [ ] `capture_backend_unavailable`
- [ ] `capture_permission_denied`
- [ ] `capture_process_failed`
- [ ] `capture_result_missing`
- [ ] `capture_image_invalid`
- [ ] `capture_too_large`
- [ ] `temporary_cleanup_failed`
- [ ] `ocr_engine_unavailable`
- [ ] `ocr_language_missing`
- [ ] `ocr_timed_out`
- [ ] `ocr_failed`
- [ ] `ocr_empty_result`
- [ ] `clipboard_unavailable`
- [ ] `clipboard_write_failed`
- [ ] `settings_invalid`
- [ ] `settings_write_failed`
- [ ] `unsupported_environment`
- [ ] `internal_error`

## 24. Release-blocking invariants

These invariants are fail-closed. Violating any one blocks release:

- [ ] No capture or OCR path invokes a shell with constructed user-influenced text.
- [ ] No screenshot or OCR content is transmitted over the network.
- [ ] No screenshot or OCR content appears in logs or diagnostics.
- [ ] Cancellation, empty OCR, and OCR failure do not overwrite clipboard.
- [ ] Temporary capture artifacts are deleted or a visible cleanup failure is produced.
- [ ] Only one region selector can be active.
- [ ] The frontend cannot choose an arbitrary executable or filesystem path.
- [ ] Portal area capture is used only when capability is advertised.
- [ ] The Ubuntu 22.04 GNOME fallback is preserved until the minimum supported platform changes through an explicit spec update.
- [ ] CI success is not substituted for manual Wayland/X11 validation.
- [ ] A release artifact is not claimed as validated unless the exact checksum was tested.

## 25. Deferred post-v0.1 backlog

These tasks are intentionally not part of v0.1 unless the specification is updated:

- [ ] KDE/Spectacle capture backend.
- [ ] Sway/Hyprland/wlroots capture backend.
- [ ] Flatpak package.
- [ ] Stable AppImage support.
- [ ] Additional OCR language packs.
- [ ] Automatic language detection.
- [ ] PaddleOCR engine.
- [ ] Optional encrypted local history.
- [ ] Copy as Markdown code block.
- [ ] Shell prompt-prefix removal.
- [ ] Code-language-aware suspicious glyph hints.
- [ ] Local translation.
- [ ] Windows support.
- [ ] macOS support.

Deferred items must not leak into v0.1 as incomplete UI controls or undocumented experimental behavior.

## 26. Recommended first implementation sequence

The shortest path to a demonstrable but structurally sound vertical slice is:

1. [ ] Complete M0 decisions.
2. [ ] Scaffold Tauri/React/TypeScript/Tailwind under M1.
3. [ ] Implement minimal M2 state and fake services.
4. [ ] Build M4 capture/preview UI against fakes.
5. [ ] Implement M6 GNOME helper capture first for Ubuntu compatibility.
6. [ ] Implement M8 one safe preprocessing path.
7. [ ] Implement M9 Tesseract English terminal mode.
8. [ ] Implement M10 clipboard.
9. [ ] Connect the vertical slice through M11.
10. [ ] Prove the slice manually on Ubuntu 22.04 Wayland before expanding.
11. [ ] Add M7 portal optimization.
12. [ ] Add shortcut/tray/single-instance integration.
13. [ ] Complete hardening, matrix validation, packaging, and signoff.

The vertical slice is not a release. Privacy, cleanup, test, and platform gates remain mandatory.

## 27. Completion record

When v0.1 is complete, replace this section with a concise record containing:

- release tag;
- release commit SHA;
- tested `.deb` filename and SHA-256;
- CI evidence;
- Ubuntu validation evidence links;
- unresolved non-blocking issues;
- next planned milestone.

Until then, the project status is **pre-release and incomplete**.
