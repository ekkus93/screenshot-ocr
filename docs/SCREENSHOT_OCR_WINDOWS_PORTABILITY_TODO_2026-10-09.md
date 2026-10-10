# Screenshot OCR Windows Portability and Windows Application TODO

**Repository:** ekkus93/screenshot-ocr  
**Document:** docs/SCREENSHOT_OCR_WINDOWS_PORTABILITY_TODO_2026-10-09.md  
**Companion spec:** docs/SCREENSHOT_OCR_WINDOWS_PORTABILITY_SPEC_2026-10-09.md  
**Date:** 2026-10-09  
**Baseline branch:** ralph/windows-support  
**Baseline commit:** bbfd605d5c65b83735a9795622d92062c6c9c59d  
**Status:** authoritative implementation checklist for the portable-core refactor and Windows application port

## 1. Rules for this TODO

- [ ] means not implemented or not proven.
- [x] means implemented, tested, documented, and proven by required evidence.
- Do not check a parent item merely because some child code exists.
- Keep this TODO authoritative for this Windows-portability effort.
- Work on ralph/windows-support.
- Do not modify master during Phase 1 or Phase 2.
- Do not create a new branch or pull request for each task or subtask.
- Do not merge this work to master until the entire required effort is complete and the repository owner explicitly approves final integration.
- Use the companion specification as the normative source when a checklist item is ambiguous.
- Preserve existing privacy, no-shell, bounded-process, cancellation, and content-leakage rules.
- Do not claim physical desktop validation from hosted CI.
- Do not commit private screenshots, OCR output, clipboard contents, user paths, or private Windows diagnostic data.
- Keep first-party source files within the repository line-limit policy.
- If Windows implementation repeatedly requires platform conditionals in portable orchestration, stop and repair the abstraction instead of normalizing the leak.

## 2. Target outcome

This TODO is complete when:

- Phase 1 has produced a genuinely platform-neutral shared Rust core while preserving Linux behavior;
- Phase 2 has implemented a qualified Windows x86-64 application using that core;
- Windows native capture, cancellation, shortcut, tray, clipboard, OCR runtime, packaging, and diagnostics work;
- Windows and Linux hosted CI are green on the exact final branch commit;
- required physical Windows acceptance has been performed and recorded;
- documentation is reconciled;
- the branch is merge-ready;
- master has remained untouched by this project unless and until the owner separately authorizes final integration.

## 3. Milestone map

~~~text
W0   Baseline, branch policy, and evidence
W1   Architecture inventory and portability classification

PHASE 1
W2   Portable-core boundary
W3   Platform service contracts
W4   Linux capture adapter extraction
W5   Linux environment and desktop extraction
W6   OCR runtime portability
W7   Diagnostics, settings, and error boundary audit
W8   Tauri/platform composition root
W9   Cargo and target-specific dependency cleanup
W10  Portable-core tests and architecture guards
W11  Linux regression qualification
W12  Phase 1 architecture audit and gate

PHASE 2
W13  Windows build foundation
W14  Windows capture feasibility decision
W15  Windows region-selection implementation
W16  Windows DPI and multi-monitor correctness
W17  Windows environment and desktop integration
W18  Windows Tesseract/tessdata distribution
W19  Windows diagnostics, errors, and privacy
W20  Windows packaging and installer
W21  Cross-platform CI
W22  Windows automated qualification
W23  Physical Windows desktop acceptance
W24  Documentation and evidence reconciliation
W25  Final merge-readiness gate
~~~

## 4. W0 — Baseline, branch policy, and evidence

### W0.1 Confirm branch identity

- [ ] Confirm ralph/windows-support exists.
- [ ] Confirm it was created from bbfd605d5c65b83735a9795622d92062c6c9c59d.
- [ ] Record current branch head before implementation begins.
- [ ] Confirm master has not been modified by this portability project.
- [ ] Read docs/SCREENSHOT_OCR_WINDOWS_PORTABILITY_SPEC_2026-10-09.md.
- [ ] Read docs/ARCHITECTURE.md.
- [ ] Read docs/adr/0002-cross-platform-activation-and-cancellation.md.
- [ ] Read current privacy and threat-model documents.
- [ ] Inspect current CI state for the baseline commit.

### W0.2 Preserve non-negotiable behavior

- [ ] Preserve one-active-capture ownership.
- [ ] Preserve Rust-authoritative cancellation.
- [ ] Preserve no-shell invocation.
- [ ] Preserve bounded Tesseract subprocesses.
- [ ] Preserve no OCR history.
- [ ] Preserve content-free diagnostics.
- [ ] Preserve no broad Tauri filesystem/shell capability.
- [ ] Preserve clipboard-failure recovery.
- [ ] Preserve settings recovery behavior.
- [ ] Preserve Linux GNOME and portal support.

### W0 acceptance gate

- [ ] Baseline and branch policy are verified.
- [ ] No implementation work has touched master.

## 5. W1 — Architecture inventory and portability classification

### W1.1 Inventory Rust modules

Classify every module as portable, platform contract/composition, Linux-specific, frontend/Tauri adapter, or mixed.

At minimum inspect:

- [ ] src-tauri/src/app.rs
- [ ] src-tauri/src/actions.rs
- [ ] src-tauri/src/cancellation.rs
- [ ] src-tauri/src/capture/mod.rs
- [ ] src-tauri/src/capture/environment.rs
- [ ] src-tauri/src/capture/gnome.rs
- [ ] src-tauri/src/capture/portal.rs
- [ ] src-tauri/src/commands.rs
- [ ] src-tauri/src/desktop.rs
- [ ] src-tauri/src/diagnostics.rs
- [ ] src-tauri/src/error.rs
- [ ] src-tauri/src/image_pipeline/
- [ ] src-tauri/src/models.rs
- [ ] src-tauri/src/ocr/
- [ ] src-tauri/src/settings.rs
- [ ] src-tauri/src/state.rs
- [ ] src-tauri/src/lib.rs
- [ ] src-tauri/src/main.rs

### W1.2 Inventory platform assumptions

- [ ] Find every reference to linux.
- [ ] Find every reference to GNOME.
- [ ] Find every reference to portal/ashpd.
- [ ] Find every DISPLAY/WAYLAND/XDG environment-variable read.
- [ ] Find every hard-coded executable search path.
- [ ] Find every hard-coded shortcut label.
- [ ] Find every cfg target_os use.
- [ ] Find every OS-specific error mapping.
- [ ] Find every platform-specific path/resource assumption.
- [ ] Find every platform-specific Cargo dependency.
- [ ] Find every Linux-only Tauri bundle setting.

### W1.3 Inventory frontend assumptions

- [ ] Identify visible Linux-only wording.
- [ ] Identify Linux-only capture backend choices.
- [ ] Identify shortcut copy that must vary by platform.
- [ ] Identify diagnostics labels that must vary by platform.
- [ ] Confirm OCR review/copy/settings/job-state UI is otherwise portable.

### W1.4 Produce architecture classification note

- [ ] Record the final classification in architecture docs or an implementation status note.
- [ ] Identify each mixed module that must be split or redirected.
- [ ] Avoid inventing abstractions for components already portable.

### W1 acceptance gate

- [ ] Every known OS-specific assumption has an owner and planned boundary.

# PHASE 1 — PORTABLE CORE REFACTOR

## 6. W2 — Portable-core boundary

### W2.1 Define the portable core

- [ ] Define which Rust modules constitute the portable core.
- [ ] Ensure job state/cancellation remain portable.
- [ ] Ensure application action routing remains portable.
- [ ] Ensure image pipeline remains portable.
- [ ] Ensure OCR cleanup/scoring remains portable.
- [ ] Ensure request/result models remain portable.
- [ ] Ensure settings schema/validation remain portable.
- [ ] Ensure public error policy remains portable.

### W2.2 Remove concrete Linux dependencies from orchestration

- [ ] app.rs or its successor no longer imports GnomeScreenshotBackend directly.
- [ ] app.rs or its successor no longer imports PortalScreenshotBackend directly.
- [ ] app.rs or its successor no longer invokes Linux EnvironmentProbe directly.
- [ ] app.rs or its successor no longer selects Linux backend IDs itself.
- [ ] app.rs or its successor no longer resolves Tesseract from Linux environment data.
- [ ] Shared orchestration consumes platform services/contracts instead.

### W2.3 Core purity audit

- [ ] Portable modules contain no GNOME helper names.
- [ ] Portable modules contain no XDG Portal implementation code.
- [ ] Portable modules contain no Windows API code.
- [ ] Portable modules contain no Linux desktop environment parsing.
- [ ] Portable modules contain no Windows registry/path logic.
- [ ] Portable modules contain no OS-specific screen-capture mechanism.
- [ ] Portable modules contain no platform-specific installer/resource paths.
- [ ] Any remaining cfg target_os in portable modules is justified and documented.

### W2 acceptance gate

- [ ] The portable core can be described without mentioning Linux or Windows implementation mechanisms.

## 7. W3 — Platform service contracts

### W3.1 Capture service contract

- [ ] Define a narrow platform capture interface.
- [ ] Reuse portable cancellation token semantics.
- [ ] Return portable captured-image data.
- [ ] Return stable backend identity.
- [ ] Return safe cleanup warning state.
- [ ] Keep arbitrary filesystem paths out of the contract.
- [ ] Keep shell/command fragments out of the contract.

### W3.2 Platform environment/capability contract

- [ ] Define only the capabilities shared orchestration actually needs.
- [ ] Do not expose raw environment variables to the portable core.
- [ ] Do not expose raw OS version strings unless needed.
- [ ] Keep diagnostic capability values content-free.

### W3.3 OCR runtime resolver contract

- [ ] Separate runtime discovery/resource resolution from shared OCR recognition.
- [ ] Define a safe internal runtime descriptor.
- [ ] Prevent frontend-supplied executable paths.
- [ ] Support Linux system runtime.
- [ ] Leave a clean Windows packaged-runtime implementation point.

### W3.4 Desktop integration boundary

- [ ] Decide which tray/shortcut/single-instance behavior remains directly portable through Tauri.
- [ ] Isolate only genuinely platform-specific setup/policy.
- [ ] Keep action semantics in the shared router.
- [ ] Keep cancellation semantics in shared state.

### W3.5 Contract tests

- [ ] Add fakes/test doubles for platform services where useful.
- [ ] Test capture success through fake service.
- [ ] Test capture failure mapping through fake service.
- [ ] Test cancellation through fake service.
- [ ] Test cleanup-warning propagation.
- [ ] Test OCR runtime unavailable mapping.

### W3 acceptance gate

- [ ] Platform contracts are narrow enough that Windows can implement them without duplicating application orchestration.

## 8. W4 — Linux capture adapter extraction

### W4.1 Move GNOME capture behind Linux adapter

- [ ] Preserve GNOME Screenshot behavior.
- [ ] Preserve safe executable discovery.
- [ ] Preserve private temp ownership.
- [ ] Preserve cleanup semantics.
- [ ] Preserve timeout/cancellation behavior.
- [ ] Preserve stable public error mapping.

### W4.2 Move portal capture behind Linux adapter

- [ ] Preserve XDG Portal area-selection policy.
- [ ] Preserve portal capability probing.
- [ ] Preserve cancellation behavior.
- [ ] Preserve portal result cleanup/privacy behavior.
- [ ] Keep ashpd types from leaking into portable modules.

### W4.3 Linux backend selection

- [ ] Move Auto/Gnome/Portal selection into Linux platform code or an equivalent Linux-specific service.
- [ ] Preserve current preference semantics.
- [ ] Preserve fail-closed behavior for explicitly requested unavailable backend.
- [ ] Keep portable orchestration unaware of GNOME versus portal mechanics.

### W4.4 Linux capture tests

- [ ] Preserve existing deterministic GNOME tests.
- [ ] Preserve existing deterministic portal tests.
- [ ] Add tests for Linux backend-selection service.
- [ ] Add test that portable orchestration accepts a generic captured image regardless of backend.

### W4 acceptance gate

- [ ] Linux capture is functionally unchanged but reachable only through the platform boundary.

## 9. W5 — Linux environment and desktop extraction

### W5.1 Environment probe

- [ ] Move Linux OS guard out of portable core.
- [ ] Move DISPLAY probing to Linux platform code.
- [ ] Move WAYLAND_DISPLAY probing to Linux platform code.
- [ ] Move XDG_CURRENT_DESKTOP/GNOME detection to Linux platform code.
- [ ] Move GNOME Screenshot executable discovery to Linux platform code.
- [ ] Return only required generic capability information to shared code.

### W5.2 Shortcut policy

- [ ] Preserve Linux X11 direct Super+Shift+O behavior.
- [ ] Preserve GNOME Wayland custom-shortcut guidance.
- [ ] Keep shortcut action routed through shared action router.
- [ ] Ensure Windows can later use Control+Shift+O without changing action semantics.

### W5.3 Tray/single-instance policy

- [ ] Preserve current tray actions.
- [ ] Preserve single-instance command allowlist.
- [ ] Keep unknown second-instance behavior safe.
- [ ] Keep quit cancellation/cleanup behavior shared.

### W5.4 Linux diagnostics

- [ ] Keep Linux-specific capability details in Linux diagnostic provider.
- [ ] Preserve content-free status strings.
- [ ] Preserve direct-shortcut registration status.
- [ ] Preserve capture backend capability status.

### W5 acceptance gate

- [ ] Shared application code no longer parses Linux desktop environment state.

## 10. W6 — OCR runtime portability

### W6.1 Separate discovery from execution

- [ ] Identify existing TesseractEngine responsibilities.
- [ ] Keep recognition/probe process control shared where feasible.
- [ ] Extract Linux executable/tessdata location policy.
- [ ] Introduce runtime descriptor/resolver if required.
- [ ] Keep subprocess timeout/cancellation behavior shared.
- [ ] Keep no-shell behavior shared.
- [ ] Keep bounded output behavior shared.

### W6.2 Linux runtime behavior

- [ ] Preserve current trusted executable-directory policy.
- [ ] Preserve missing-engine error.
- [ ] Preserve missing-English-language error.
- [ ] Preserve Debian runtime dependency behavior.

### W6.3 Windows readiness

- [ ] Confirm the shared engine can accept a packaged Windows executable/runtime descriptor without Linux assumptions.
- [ ] Confirm Windows path separators do not enter OCR policy logic.
- [ ] Confirm tessdata location can be supplied internally without frontend control.
- [ ] Add unit tests using synthetic runtime descriptors where practical.

### W6 acceptance gate

- [ ] Shared OCR recognition code is reusable by Linux and Windows without OS branching.

## 11. W7 — Diagnostics, settings, and error boundary audit

### W7.1 Diagnostics

- [ ] Separate platform-specific probes from shared diagnostic aggregation.
- [ ] Keep cleanup failure counters shared.
- [ ] Keep safe last-error tracking shared.
- [ ] Add generic platform capability fields only when useful.
- [ ] Prevent raw platform error text from escaping.

### W7.2 Settings

- [ ] Review CaptureBackendPreference portability.
- [ ] Decide how Linux-only stored preferences behave on Windows.
- [ ] Ensure Auto remains valid cross-platform.
- [ ] Ensure unsupported persisted values recover safely or are mapped truthfully.
- [ ] Preserve schema migration/recovery behavior.
- [ ] Add tests for loading Linux-origin settings on Windows-like platform service context if applicable.

### W7.3 Error taxonomy

- [ ] Review every AppError/ErrorCode for platform assumptions.
- [ ] Keep generic errors generic.
- [ ] Add platform-specific internal variants only where necessary.
- [ ] Map platform-native failures to stable content-free public errors.
- [ ] Avoid embedding paths or raw OS messages.

### W7 acceptance gate

- [ ] Diagnostics/settings/errors can serve both platforms without misleading platform assumptions.

## 12. W8 — Tauri/platform composition root

### W8.1 Platform selection

- [ ] Select Linux implementation at compile time on Linux.
- [ ] Select Windows implementation point at compile time on Windows.
- [ ] Keep platform selection in one narrow composition area.
- [ ] Avoid scattered cfg target_os blocks.

### W8.2 AppServices construction

- [ ] Inject or construct platform services in AppServices or equivalent root.
- [ ] Keep public Tauri command surface stable where possible.
- [ ] Preserve frontend DTO compatibility unless a justified change is required.
- [ ] Preserve before/after capture window lifecycle contract.

### W8.3 Compile proof

- [ ] Linux build compiles after composition-root refactor.
- [ ] Test configuration can construct fake platform services.
- [ ] No Windows implementation is required yet except minimal stubs needed to prove architecture if chosen.

### W8 acceptance gate

- [ ] Platform selection is localized and application orchestration is platform-agnostic.

## 13. W9 — Cargo and target-specific dependency cleanup

### W9.1 Linux target dependencies

- [ ] Move ashpd under cfg(target_os = "linux") dependencies.
- [ ] Review any other Linux-only crates.
- [ ] Keep Linux Tauri system requirements documented.
- [ ] Ensure Linux tests still compile with locked dependencies.

### W9.2 Shared dependencies

- [ ] Confirm image remains shared.
- [ ] Confirm tokio remains shared.
- [ ] Confirm serde remains shared.
- [ ] Confirm Tauri plugins used on both platforms remain shared.
- [ ] Avoid adding a dependency merely to abstract a trivial type.

### W9.3 Future Windows dependency slot

- [ ] Document preferred target-specific Windows dependency section.
- [ ] Do not add large Windows bindings until the Phase 2 capture decision requires them.
- [ ] Keep Cargo.lock deterministic.

### W9 acceptance gate

- [ ] A Windows target no longer inherits Linux-only Rust dependencies by default.

## 14. W10 — Portable-core tests and architecture guards

### W10.1 Shared orchestration tests

- [ ] Fake capture success.
- [ ] Fake capture cancellation.
- [ ] Fake capture unavailable.
- [ ] Fake capture cleanup warning.
- [ ] Fake OCR runtime unavailable.
- [ ] Fake OCR language missing.
- [ ] One-active-job ownership.
- [ ] Cancellation during OCR.
- [ ] Candidate selection unchanged.
- [ ] Clipboard recovery contract unchanged.

### W10.2 Architecture guard

Implement at least one practical guard:

- [ ] Add source/dependency guard for Linux implementation imports in portable modules; or
- [ ] Add module visibility rules that make such imports impossible; or
- [ ] Add a portable crate boundary if justified; or
- [ ] Add equivalent compile-time/static validation.

The guard should cover at least:

- [ ] GNOME implementation imports.
- [ ] ashpd types.
- [ ] Linux environment parsing in portable core.
- [ ] Windows API imports in portable core once Windows code exists.

### W10.3 Content-leakage guard

- [ ] Ensure refactor does not weaken scripts/check-content-leakage.py.
- [ ] Add new platform fields to leakage review if necessary.
- [ ] Keep synthetic secret tests passing.

### W10 acceptance gate

- [ ] Tests protect both behavior and the new architectural boundary.

## 15. W11 — Linux regression qualification

### W11.1 Automated quality

- [ ] npm ci
- [ ] npm run format:check
- [ ] npm run lint
- [ ] npm run typecheck
- [ ] npm run test
- [ ] npm run build
- [ ] cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
- [ ] cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets --locked -- -D warnings
- [ ] cargo test --manifest-path src-tauri/Cargo.toml --workspace --locked
- [ ] cargo build --manifest-path src-tauri/Cargo.toml --workspace --locked
- [ ] python3 scripts/check-content-leakage.py

### W11.2 Debian packaging

- [ ] Build .deb using the same intended release path.
- [ ] Confirm Debian dependency metadata still includes required Linux runtime packages.
- [ ] Inspect package metadata.
- [ ] Generate checksum where existing process expects it.

### W11.3 Linux behavior evidence

Where physical Linux acceptance evidence already exists, ensure the refactor has not invalidated its assumptions.

If physical revalidation is performed:

- [ ] GNOME Wayland capture.
- [ ] GNOME X11 capture.
- [ ] cancellation.
- [ ] tray actions.
- [ ] shortcut policy.
- [ ] clipboard.
- [ ] multi-monitor/scaling as appropriate.

Do not invent physical evidence if it was not run.

### W11 acceptance gate

- [ ] Linux automated validation is green on the exact Phase 1 head.
- [ ] No known Linux regression remains.

## 16. W12 — Phase 1 architecture audit and gate

### W12.1 Core audit

- [ ] Portable core contains no concrete Linux capture implementation.
- [ ] Portable core contains no Linux environment-variable parsing.
- [ ] Portable core contains no hard-coded Linux helper path policy.
- [ ] Shared OCR engine accepts platform-resolved runtime.
- [ ] Platform composition is localized.
- [ ] Linux dependencies are target-scoped.
- [ ] Shared frontend behavior is unchanged except truthful portability adjustments.

### W12.2 Documentation

- [ ] Update docs/ARCHITECTURE.md.
- [ ] Add/update ADR if the platform-service boundary is a material architectural decision.
- [ ] Record Phase 1 exact commit SHA.
- [ ] Record Phase 1 CI run IDs.
- [ ] Record any explicitly deferred issue.

### W12.3 Hard Phase 1 gate

- [ ] All W0-W12 required tasks are complete or explicitly documented as non-blocking.
- [ ] Linux is green.
- [ ] Portable boundary is reviewable.
- [ ] Windows work can proceed without redesigning the core.

# PHASE 2 — WINDOWS APPLICATION

## 17. W13 — Windows build foundation

### W13.1 Compile target

- [ ] Add target-specific Windows dependencies only as needed.
- [ ] Ensure Rust code compiles on a Windows runner.
- [ ] Ensure frontend builds on Windows.
- [ ] Ensure Tauri app links on Windows.
- [ ] Resolve any path-separator assumptions.
- [ ] Resolve any Unix-only std::os imports.
- [ ] Resolve any Linux-only build-script assumptions.

### W13.2 Windows product metadata

- [ ] Update package description so it is no longer Linux-only once Windows support is real.
- [ ] Add Windows icon resources as needed.
- [ ] Verify application identifier remains appropriate.
- [ ] Verify executable name.
- [ ] Keep version source consistent.

### W13.3 Minimal Windows startup

- [ ] Launch application on Windows without capture implementation crashing startup.
- [ ] Show main window.
- [ ] Load settings.
- [ ] Show safe diagnostics.
- [ ] Quit cleanly.

### W13 acceptance gate

- [ ] A Windows build launches using the shared core and platform composition.

## 18. W14 — Windows capture feasibility decision

### W14.1 Evaluate native approaches

Evaluate at least the viable approaches relevant to the current Windows support target:

- [ ] Windows Graphics Capture.
- [ ] GDI/BitBlt.
- [ ] DXGI Desktop Duplication.
- [ ] Any other strongly justified API.

For each record:

- [ ] arbitrary rectangular region feasibility;
- [ ] Windows 10/11 support;
- [ ] mixed-DPI handling;
- [ ] multi-monitor handling;
- [ ] negative virtual coordinates;
- [ ] cancellation model;
- [ ] overlay interaction;
- [ ] pixel format conversion;
- [ ] dependency footprint;
- [ ] security/maintenance implications.

### W14.2 Choose mechanism

- [ ] Select one primary capture mechanism.
- [ ] Decide whether a fallback is necessary.
- [ ] Document why.
- [ ] Add ADR or architecture note.
- [ ] Define testable coordinate contract.

### W14 acceptance gate

- [ ] Capture implementation is chosen based on evidence rather than convenience alone.

## 19. W15 — Windows region-selection implementation

### W15.1 Selector overlay

- [ ] Create selector without using external screenshot applications.
- [ ] Cover intended virtual desktop area.
- [ ] Show visible selection state.
- [ ] Support click-drag rectangle.
- [ ] Support Escape cancellation.
- [ ] Handle mouse release outside origin monitor if supported.
- [ ] Reject zero-size selection.
- [ ] Define minimum safe selection.
- [ ] Ensure selector never remains orphaned.

### W15.2 Capture pixels

- [ ] Hide/exclude application UI appropriately.
- [ ] Capture selected pixels.
- [ ] Convert into shared image type.
- [ ] Preserve orientation.
- [ ] Preserve expected alpha/color behavior.
- [ ] Avoid unnecessary disk writes.
- [ ] If disk is unavoidable, use private owned temp storage and existing cleanup rules.

### W15.3 Cancellation and failure

- [ ] Shared cancel action dismisses selector.
- [ ] Escape maps to CaptureCancelled.
- [ ] Native API failure maps to stable safe error.
- [ ] Window restoration occurs after cancel/failure.
- [ ] Job ownership remains held until native resources close.
- [ ] Rapid repeated activations cannot create multiple selectors.

### W15.4 Tests

- [ ] Unit-test region normalization.
- [ ] Unit-test invalid rectangle rejection.
- [ ] Unit-test negative coordinates.
- [ ] Unit-test overflow/resource bounds.
- [ ] Unit-test native error mapping where possible.

### W15 acceptance gate

- [ ] Windows produces a valid shared captured image without bypassing the core.

## 20. W16 — Windows DPI and multi-monitor correctness

### W16.1 Coordinate model

- [ ] Explicitly distinguish logical and physical coordinates.
- [ ] Document which coordinate space selector events use.
- [ ] Document which coordinate space capture API uses.
- [ ] Centralize conversion.
- [ ] Avoid scattered scale-factor math.

### W16.2 DPI tests

- [ ] Automated pure test for 100%.
- [ ] Automated pure test for 125%.
- [ ] Automated pure test for 150%.
- [ ] Automated pure test for 200%.
- [ ] Test fractional conversion rounding policy.
- [ ] Test edge coordinates.

### W16.3 Multi-monitor tests

- [ ] Test monitor right of primary.
- [ ] Test monitor left of primary.
- [ ] Test monitor above primary.
- [ ] Test negative virtual coordinates.
- [ ] Test different resolutions.
- [ ] Test mixed scale factors in pure coordinate logic.
- [ ] Define cross-monitor drag policy.
- [ ] Test cross-monitor selection if supported.

### W16 acceptance gate

- [ ] Coordinate conversion is deterministic and ready for physical validation.

## 21. W17 — Windows environment and desktop integration

### W17.1 Platform probe

- [ ] Implement Windows platform identity/capability probe.
- [ ] Avoid collecting unnecessary machine details.
- [ ] Detect capture capability.
- [ ] Detect packaged OCR runtime.
- [ ] Detect English language data.
- [ ] Feed safe statuses into diagnostics.

### W17.2 Shortcut

- [ ] Register Control+Shift+O where supported.
- [ ] Route it through shared action router.
- [ ] Record registration success/failure safely.
- [ ] Do not make registration failure fatal.
- [ ] Prevent repeat key events from launching overlapping captures.

### W17.3 Tray

- [ ] Capture text action.
- [ ] Cancel capture action.
- [ ] Show Screenshot OCR action.
- [ ] Quit action.
- [ ] All actions route through shared router.
- [ ] Quit cancels/cleans active work before process exit.

### W17.4 Single instance

- [ ] Second instance capture routes correctly.
- [ ] Second instance cancel routes correctly.
- [ ] Second instance show routes correctly.
- [ ] Second instance quit obeys cleanup.
- [ ] Unknown commands only show/focus.
- [ ] No arbitrary argument injection.

### W17.5 Window focus/lifecycle

- [ ] Main window hides at correct selection boundary.
- [ ] Main window restores after capture.
- [ ] Main window restores after recoverable clipboard failure.
- [ ] Main window does not steal focus from selector incorrectly.
- [ ] No hidden-window dead-end after failure.

### W17 acceptance gate

- [ ] Windows activation/cancellation uses the same shared application semantics as Linux.

## 22. W18 — Windows Tesseract/tessdata distribution

### W18.1 Choose runtime source/version

- [ ] Select approved Windows Tesseract distribution/source.
- [ ] Pin version.
- [ ] Record download/provenance process.
- [ ] Record SHA-256 or equivalent integrity data for build input where practical.
- [ ] Review license obligations.
- [ ] Review Leptonica/native dependency licenses.

### W18.2 Bundle runtime

- [ ] Bundle tesseract executable.
- [ ] Bundle required DLLs.
- [ ] Bundle eng.traineddata.
- [ ] Place resources in application-controlled location.
- [ ] Avoid current-working-directory DLL search hazards.
- [ ] Resolve runtime path internally.
- [ ] Do not require PATH modification.
- [ ] Do not require separate Tesseract installation.

### W18.3 Shared engine integration

- [ ] Windows resolver returns shared runtime descriptor.
- [ ] Shared bounded probe works on Windows.
- [ ] Shared OCR recognition works on Windows.
- [ ] Cancellation kills/reaps Tesseract on Windows.
- [ ] Missing executable maps safely.
- [ ] Missing eng data maps safely.
- [ ] Raw process output is not logged.

### W18.4 Offline proof

- [ ] Demonstrate OCR works without network access.
- [ ] Ensure runtime does not download language data at first use.
- [ ] Document offline behavior.

### W18 acceptance gate

- [ ] A clean Windows install can OCR English text locally without manual OCR-engine installation.

## 23. W19 — Windows diagnostics, errors, and privacy

### W19.1 Diagnostics

- [ ] Add Windows capture backend status.
- [ ] Add packaged OCR runtime status.
- [ ] Add English language status.
- [ ] Add shortcut status.
- [ ] Add tray status if useful.
- [ ] Keep statuses content-free.

### W19.2 Public errors

- [ ] Map selector cancelled.
- [ ] Map capture unavailable.
- [ ] Map native capture failure.
- [ ] Map OCR runtime unavailable.
- [ ] Map OCR language missing.
- [ ] Map clipboard unavailable/write failure.
- [ ] Map internal platform failures without raw OS text.

### W19.3 Privacy review

- [ ] No screenshot pixels in logs.
- [ ] No OCR text in logs.
- [ ] No clipboard text in logs.
- [ ] No user profile paths in diagnostics.
- [ ] No install paths in public errors.
- [ ] No raw Windows error strings in public errors.
- [ ] No private capture artifacts in CI.
- [ ] Synthetic fixtures only.

### W19.4 Native DLL/runtime threat review

- [ ] Document DLL search strategy.
- [ ] Document packaged runtime trust/provenance.
- [ ] Confirm application does not load DLLs from arbitrary current directory.
- [ ] Update threat model.

### W19 acceptance gate

- [ ] Windows adds no new known content-leakage or shell-execution path.

## 24. W20 — Windows packaging and installer

### W20.1 Tauri bundle configuration

- [ ] Keep Debian bundle path working on Linux.
- [ ] Add Windows NSIS bundle path.
- [ ] Ensure Linux package dependency metadata does not affect Windows.
- [ ] Ensure Windows resources do not affect Linux package.
- [ ] Verify icons.
- [ ] Verify product/version metadata.

### W20.2 WebView2 policy

- [ ] Verify supported Windows WebView2 assumption.
- [ ] Decide bootstrap/download/offline installer policy.
- [ ] Document user impact truthfully.
- [ ] Ensure OCR itself has no network dependency.

### W20.3 Installer behavior

On clean Windows test system:

- [ ] Installer starts.
- [ ] Installation succeeds.
- [ ] App launches.
- [ ] Packaged Tesseract is present.
- [ ] English tessdata is present.
- [ ] Capture works.
- [ ] OCR works.
- [ ] Uninstall succeeds.
- [ ] App-owned binaries/resources are removed.
- [ ] User-settings retention/removal policy is documented.

### W20.4 Upgrade/reinstall

- [ ] Reinstall same version behavior is understood.
- [ ] Upgrade from prior Windows build is tested once a prior build exists.
- [ ] Settings remain valid or recover safely.
- [ ] Bundled OCR runtime upgrades cleanly.

### W20.5 Artifact integrity

- [ ] Generate installer SHA-256.
- [ ] Use stable artifact naming.
- [ ] Upload only reviewed installer/checksum artifacts.
- [ ] Do not claim code signing unless actually signed.

### W20 acceptance gate

- [ ] Windows can be installed and used without developer tools or manual Tesseract setup.

## 25. W21 — Cross-platform CI

### W21.1 Branch triggers

- [ ] Ensure meaningful CI runs on ralph/windows-support pushes.
- [ ] Preserve master CI behavior.
- [ ] Avoid cancelling unrelated platform jobs incorrectly.

### W21.2 Linux jobs

- [ ] Repository policy.
- [ ] Content-leakage guard.
- [ ] Frontend quality.
- [ ] Layout regression where applicable.
- [ ] Rust formatting.
- [ ] Rust Clippy.
- [ ] Rust tests.
- [ ] Rust build.
- [ ] Debian packaging path remains valid.

### W21.3 Windows jobs

- [ ] Set up Node.
- [ ] Set up pinned Rust.
- [ ] npm ci.
- [ ] Frontend build or required shared frontend checks.
- [ ] cargo fmt check or shared formatting gate.
- [ ] cargo clippy for Windows-relevant targets.
- [ ] cargo test.
- [ ] cargo build.
- [ ] Tauri Windows build.
- [ ] NSIS installer build.
- [ ] Verify bundled OCR files.
- [ ] Generate SHA-256.
- [ ] Upload installer/checksum artifact.

### W21.4 Exact-head qualification

- [ ] Record Linux run IDs for final commit.
- [ ] Record Windows run IDs for final commit.
- [ ] Confirm every required run head SHA equals the final branch SHA.
- [ ] Diagnose ordinary CI failures rather than treating them as user blockers.

### W21 acceptance gate

- [ ] Both platform CI paths are green on the same exact commit.

## 26. W22 — Windows automated qualification

### W22.1 Rust tests

- [ ] Portable orchestration tests pass on Windows.
- [ ] Windows coordinate tests pass.
- [ ] Windows runtime resolver tests pass.
- [ ] Windows error-mapping tests pass.
- [ ] Cancellation/state-machine tests pass.
- [ ] Settings tests pass.

### W22.2 Frontend tests

- [ ] Platform-specific shortcut label tests.
- [ ] Platform-specific capture backend choice tests.
- [ ] Shared review/copy tests.
- [ ] Shared cancellation UI tests.
- [ ] Shared settings error tests.
- [ ] No Linux-only copy shown as actionable Windows behavior.

### W22.3 Build tests

- [ ] Debug Windows build.
- [ ] Release Windows build.
- [ ] Installer build.
- [ ] Packaged resource audit.
- [ ] No unresolved Linux library dependency.

### W22 acceptance gate

- [ ] Hosted automated evidence supports a physical Windows acceptance candidate.

## 27. W23 — Physical Windows desktop acceptance

Hosted CI cannot check these boxes.

### W23.1 Basic Windows 11 x86-64

- [ ] Clean install.
- [ ] Launch.
- [ ] Capture via UI.
- [ ] OCR success.
- [ ] Clipboard success.
- [ ] Preview-before-copy behavior.
- [ ] Cancel selection with Escape.
- [ ] Cancel via app action where applicable.
- [ ] Capture again.
- [ ] Quit.
- [ ] Uninstall.

### W23.2 Shortcut/tray/single-instance

- [ ] Control+Shift+O starts capture.
- [ ] Repeated shortcut cannot create overlapping selectors.
- [ ] Tray Capture text works.
- [ ] Tray Cancel capture works.
- [ ] Tray Show works.
- [ ] Tray Quit cleans active capture.
- [ ] Second-instance capture works.
- [ ] Unknown second-instance command is safe.

### W23.3 Scaling

- [ ] 100%.
- [ ] 125%.
- [ ] 150%.
- [ ] 200%.
- [ ] Selection rectangle corresponds to captured pixels.
- [ ] Edge selections are accurate.

### W23.4 Multi-monitor

- [ ] Two monitors.
- [ ] Secondary monitor right of primary.
- [ ] Secondary monitor left of primary or another negative-coordinate arrangement.
- [ ] Mixed resolution.
- [ ] Mixed DPI.
- [ ] Cross-monitor selection if supported.
- [ ] Explicitly documented behavior if cross-monitor selection is intentionally unsupported.

### W23.5 Content cases

Use non-sensitive test content.

- [ ] Light background/dark text.
- [ ] Dark background/light text.
- [ ] Browser text.
- [ ] Terminal/monospace text.
- [ ] Small text.
- [ ] Large text.
- [ ] Punctuation.
- [ ] Whitespace/terminal mode as applicable.

### W23.6 Failure recovery

- [ ] OCR engine missing/corrupt simulation if safely testable.
- [ ] Language data missing/corrupt simulation if safely testable.
- [ ] Clipboard failure recovery if safely injectable.
- [ ] Selector cancellation restores UI.
- [ ] Native capture failure restores UI.
- [ ] No orphan overlay/window.
- [ ] No user content appears in diagnostics/logs.

### W23.7 Windows 10 22H2

If Windows 10 remains in the declared support matrix:

- [ ] Install.
- [ ] Launch.
- [ ] Capture.
- [ ] OCR.
- [ ] Shortcut.
- [ ] Tray.
- [ ] Uninstall.

If Windows 10 support is dropped because the chosen API/toolchain cannot support it reliably:

- [ ] Document the evidence.
- [ ] Update spec/support matrix before release.
- [ ] Do not silently claim Windows 10 support.

### W23 acceptance gate

- [ ] Required physical Windows evidence is recorded against the exact tested commit.

## 28. W24 — Documentation and evidence reconciliation

### W24.1 Architecture docs

- [ ] Update docs/ARCHITECTURE.md with portable core/platform adapters.
- [ ] Add/update platform architecture ADR.
- [ ] Add Windows capture API ADR.
- [ ] Document coordinate/DPI model.
- [ ] Document OCR runtime resolution.

### W24.2 User docs

- [ ] Update README with Windows installation.
- [ ] Update README with Control+Shift+O.
- [ ] Update README with tray behavior.
- [ ] Update README with offline OCR statement.
- [ ] Separate Linux and Windows dependency/install instructions.
- [ ] Document Windows minimum version truthfully.
- [ ] Document unsigned installer status if applicable.

### W24.3 Privacy/security docs

- [ ] Update docs/PRIVACY.md.
- [ ] Update docs/THREAT_MODEL.md.
- [ ] Document bundled native runtime.
- [ ] Document DLL search/provenance decision.
- [ ] Document Windows temporary capture behavior if any.

### W24.4 CI/release docs

- [ ] Document Linux versus Windows hosted gates.
- [ ] Document physical acceptance boundary.
- [ ] Record final Linux CI run IDs.
- [ ] Record final Windows CI run IDs.
- [ ] Record installer artifact name.
- [ ] Record installer SHA-256.
- [ ] Record final branch commit SHA.
- [ ] Record remaining non-goal/deferred items.

### W24.5 TODO reconciliation

- [ ] Check only tasks supported by evidence.
- [ ] Leave unrun physical tests unchecked.
- [ ] Do not delete incomplete tasks to make the project look complete.
- [ ] Add evidence notes where useful.
- [ ] Re-read companion spec for untracked normative requirements.

### W24 acceptance gate

- [ ] Documentation and TODO state match the actual implementation and evidence.

## 29. W25 — Final merge-readiness gate

### W25.1 Phase completion

- [ ] Phase 1 gate passed.
- [ ] Phase 2 implementation gate passed.
- [ ] Linux exact-head CI green.
- [ ] Windows exact-head CI green.
- [ ] Required Windows physical acceptance passed.
- [ ] Linux behavior has no known project-introduced regression.
- [ ] Windows installer qualified.
- [ ] Privacy/security review complete.
- [ ] Documentation reconciled.

### W25.2 Architecture final audit

- [ ] Portable core contains no concrete Linux implementation imports.
- [ ] Portable core contains no concrete Windows implementation imports.
- [ ] Platform selection is localized.
- [ ] One shared OCR pipeline remains.
- [ ] One shared capture job state machine remains.
- [ ] One shared cancellation model remains.
- [ ] One shared action router remains.
- [ ] Linux and Windows adapters implement mechanisms rather than duplicate policy.
- [ ] Target-specific dependencies are correctly scoped.

### W25.3 Branch/master policy

- [ ] Confirm all project commits are on ralph/windows-support.
- [ ] Confirm this project has not modified master.
- [ ] Record divergence from current master if master changed independently.
- [ ] Resolve/requalify divergence only if needed for final integration.
- [ ] Do not merge to master automatically.
- [ ] Present final merge-ready evidence to the repository owner.

### W25 final acceptance gate

- [ ] Every required checklist item is complete or explicitly deferred by scope with rationale.
- [ ] The exact final branch head is fully qualified.
- [ ] The branch is ready for owner-approved integration into master.

## 30. Explicitly deferred after this project

These remain unchecked/non-goal unless separately authorized:

- [ ] macOS implementation.
- [ ] Windows ARM64.
- [ ] Microsoft Store packaging.
- [ ] MSI if NSIS is sufficient.
- [ ] Automatic updater.
- [ ] Cloud OCR.
- [ ] Additional OCR languages beyond approved scope.
- [ ] Installer code signing if credentials are unavailable.
- [ ] Platform-specific visual redesign.
