# Screenshot OCR FIX1 Remediation TODO

**Repository:** `ekkus93/screenshot-ocr`  
**Document:** `docs/SCREENSHOT_OCR_FIX1_TODO_2026-08-02.md`  
**Companion spec:** `docs/SCREENSHOT_OCR_FIX1_SPEC_2026-08-02.md`  
**Date:** 2026-08-02  
**Baseline reviewed commit:** `b45fc89027dddef981332dbde976122cffa70050`  
**Status:** implementation checklist for FIX1 review remediation

## 1. Rules for this TODO

- `[ ]` means not implemented or not proven.
- `[x]` means implemented, tested, documented, and proven by evidence.
- Do not check a parent task only because some child code exists.
- Do not claim Ubuntu Wayland/X11 physical validation from hosted CI.
- Do not suppress warnings to make CI pass.
- Do not log screenshots, OCR text, clipboard text, executable paths, temporary paths, portal result URIs, raw stderr, or raw stdout.
- Do not create a branch or pull request unless the repository owner explicitly requests one.
- Keep first-party source files below the repository line-limit policy.

## 2. Target outcome

FIX1 is complete when the app is a truthful, safer pre-release candidate with the highest-risk review findings fixed:

- no active-looking UI controls for unimplemented features;
- visible settings and corrupt-settings recovery errors;
- recoverable immediate-copy clipboard failure;
- truthful capture/OCR progress state;
- bounded Tesseract probing;
- bounded image preprocessing after resize;
- production/test boundary cleanup;
- explicit GNOME/portal cleanup decisions;
- content-leakage guard tests;
- public error-code reconciliation;
- documentation and v0.1 TODO updated honestly;
- full CI passing on the final commit.

## 3. Milestone map

```text
F1.0 Baseline and safety guardrails
F1.1 Settings UI truthfulness and error visibility
F1.2 Corrupt settings recovery
F1.3 Immediate-copy clipboard failure recovery
F1.4 Capture/OCR progress state
F1.5 Tesseract probe hardening
F1.6 Image preprocessing limits and naming
F1.7 GNOME temp ownership and stale cleanup
F1.8 Portal lifecycle review
F1.9 Frontend production/test boundary
F1.10 Tray, shortcut, startup truthfulness
F1.11 Content-leakage guard and OCR fixture foundation
F1.12 Public error-code and CI/security reconciliation
F1.13 Documentation and authoritative TODO reconciliation
F1.14 Final validation and evidence
```

## 4. F1.0 — Baseline and safety guardrails

### F1.0.1 Confirm starting point

- [ ] Confirm `master` is at or descends from reviewed commit `b45fc89027dddef981332dbde976122cffa70050`.
- [ ] Read `docs/SCREENSHOT_OCR_FIX1_SPEC_2026-08-02.md`.
- [ ] Read `docs/SCREENSHOT_OCR_V0_1_TODO.md` task-state rules.
- [ ] Read `docs/IMPLEMENTATION_STATUS_2026-08-02.md` and preserve its distinction between hosted CI and physical validation.
- [ ] Run or inspect current CI status before editing.

### F1.0.2 No-regression rules

- [ ] Preserve no-shell invocation for capture and OCR.
- [ ] Preserve one-active-capture ownership.
- [ ] Preserve no persistent OCR history.
- [ ] Preserve no broad Tauri filesystem or shell capabilities.
- [ ] Preserve CI gates for frontend, Rust, repository policy, and `.deb` packaging.
- [ ] Preserve GNOME fallback support.
- [ ] Preserve portal area-capture fail-closed policy.

### F1.0 acceptance gate

- [ ] The implementation plan is understood and no safety baseline is weakened.

## 5. F1.1 — Settings UI truthfulness and Settings-page errors

### F1.1.1 Audit settings fields and UI controls

- [ ] List every field in `AppSettings`.
- [ ] List every visible Settings UI control.
- [ ] For each control, classify behavior as implemented, partially implemented, or not implemented.
- [ ] Specifically classify:
  - [ ] `notifyAfterCopy`.
  - [ ] `startAtLogin`.
  - [ ] `closeToTray`.
  - [ ] `preserveWhitespace`.
  - [ ] `captureBackend`.
  - [ ] `previewBeforeCopy`.
  - [ ] `textMode`.
  - [ ] `language`.
  - [ ] `shortcut` display/instructions.

### F1.1.2 Remove or disable aspirational controls

Choose the simplest honest behavior for FIX1.

- [ ] Remove or disable the active `Notify after copy` control unless notifications are fully implemented.
- [ ] Remove or disable the active `Start at login` control unless autostart is fully implemented.
- [ ] Remove or disable the active `Keep running when window closes` control unless close-to-tray behavior is fully implemented.
- [ ] If controls are disabled, add explicit copy such as `Not implemented in this pre-release build`.
- [ ] If controls are removed from UI but fields remain in schema, keep defaults stable and document that the fields are reserved.
- [ ] Do not remove schema fields without a migration/reset plan.

### F1.1.3 Make preserve-whitespace truthful

- [ ] Decide whether `preserveWhitespace` is a real user-configurable behavior in FIX1.
- [ ] If yes, apply it in OCR cleanup policy and test both values.
- [ ] If no, remove or disable the UI control and document terminal mode as always preserving whitespace.
- [ ] Ensure disabling/removing the control cannot make valid default settings fail validation.

### F1.1.4 Show Settings errors on Settings page

- [ ] Add a visible error/warning region inside `SettingsPanel`.
- [ ] Display save failures on the Settings page.
- [ ] Display reset failures on the Settings page.
- [ ] Display settings-load recovery warnings on the Settings page after F1.2.
- [ ] Ensure errors are accessible with `role="alert"` or an equivalent accessible pattern.
- [ ] Ensure errors do not expose paths or raw settings JSON.
- [ ] Preserve unsaved values after failed save.

### F1.1.5 Tests

- [ ] Add frontend test: Settings save failure is visible on Settings tab.
- [ ] Add frontend test: failed save does not discard unsaved settings.
- [ ] Add frontend test: removed/disabled aspirational controls do not appear as active working controls.
- [ ] Add frontend test: capture backend/text mode/preview controls still work where implemented.

### F1.1 acceptance gate

- [ ] The Settings page no longer misrepresents unimplemented functionality.
- [ ] Settings errors are visible where the user is acting.

## 6. F1.2 — Corrupt settings recovery

### F1.2.1 Backend recovery contract

- [ ] Replace silent `get_settings -> default settings` recovery with an explicit safe recovery contract.
- [ ] Add a DTO such as `SettingsLoadResult` or equivalent.
- [ ] Include `settings` plus optional safe warning.
- [ ] Warning must include stable code, message, guidance, and `recoveredWithDefaults` or equivalent.
- [ ] Warning must not include filesystem paths, raw JSON, or user content.
- [ ] Preserve quarantine behavior for corrupt settings.
- [ ] Record safe runtime diagnostic error code.

### F1.2.2 Frontend integration

- [ ] Update `getSettings` TypeScript wrapper and types.
- [ ] Update `useAppController` initialization to handle recovery warnings.
- [ ] Show recovery warning in Settings UI.
- [ ] Avoid showing scary fatal error styling for successful recovery.
- [ ] Provide a clear next action: save defaults/reset/review settings.

### F1.2.3 Settings tests

- [ ] Rust test: corrupt settings are quarantined.
- [ ] Rust test: corrupt settings load returns safe recovery warning/result.
- [ ] Rust test: recovery warning serializes without paths or raw content.
- [ ] Frontend test: corrupt settings warning appears on Settings tab.
- [ ] Frontend test: defaults are visible after recovery but warning is not hidden.

### F1.2 acceptance gate

- [ ] Corrupt settings recovery is visible and content-free.

## 7. F1.3 — Immediate-copy clipboard failure recovery

### F1.3.1 Backend outcome design

Choose and implement one recoverable design:

- [ ] Option A: Return `OcrResult` with `copied = false` and warning `clipboard_write_failed` after OCR success but clipboard failure.
- [ ] Option B: Return a richer command outcome with `result` plus `PublicError` for recoverable clipboard failure.
- [ ] Document the chosen design in code comments or architecture docs.

Required behavior:

- [ ] If OCR fails before text exists, return ordinary public error.
- [ ] If OCR succeeds and immediate clipboard write fails, return recoverable text to frontend.
- [ ] Do not claim copied success.
- [ ] Restore/focus the window.
- [ ] Record safe last error code.
- [ ] Do not write whitespace-only output.
- [ ] Do not log or serialize clipboard contents outside the result text intentionally returned to the active UI.

### F1.3.2 Frontend UX

- [ ] Put recovered text into editor after immediate-copy clipboard failure.
- [ ] Set status to a recoverable review/error hybrid state, or reviewing with visible error.
- [ ] Show clipboard failure message and guidance.
- [ ] Keep `Copy text` retry action enabled.
- [ ] Ensure `Clear` and `Capture again` behave normally after failure.

### F1.3.3 Tests

- [ ] Rust unit/integration test: immediate-copy clipboard failure returns recoverable result or warning.
- [ ] Rust test: copied flag remains false on clipboard failure.
- [ ] Frontend test: immediate-copy clipboard failure preserves text in editor.
- [ ] Frontend test: retry copy calls `copy_text` with edited text.
- [ ] Frontend test: clipboard failure does not display `Text copied`.

### F1.3 acceptance gate

- [ ] Recognized text is never lost merely because clipboard write failed after OCR succeeded.

## 8. F1.4 — Capture/OCR progress state

### F1.4.1 Decide progress mechanism

- [ ] Decide whether to implement Rust stage events or conservative combined UI copy.
- [ ] Prefer Rust stage events if feasible without overexpanding scope.
- [ ] Document the decision.

### F1.4.2 Stage event implementation, if selected

- [ ] Add Rust `CaptureStage` enum.
- [ ] Add Rust `CaptureStageEvent` DTO with `job_id` and `stage` only.
- [ ] Add stable event name, for example `screenshot-ocr://capture-stage`.
- [ ] Emit `preparing` before environment/settings/backend work.
- [ ] Emit `selecting` before opening region selector.
- [ ] Emit `captured` after pixels are loaded into memory and source artifact cleanup is attempted.
- [ ] Emit `processing` before OCR/preprocessing loop.
- [ ] Emit `copying` before immediate-copy clipboard write.
- [ ] Emit `completed` before command success return.
- [ ] Never emit OCR text, paths, command args, stderr, stdout, image data, portal URI, or diagnostics content in stage events.

### F1.4.3 Frontend stage handling

- [ ] Listen for capture-stage events.
- [ ] Ignore stale event job IDs.
- [ ] Ignore stale request tokens.
- [ ] Map stages to visible status text.
- [ ] Ensure `processing` can be shown during OCR.
- [ ] Ensure cancellation/failure states override stage events correctly.

### F1.4.4 Conservative alternative, if events deferred

- [ ] Change status copy from `Select a screen region` to truthful combined copy such as `Select a region; OCR will continue after selection`.
- [ ] Remove unused `processing` state or document why it remains reserved.
- [ ] Add tests for the truthful status copy.

### F1.4.5 Tests

- [ ] Rust test: stage event DTO serializes with job ID and stage only.
- [ ] Rust test: stage event serialization does not contain synthetic secret/path fixture.
- [ ] Frontend test: stale stage event is ignored.
- [ ] Frontend test: processing status appears for matching job.
- [ ] Frontend test: cancellation is not overwritten by late processing/completed event.

### F1.4 acceptance gate

- [ ] The UI no longer incorrectly remains semantically stuck at `selecting` during OCR.

## 9. F1.5 — Tesseract probe hardening

### F1.5.1 Refactor probe

- [ ] Convert `TesseractEngine::probe_english()` to bounded async or a bounded blocking wrapper.
- [ ] Do not invoke a shell.
- [ ] Set stdin to null.
- [ ] Capture stdout with a strict limit.
- [ ] Suppress or safely bound stderr.
- [ ] Apply timeout.
- [ ] Kill and reap child on timeout.
- [ ] Return `OcrEngineUnavailable` for missing/broken executable.
- [ ] Return `OcrLanguageMissing` for missing `eng`.
- [ ] Never log raw language output.

### F1.5.2 Cancellation-aware capture use

- [ ] Allow capture flow to pass cancellation token into Tesseract language probe.
- [ ] Ensure capture cancellation during language probing exits promptly.
- [ ] Ensure diagnostics probing cannot hang indefinitely even without cancellation.

### F1.5.3 Version decision

- [ ] Decide whether to implement bounded `tesseract --version` in FIX1.
- [ ] If implemented, parse only safe bounded text and expose a safe version string.
- [ ] If deferred, document the deferral and keep the v0.1 TODO version item unchecked.

### F1.5.4 Tests

- [ ] Test fake hanging `--list-langs` helper times out.
- [ ] Test timed-out probe kills/reaps helper.
- [ ] Test oversized stdout is rejected.
- [ ] Test missing `eng` returns `OcrLanguageMissing`.
- [ ] Test valid output returns sanitized language codes.
- [ ] Test weird language names are filtered.
- [ ] Test diagnostics path uses bounded probe.

### F1.5 acceptance gate

- [ ] Tesseract probing cannot hang capture or diagnostics indefinitely.

## 10. F1.6 — Image preprocessing limits and naming

### F1.6.1 Shared resource policy

- [ ] Centralize or clearly share max encoded bytes, width, height, and decoded pixel limits.
- [ ] Add helper that validates dimensions for both decoded and generated images.
- [ ] Use checked arithmetic for width/height scaling.
- [ ] Skip unsafe variants instead of panicking or overallocating.

### F1.6.2 Fix resize-bound bug

- [ ] Before 2x resize, validate scaled dimensions and scaled pixel count.
- [ ] Add boundary test for image where original is allowed but 2x would exceed limit.
- [ ] Ensure at least original/grayscale safe variants remain when upscale is skipped.

### F1.6.3 Resolve misleading variant names

- [ ] Decide whether `GrayscaleContrast` becomes true contrast normalization.
- [ ] If implementing true contrast normalization, add deterministic tests proving pixel transformation.
- [ ] If not implementing contrast, rename enum to `Grayscale` and update frontend/types/tests/docs.
- [ ] Decide whether `Upscale3x` is implemented or removed/deferred.
- [ ] If implementing 3x, add strict bounds and tests.
- [ ] If removing/defering 3x, update enum, docs, and TODO state honestly.

### F1.6.4 Tests

- [ ] Test zero dimensions still rejected.
- [ ] Test max allowed dimensions accepted where safe.
- [ ] Test max+1 dimensions rejected.
- [ ] Test resize-skip boundary.
- [ ] Test dark image still gets inverted variant.
- [ ] Test no intermediate preprocessing files are written.

### F1.6 acceptance gate

- [ ] Preprocessing cannot exceed declared resource limits and variant names match implementation.

## 11. F1.7 — GNOME temp ownership and stale cleanup

### F1.7.1 Ownership marker

- [ ] Define marker filename, for example `.screenshot-ocr-owned`.
- [ ] Marker content must be static and content-free.
- [ ] Write marker after creating capture directory and setting permissions.
- [ ] Validate marker before cleanup/scavenging.
- [ ] Do not include username, screenshot text, paths, command args, or timestamps if unnecessary.

### F1.7.2 Permission failure cleanup

- [ ] If directory creation succeeds but permission-setting fails, remove the just-created directory.
- [ ] Add unit test for simulated permission failure where practical.
- [ ] Surface cleanup failure safely if directory cannot be removed.

### F1.7.3 Stale artifact scavenging

- [ ] Define allowed base directory: preferably `XDG_RUNTIME_DIR`, fallback temp only with strict path rules.
- [ ] Identify stale dirs only by prefix plus ownership marker plus private permission expectations.
- [ ] Do not recurse into arbitrary directories.
- [ ] Do not follow symlinks.
- [ ] Do not delete anything lacking a valid marker.
- [ ] Bound number of directories scanned/deleted.
- [ ] Count or report cleanup failures without paths.
- [ ] Run scavenger at startup or before capture.

### F1.7.4 Tests

- [ ] Test marker creation.
- [ ] Test scavenger removes owned stale directory.
- [ ] Test scavenger refuses directory without marker.
- [ ] Test scavenger refuses symlink tricks.
- [ ] Test scavenger refuses nonmatching prefix.
- [ ] Test cleanup failure increments safe count or returns safe error.

### F1.7 acceptance gate

- [ ] GNOME temp cleanup has an ownership strategy, or incomplete pieces are explicitly deferred and left unchecked in v0.1 TODO.

## 12. F1.8 — Portal lifecycle review

### F1.8.1 Review `ashpd` lifecycle capabilities

- [ ] Determine whether outstanding portal screenshot requests can be closed/cancelled explicitly.
- [ ] Determine whether returned screenshot files are application-owned, portal-owned, or desktop-managed.
- [ ] Record findings in code comments or docs.

### F1.8.2 Implement supported cleanup

- [ ] Close/cancel portal requests on cancellation/timeout if supported.
- [ ] Delete returned file after decode only if application ownership is clear.
- [ ] If not deleting, document why deletion would be unsafe or incorrect.
- [ ] Never expose portal URI/path in errors, diagnostics, logs, or events.

### F1.8.3 Tests

- [ ] Test portal file URI parsing still rejects non-file schemes.
- [ ] Test portal symlink rejection still passes.
- [ ] Test portal oversized file rejection still passes.
- [ ] Add test for lifecycle decision helper if one is introduced.

### F1.8 acceptance gate

- [ ] Portal lifecycle behavior is explicit and privacy-safe.

## 13. F1.9 — Frontend production/test boundary

### F1.9.1 Move controller type

- [ ] Create production type helper in `src/app/controllerTypes.ts` or equivalent.
- [ ] Move `ReturnTypeOfController` out of `src/test/types.ts`.
- [ ] Update `CapturePanel` imports.
- [ ] Update `SettingsPanel` imports.
- [ ] Update `DiagnosticsPanel` imports if applicable.
- [ ] Leave `src/test` for test-only utilities.

### F1.9.2 Tests / lint

- [ ] Run TypeScript typecheck.
- [ ] Add or document grep/review rule preventing production imports from `src/test`.
- [ ] Ensure production build succeeds.

### F1.9 acceptance gate

- [ ] No production module imports from `src/test`.

## 14. F1.10 — Tray, shortcut, startup truthfulness

### F1.10.1 Tray menu consistency

- [ ] Compare tray menu against v0.1 TODO.
- [ ] Decide whether to add Settings tray action or explicitly defer it.
- [ ] If adding Settings action, route it to show main window and switch/open Settings if frontend route support exists.
- [ ] If deferring, document that tray Settings item is not implemented and leave TODO unchecked.

### F1.10.2 Shortcut and startup docs

- [ ] Ensure README says Wayland uses GNOME custom shortcut path.
- [ ] Ensure diagnostics direct shortcut status is accurate.
- [ ] Ensure X11 direct shortcut registration failure is visible in diagnostics.
- [ ] Ensure repeated shortcut/CLI invocations cannot overlap selectors.

### F1.10.3 Remove/disable startup and close-to-tray UI unless implemented

- [ ] Remove or disable start-at-login UI per F1.1.
- [ ] Remove or disable close-to-tray UI per F1.1.
- [ ] Document actual quit behavior.

### F1.10.4 Tests

- [ ] Rust test: unknown second-instance argument only shows window.
- [ ] Rust test: repeated action request while reserved returns `CaptureAlreadyActive` or equivalent.
- [ ] Rust test: active selector toggle cancels instead of starting second selector.
- [ ] Frontend test if Settings tray action route is implemented.

### F1.10 acceptance gate

- [ ] Tray/shortcut/startup surfaces do not overpromise.

## 15. F1.11 — Content-leakage guard and OCR fixture foundation

### F1.11.1 Synthetic secret fixture

- [ ] Define a distinctive synthetic secret string used only in tests.
- [ ] Ensure it is obviously fake.
- [ ] Use it in diagnostics/public-error/log redaction tests.

### F1.11.2 Log/content guard

- [ ] Add test helper to capture logs where practical.
- [ ] Verify public errors do not include synthetic secret.
- [ ] Verify diagnostics do not include synthetic secret.
- [ ] Verify stage events do not include synthetic secret.
- [ ] Verify settings recovery warning does not include raw corrupt file content.
- [ ] Grep first-party source for disallowed `console.log` or tracing of content-bearing values.

### F1.11.3 OCR fixture foundation

- [ ] Create docs for synthetic fixture generation, or add a small deterministic generator.
- [ ] Add at least one cleanup/punctuation fixture test that does not require real private screenshots.
- [ ] Separate real-Tesseract tests from pure cleanup/scoring tests.
- [ ] Ensure no private screenshot or real OCR output artifact is committed.

### F1.11.4 CI artifact safety

- [ ] Confirm CI uploads only reviewed package/checksum artifacts.
- [ ] Ensure no test screenshots/OCR outputs are uploaded.
- [ ] Document any generated fixture files and why they are safe.

### F1.11 acceptance gate

- [ ] There is automated protection against obvious content leakage regressions.

## 16. F1.12 — Public error-code and CI/security reconciliation

### F1.12.1 Public error-code mismatch

- [ ] Decide whether `clipboard_unavailable` is required.
- [ ] If required, add `ClipboardUnavailable` to Rust `ErrorCode` and `AppError` as appropriate.
- [ ] If not required, update TODO/spec references to remove or defer `clipboard_unavailable`.
- [ ] Keep `clipboard_write_failed` for actual write failure.
- [ ] Add tests for changed mappings.
- [ ] Ensure frontend handles both known and unknown public codes safely.

### F1.12.2 GitHub Actions pinning decision

- [ ] Decide whether to pin third-party actions by immutable commit SHA in FIX1.
- [ ] If yes, replace action tags with commit SHAs and document update procedure.
- [ ] If no, add status note that action-SHA pinning remains open.
- [ ] Do not check the v0.1 TODO action-pinning item unless actually pinned.

### F1.12.3 Dependency/security review foundation

- [ ] Decide whether to add dependency review/security tooling in FIX1.
- [ ] If added, keep workflow permissions least-privilege.
- [ ] If deferred, document deferral and leave v0.1 TODO item unchecked.

### F1.12 acceptance gate

- [ ] Public error-code checklist and CI/security TODO state are honest.

## 17. F1.13 — Documentation and authoritative TODO reconciliation

### F1.13.1 README updates

- [ ] Update Settings/feature descriptions to match implemented or disabled controls.
- [ ] Clarify GNOME Wayland shortcut setup.
- [ ] Clarify actual tray behavior.
- [ ] Clarify unimplemented/deferred autostart, notifications, or close-to-tray if applicable.
- [ ] Keep pre-release status clear.

### F1.13.2 Privacy and threat model updates

- [ ] Update `docs/PRIVACY.md` for clipboard-failure recovery behavior.
- [ ] Update `docs/PRIVACY.md` for corrupt-settings recovery warning.
- [ ] Update `docs/PRIVACY.md` for GNOME stale cleanup/marker behavior if implemented.
- [ ] Update `docs/THREAT_MODEL.md` for bounded Tesseract probing.
- [ ] Update `docs/THREAT_MODEL.md` for settings recovery behavior.
- [ ] Update `docs/THREAT_MODEL.md` for stage event privacy if implemented.

### F1.13.3 Architecture updates

- [ ] Document any new stage-event DTO.
- [ ] Document any new capture command outcome DTO.
- [ ] Document any new settings load result DTO.
- [ ] Document portal lifecycle decision.
- [ ] Document production/test type-boundary rule.

### F1.13.4 v0.1 TODO reconciliation

- [ ] For each item fixed in FIX1, update `docs/SCREENSHOT_OCR_V0_1_TODO.md` only if the task-state rules are satisfied.
- [ ] Add evidence notes with commit SHA and CI run where appropriate.
- [ ] Leave manual validation boxes unchecked unless real physical validation evidence exists.
- [ ] Leave aspirational/deferred items unchecked with clear notes.
- [ ] Do not delete incomplete tasks to make status look better.

### F1.13.5 Status report

- [ ] Create or update a status file summarizing FIX1 results.
- [ ] Include final commit SHA.
- [ ] Include CI run ID after CI passes.
- [ ] Include package artifact/checksum if package CI passes.
- [ ] List remaining release blockers.

### F1.13 acceptance gate

- [ ] Documentation matches implementation and does not overclaim release readiness.

## 18. F1.14 — Final validation and evidence

### F1.14.1 Local/CI validation commands

Run or confirm hosted CI covers:

- [ ] `npm ci`
- [ ] `npm run format:check`
- [ ] `npm run lint`
- [ ] `npm run typecheck`
- [ ] `npm run test`
- [ ] `npm run build`
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-features --locked`
- [ ] `npm run tauri -- build --bundles deb`

### F1.14.2 Hosted CI

- [ ] Push final FIX1 commit to `master`.
- [ ] Confirm repository-policy job passes.
- [ ] Confirm frontend-quality job passes.
- [ ] Confirm rust-quality job passes.
- [ ] Confirm Debian package smoke job passes.
- [ ] Record CI run ID.
- [ ] Record final commit SHA.
- [ ] Record package artifact ID if produced.
- [ ] Record package SHA-256 if produced.

### F1.14.3 Manual validation boundary

- [ ] Explicitly state that FIX1 did not complete Ubuntu 22.04/24.04 Wayland/X11 physical validation unless it actually did.
- [ ] Keep M15 manual validation tasks unchecked unless real evidence files exist.
- [ ] List recommended first physical validation environment.

### F1.14 acceptance gate

- [ ] Full automated validation is green on the exact final commit.
- [ ] No release-readiness claim exceeds the evidence.

## 19. Final FIX1 completion checklist

FIX1 is complete only when all boxes below are true:

- [ ] No active-looking UI controls remain for unimplemented notification/autostart/close-to-tray behavior.
- [ ] Settings save/reset/load errors are visible on Settings UI.
- [ ] Corrupt settings recovery is visible and content-free.
- [ ] Immediate-copy clipboard failure preserves OCR text and offers retry.
- [ ] Capture/OCR progress status is truthful.
- [ ] Tesseract language probe is bounded and cannot hang indefinitely.
- [ ] Image preprocessing after resize respects resource limits.
- [ ] Variant names match actual behavior.
- [ ] Production modules no longer import from `src/test`.
- [ ] GNOME temp ownership/stale cleanup is implemented or explicitly deferred with unchecked TODO state.
- [ ] Portal lifecycle behavior is reviewed and documented.
- [ ] Content-leakage guard tests exist.
- [ ] OCR fixture foundation exists without private screenshots.
- [ ] Public error-code mismatch is resolved.
- [ ] CI action-pinning/security-tooling decision is documented honestly.
- [ ] Documentation reflects reality.
- [ ] `docs/SCREENSHOT_OCR_V0_1_TODO.md` is reconciled only where evidence exists.
- [ ] Full CI passes on the final commit.
- [ ] Final status evidence is recorded.

## 20. Remaining after FIX1

Do not treat these as complete unless separate evidence exists:

- [ ] Ubuntu 22.04 GNOME Wayland physical validation.
- [ ] Ubuntu 22.04 GNOME X11 physical validation.
- [ ] Ubuntu 24.04 GNOME Wayland physical validation.
- [ ] Ubuntu 24.04 GNOME X11 physical validation.
- [ ] Real clipboard paste/ownership validation.
- [ ] Real tray behavior validation.
- [ ] Real GNOME custom shortcut validation.
- [ ] Real portal permission/cancellation validation.
- [ ] Multi-monitor validation.
- [ ] 125%, 150%, and 200% scaling validation.
- [ ] Package install/upgrade/reinstall/removal validation.
- [ ] Full OCR regression and accuracy evidence.
- [ ] Dependency/license/security review.
- [ ] Final v0.1 implementation report.
- [ ] Release tag/signoff.
