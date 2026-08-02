# Screenshot OCR FIX1 Remediation Specification

**Repository:** `ekkus93/screenshot-ocr`  
**Document:** `docs/SCREENSHOT_OCR_FIX1_SPEC_2026-08-02.md`  
**Companion TODO:** `docs/SCREENSHOT_OCR_FIX1_TODO_2026-08-02.md`  
**Date:** 2026-08-02  
**Baseline reviewed commit:** `b45fc89027dddef981332dbde976122cffa70050`  
**Status:** implementation specification for the first post-review remediation pass

## 1. Purpose

FIX1 turns the current green automated vertical slice into a more truthful, safer, and more testable pre-release candidate. It does not try to finish the full v0.1 release matrix. Instead, it fixes the concrete issues found in the comprehensive code review and prepares the project for physical Ubuntu validation.

The current codebase already has a working architecture: Tauri 2, React/TypeScript/Tailwind, Rust command adapters, capture backends, bounded image decoding, Tesseract OCR, clipboard write support, settings, diagnostics, CI, and Debian package smoke. FIX1 must preserve those strengths while closing the highest-risk correctness and UX gaps.

## 2. Non-goals

FIX1 is not the final v0.1 release.

FIX1 must not claim completion of:

- Ubuntu 22.04 GNOME Wayland physical validation;
- Ubuntu 22.04 GNOME X11 physical validation;
- Ubuntu 24.04 GNOME Wayland physical validation;
- Ubuntu 24.04 GNOME X11 physical validation;
- complete OCR accuracy certification;
- full accessibility certification;
- final release tagging;
- support for KDE, wlroots compositors, Windows, macOS, Flatpak, AppImage, or additional OCR languages.

Physical desktop validation remains a separate release gate under the existing v0.1 TODO.

## 3. Guiding principles

1. **Truthful UI beats aspirational UI.** A visible setting or action must either work, be explicitly disabled with explanation, or be removed until implemented.
2. **Recoverability beats opaque failure.** If the app has recognized text but cannot copy it, the user must be able to recover, review, edit, and retry copying that text.
3. **Bounded subprocesses everywhere.** Every external helper invocation must have timeout, cancellation where applicable, bounded output, and no shell execution.
4. **Privacy remains fail-closed.** Screenshot pixels, OCR text, clipboard contents, executable paths, temporary paths, and portal result URIs must not appear in logs, public errors, diagnostics, panic messages, or CI artifacts.
5. **TODO checkboxes require evidence.** Existing v0.1 TODO items may only be checked when implementation, tests, documentation, and required validation evidence exist.
6. **No CI/physical-validation conflation.** Hosted CI can prove source quality and package creation. It cannot prove real GNOME Wayland/X11 selector, tray, shortcut, clipboard, scaling, or multi-monitor behavior.

## 4. Severity definitions

- **P0:** Could leak screenshot/OCR/clipboard content, execute attacker-controlled commands, corrupt unrelated files, silently overwrite clipboard on failure/cancellation, or allow multiple active selectors.
- **P1:** Blocks trustworthy pre-release use or makes the UI materially misleading, such as unimplemented settings, invisible save failures, unbounded helper probes, or loss of recognized text after clipboard failure.
- **P2:** Important correctness, test coverage, documentation, or maintainability gap that should be fixed before final v0.1 but may not block a FIX1 candidate.

FIX1 must leave no known P0 issue and should close all P1 issues listed in this document unless explicitly deferred with rationale.

## 5. Affected modules

Expected affected areas include:

```text
src/app/useAppController.ts
src/app/App.tsx
src/features/capture/CapturePanel.tsx
src/features/settings/SettingsPanel.tsx
src/features/diagnostics/DiagnosticsPanel.tsx
src/lib/types.ts
src/lib/tauri.ts
src-tauri/src/actions.rs
src-tauri/src/app.rs
src-tauri/src/capture/environment.rs
src-tauri/src/capture/gnome.rs
src-tauri/src/capture/portal.rs
src-tauri/src/commands.rs
src-tauri/src/diagnostics.rs
src-tauri/src/error.rs
src-tauri/src/image_pipeline/mod.rs
src-tauri/src/models.rs
src-tauri/src/ocr/tesseract.rs
src-tauri/src/ocr/cleanup.rs
src-tauri/src/settings.rs
src-tauri/src/desktop.rs
src-tauri/capabilities/default.json
.github/workflows/ci.yml
docs/SCREENSHOT_OCR_V0_1_TODO.md
docs/IMPLEMENTATION_STATUS_2026-08-02.md
README.md
docs/PRIVACY.md
docs/THREAT_MODEL.md
docs/ARCHITECTURE.md
```

This list is not exclusive. Keep source files below the repository limit and split modules before they become too large.

## 6. Requirements

### FIX1-R1: Settings UI truthfulness

The Settings UI must not present working-looking controls for behavior that is not implemented.

At baseline, the following controls are misleading or incomplete:

- `notifyAfterCopy` / "Notify after copy";
- `startAtLogin` / "Start at login";
- `closeToTray` / "Keep running when window closes";
- `preserveWhitespace` if it is not actually used to alter cleanup behavior.

FIX1 must choose one of the following policies for each control:

1. implement it fully with tests and documentation;
2. keep the field internally but remove the active UI control;
3. display it disabled with explicit "not implemented in this build" copy;
4. remove it from schema only if a migration/reset strategy is defined.

The preferred FIX1 approach is to remove or disable aspirational controls rather than implementing autostart, notifications, and close-to-tray in this pass.

The Settings page must display save/reset/load errors on the Settings tab itself. Errors must not be visible only on the Capture tab.

### FIX1-R2: Corrupt settings recovery warning

When settings are corrupt, invalid, or quarantined, the app must not silently present defaults as if nothing happened.

Required behavior:

- Preserve/quarantine the corrupt settings file where feasible.
- Return safe recovery state to the frontend without exposing paths.
- Show a visible warning that defaults were loaded because stored settings were invalid.
- Provide a clear user action, such as "Save defaults" or "Reset settings".
- Store only safe error code and recovery reason in diagnostics.

Public errors and diagnostics must not include the settings file path, raw JSON, or any user content.

### FIX1-R3: Immediate-copy clipboard failure recovery

In immediate-copy mode, if OCR succeeds but clipboard write fails, recognized text must remain recoverable.

Required behavior:

- Do not lose recognized OCR text when clipboard write fails.
- Restore/focus the window.
- Put the recognized text into the preview editor.
- Show a clipboard-failure error or warning.
- Provide a retry copy action.
- Preserve job ID and safe metadata where useful.
- Do not claim copied success.
- Do not write partial or whitespace-only text.

This requirement applies only when OCR has already succeeded. If capture or OCR fails before text exists, normal error behavior remains appropriate.

### FIX1-R4: Processing-state correctness

The frontend currently moves from `preparing` to `selecting` and then awaits the full Rust command. It does not show a true OCR `processing` state.

FIX1 must make capture stages truthful.

Acceptable approaches:

- Add Rust-to-frontend stage events keyed by job ID; or
- Split the command flow into explicit phases; or
- Show a conservative combined "Selecting or recognizing text" state if precise stage events are not implemented.

Preferred approach:

- Add safe stage events from Rust using stable event names and DTOs.
- Events must include job ID and stage enum only.
- Events must not include paths, OCR text, images, command args, URIs, stderr, or stdout.
- Frontend must ignore stale events by job ID/request token.

Required stages at minimum:

```text
preparing
selecting
captured
processing
copying
completed
```

Cancellation and failure must remain separate visible states.

### FIX1-R5: Bounded Tesseract probing

`TesseractEngine::probe_english()` must be hardened.

Required behavior:

- Use async process spawning or a bounded blocking wrapper.
- Do not invoke a shell.
- Set stdin to null.
- Capture stdout with a strict byte limit.
- Suppress or safely bound stderr.
- Apply timeout.
- Kill and reap the child on timeout or cancellation where a cancellation token is available.
- Map absent/broken executable to `OcrEngineUnavailable`.
- Map missing English language data to `OcrLanguageMissing`.
- Never log raw `--list-langs` output.

Diagnostics may call the bounded probe, but diagnostics must not hang indefinitely.

Version detection may be implemented in FIX1 if bounded; otherwise it must remain explicitly deferred and documented.

### FIX1-R6: Image preprocessing correctness and limits

Preprocessing must not violate the resource limits established during decode.

Required behavior:

- Before resizing, calculate the resulting width, height, and pixel count.
- Reject or skip variants that would exceed maximum width, height, pixel count, or memory policy.
- Bound total variant count.
- Ensure generated variants are deterministic.
- Avoid writing intermediate variants to disk.
- Rename `GrayscaleContrast` if it is only grayscale, or implement actual contrast normalization.
- Remove `Upscale3x` from the public enum until implemented, or implement it with bounded policy and tests.

If any variant is skipped due to limits, this should not become a user-facing error unless no safe variant remains.

### FIX1-R7: Temporary capture ownership and stale cleanup plan

The GNOME capture backend already creates a private random directory and cleans it up. FIX1 must add a stronger ownership strategy.

Required behavior:

- Add a content-free ownership marker in application-owned temp/runtime capture directories.
- Marker must not contain OCR text, screenshot content, command args, or sensitive paths.
- Startup or pre-capture scavenging may remove only directories that match all ownership constraints.
- Scavenging must never traverse or delete arbitrary user files.
- Scavenging failures must be counted or surfaced safely.
- If ownership-marker implementation is deferred, document why and leave the v0.1 TODO item unchecked.

Also fix the edge case where a created capture directory remains behind if permission-setting fails immediately after creation.

### FIX1-R8: Portal cleanup and resource lifecycle review

The portal backend must have an explicit resource lifecycle decision.

Required behavior:

- Review whether `ashpd` exposes request close/cancel or resource release APIs for screenshot results.
- If supported, close/cancel outstanding portal requests on cancellation/timeout.
- If returned files should be deleted by this application, delete them after decoding.
- If returned files are managed by the portal or outside application ownership, document the behavior and do not over-delete.
- Never serialize or log portal result URIs or paths.

### FIX1-R9: Frontend production/test boundary

Production components must not import types from `src/test`.

Required behavior:

- Move shared controller return/type helpers to production source, such as `src/app/controllerTypes.ts` or `src/lib/types.ts`.
- Test-only helpers may depend on production types, but production modules must not depend on `src/test`.
- Add lint or review guidance if practical.

### FIX1-R10: Tray and shortcut truthfulness

The tray and shortcut implementation must match the documented/product surface.

Required behavior:

- If the TODO requires a Settings tray item, either implement it or update/defer the TODO with rationale.
- Tray actions must route through the same authoritative app services as the main UI.
- Shortcut status in diagnostics must not claim registration on unsupported Wayland sessions.
- GNOME custom shortcut instructions must remain the required compatibility path for Wayland.
- Repeated invocations must not launch overlapping selectors.

Start-at-login and close-to-tray should be removed/disabled in the UI unless fully implemented.

### FIX1-R11: Content-leakage audit automation

FIX1 must add at least one automated guard against accidental sensitive output.

Required behavior:

- Add a deterministic synthetic secret fixture string used only in tests.
- Verify public errors, diagnostics, and captured test logs do not contain that secret.
- Search first-party source for obvious logging/console statements that could leak OCR text, temporary paths, command args, portal URIs, or clipboard contents.
- Ensure CI does not upload test artifacts containing screenshots or OCR text.

The guard does not replace full manual release audit, but it should catch obvious regressions.

### FIX1-R12: OCR fixture foundation

FIX1 does not need to certify OCR accuracy, but it should create the foundation for deterministic OCR regression testing.

Required behavior:

- Add synthetic fixture generation documentation or a small fixture generator.
- Include at least one lightweight generated image or mocked OCR fixture for whitespace/punctuation cleanup if real Tesseract fixtures are too environment-sensitive.
- Ensure no private screenshots are committed.
- Separate exact text cleanup tests from real Tesseract accuracy tests.

### FIX1-R13: Public error-code reconciliation

The current TODO lists `clipboard_unavailable`, but the implemented error enum contains `clipboard_write_failed` and not `clipboard_unavailable`.

Required behavior:

- Decide whether `clipboard_unavailable` is a real required public error code.
- If yes, add it and map backend availability failures separately from write failures.
- If no, update the TODO/spec references to remove or defer it.
- Add at least one test per changed public error mapping.

### FIX1-R14: CI/security hardening decision

The existing TODO asks for GitHub Actions pinned by immutable commit SHA, but CI currently uses version tags.

Required behavior:

- Decide whether FIX1 will pin third-party actions by SHA.
- If yes, update workflow actions and document update procedure.
- If no, explicitly defer the item in the TODO/status docs and leave v0.1 release gate unchecked.

Do not claim this item complete while using mutable tags.

### FIX1-R15: Documentation and evidence updates

FIX1 must update documentation to match reality.

Required updates:

- `README.md`: remove or clarify unimplemented features exposed in UI.
- `docs/PRIVACY.md`: update any clipboard/corrupt-settings/temporary-file behavior changes.
- `docs/THREAT_MODEL.md`: add bounded Tesseract probe and settings recovery mitigations.
- `docs/ARCHITECTURE.md`: reflect new stage-event or recovery-result contracts.
- `docs/IMPLEMENTATION_STATUS_2026-08-02.md` or a new status doc: record what FIX1 changed and what remains open.
- `docs/SCREENSHOT_OCR_V0_1_TODO.md`: only check boxes backed by implementation, tests, docs, and evidence.

## 7. API and DTO expectations

### 7.1 Stage event DTO

If stage events are implemented, the DTO should be content-free:

```ts
interface CaptureStageEvent {
  jobId: string;
  stage: "preparing" | "selecting" | "captured" | "processing" | "copying" | "completed";
}
```

Rust equivalent should use strongly typed enums and serde rename rules. Event names must be stable and documented.

### 7.2 Recoverable OCR result on clipboard failure

FIX1 may implement one of these patterns:

1. Return `OcrResult` with `copied = false` plus a nonfatal warning containing `clipboard_write_failed`; or
2. Return a richer command response:

```ts
type CaptureCommandOutcome =
  | { status: "ok"; result: OcrResult }
  | { status: "recoverableClipboardFailure"; result: OcrResult; error: PublicError };
```

The second pattern is clearer but requires more frontend/Rust DTO work. Either is acceptable if the UI receives the recognized text and does not claim copied success.

### 7.3 Settings load status

FIX1 should avoid hiding recovery state. A possible DTO:

```ts
interface SettingsLoadResult {
  settings: AppSettings;
  warning: null | {
    code: "settings_invalid";
    message: string;
    guidance: string;
    recoveredWithDefaults: boolean;
  };
}
```

If keeping `get_settings(): AppSettings`, then the controller must fetch safe runtime diagnostics or a separate settings warning state. The result must be visible in Settings UI.

## 8. Testing requirements

FIX1 must add deterministic automated tests for each changed behavior.

Minimum required tests:

- Settings page shows save failure on the Settings tab.
- Corrupt settings quarantine produces a visible warning/recovery state.
- Immediate-copy clipboard failure preserves recognized text in the editor and shows retry path.
- Stage events or truthful combined status cannot leave the app semantically stuck at selecting during OCR.
- Tesseract language probe times out and kills/reaps a fake hanging helper.
- Tesseract language probe rejects oversized language output.
- Resized image variants do not exceed configured dimension/pixel limits.
- `GrayscaleContrast` rename or true contrast implementation is reflected in tests.
- Production components no longer import from `src/test`.
- Tray/shortcut repeated invocation still rejects overlapping captures at the state/service layer.
- Diagnostics/public errors do not contain synthetic secret or sensitive paths.
- Public error-code reconciliation is tested.

Where real desktop behavior cannot be tested in CI, add unit/integration tests around deterministic state/policy functions and leave manual validation unchecked.

## 9. Validation commands

A FIX1 candidate must pass these commands locally or in hosted CI:

```bash
npm ci
npm run format:check
npm run lint
npm run typecheck
npm run test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-features --locked
npm run tauri -- build --bundles deb
```

Hosted CI must pass on the exact final commit.

## 10. FIX1 acceptance criteria

FIX1 is complete only when all of the following are true:

- The app no longer exposes active-looking controls for unimplemented notification/autostart/close-to-tray behavior.
- Settings errors and corrupt-settings recovery are visible on the Settings UI.
- Immediate-copy clipboard failure preserves recognized text and provides a retry path.
- Capture/OCR progress status is truthful and stale events/results are rejected.
- Tesseract language probing is bounded and cannot hang indefinitely.
- Image preprocessing cannot exceed configured resource limits after resizing.
- Production components do not import from `src/test`.
- GNOME temporary ownership/stale cleanup is implemented or explicitly deferred with evidence and unchecked TODO state.
- Portal lifecycle behavior is reviewed, implemented where supported, and documented.
- Content-leakage guard tests exist.
- Public error-code mismatch is resolved.
- CI/security action pinning decision is recorded honestly.
- Documentation reflects the implemented behavior.
- Existing v0.1 TODO checkboxes are updated only where evidence meets the stated checkbox rules.
- Full CI passes on the final commit.

## 11. Remaining post-FIX1 gates

After FIX1, the following remain mandatory before v0.1 release:

- four-environment Ubuntu GNOME physical validation;
- real clipboard paste/ownership validation;
- real tray, shortcut, close/quit behavior validation;
- real portal behavior validation where supported;
- multi-monitor and scaling validation;
- OCR fixture/accuracy evidence;
- package install/upgrade/reinstall/removal evidence;
- dependency/license/security review;
- final implementation report and release signoff.

Do not collapse these into FIX1 unless the implementation and evidence are actually produced.
