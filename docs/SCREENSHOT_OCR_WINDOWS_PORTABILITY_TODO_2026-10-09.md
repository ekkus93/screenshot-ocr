# Screenshot OCR Windows Portability and Windows Application TODO

**Repository:** ekkus93/screenshot-ocr  
**Document:** docs/SCREENSHOT_OCR_WINDOWS_PORTABILITY_TODO_2026-10-09.md  
**Companion spec:** docs/SCREENSHOT_OCR_WINDOWS_PORTABILITY_SPEC_2026-10-09.md  
**Date:** 2026-10-09  
**Baseline branch:** ralph/windows-support  
**Baseline commit:** bbfd605d5c65b83735a9795622d92062c6c9c59d  
**Status:** authoritative implementation checklist for the portable-core refactor and Windows application port

**Windows packaging evidence (2026-10-10):** `ralph/windows-support` exact head `0ef4c25843a02e19553c91cff2267eeb3176bbc0` passed CI run `38092903351`, including Windows Clippy/tests/link, Linux tests/Debian package smoke, and Windows NSIS smoke. The smoke extracted the NSIS archive without executing it, checked packaged `tesseract.exe` and `tessdata/eng.traineddata` against the pinned manifest, and generated the installer SHA-256 in CI logs. The CI smoke does **not** yet validate every adjacent runtime DLL, publish the installer, complete transitive native-dependency/license review, or demonstrate physical Windows desktop behavior. Those items remain unchecked. Final exact-head qualification must be repeated after subsequent commits.

**Evidence reconciliation (2026-10-10):** Baseline master `bbfd605d5c65b83735a9795622d92062c6c9c59d` passed run `33818790571`. The Windows-portability branch passed Linux/frontend/Windows compile and Debian smoke CI at `d89c6e399a93f956e296a8628402f4e4ebae8bba` (run `38074489534`). The native Windows selector, bundled Tesseract installer, and physical Windows acceptance are not complete. This checklist reconciliation itself requires a fresh exact-head CI run.

**Hosted qualification evidence (2026-10-11):** CI `38106144905` passed at `d0efbbbf19d6833c7d17280661f0d37e9e05ea66`. Windows Rust native-error tests, frontend cancellation UI tests, and the NSIS resource inventory verification are implemented and exercised. The subsequent payload verifier hardening at `29f9487deeab289809483d9a4375983d2842cfb9` additionally rejects unexpected nested files; its exact-head CI requires requalification. Physical Windows and native redistribution-license gates remain unchecked.

**Hosted synthetic OCR qualification (2026-10-11):** Commit `55464854ff55fc7e94b245fbec0dc5b27827e442` passed CI run `38109521277` on its exact head. In addition to Linux/frontend/Windows compile and packaging gates, the Windows runner generated a synthetic English PNG, launched the checksum-verified staged Tesseract executable directly with a 20-second timeout, and verified recognized output. This proves a native runtime can execute English OCR in hosted Windows CI; it does **not** prove installed application integration, native region selection, offline installation, mixed-DPI desktop behavior, or redistribution-license compliance. The present documentation reconciliation commit requires a new exact-head CI pass.

**Preliminary Windows PE and redistribution review (2026-10-11):** `scripts/audit-windows-pe-imports.py` inventories PE32/PE32+ static and delay-loaded DLL imports from the staged Tesseract executable and adjacent DLLs, without executing the binaries. Synthetic parser tests and the staged-runtime audit are wired into CI. The native dependency classification and license/notice audit, physical offline Windows acceptance, and installer publication remain incomplete. Hosted exact-head CI must pass again after this reconciliation.

**Latest hosted qualification (2026-10-11):** `c34aa74a15f7720017821443bc86043adb1bf7cd` passed all seven jobs in CI run `38152659120`, including Windows NSIS extraction, Microsoft Authenticode validation of the bundled offline WebView2 installer, synthetic OCR from the packaged Tesseract runtime through the shared Rust engine, Linux/Windows Rust gates, frontend, and Debian packaging. Subsequent cleanup hardening changes remove both the NSIS payload extraction tree and the pinned source-installer download/extraction workspace even on validation failure, and clarify the extracted-payload test switch; they require fresh exact-head CI. This is **not** evidence of clean offline installation, physical Windows 10/11 desktop behavior, native dependency redistribution clearance, or owner-approved integration. All such checklist gates remain unchecked.

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

```text
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
```

## 4. W0 — Baseline, branch policy, and evidence

### W0.1 Confirm branch identity

- [x] Confirm ralph/windows-support exists.
- [x] Confirm it was created from bbfd605d5c65b83735a9795622d92062c6c9c59d.
- [x] Record current branch head before implementation begins.
- [x] Confirm master has not been modified by this portability project.
- [x] Read docs/SCREENSHOT_OCR_WINDOWS_PORTABILITY_SPEC_2026-10-09.md.
- [x] Read docs/ARCHITECTURE.md.
- [x] Read docs/adr/0002-cross-platform-activation-and-cancellation.md.
- [x] Read current privacy and threat-model documents.
- [x] Inspect current CI state for the baseline commit.

### W0.2 Preserve non-negotiable behavior

- [x] Preserve one-active-capture ownership.
- [x] Preserve Rust-authoritative cancellation.
- [x] Preserve no-shell invocation.
- [x] Preserve bounded Tesseract subprocesses.
- [x] Preserve no OCR history.
- [x] Preserve content-free diagnostics.
- [x] Preserve no broad Tauri filesystem/shell capability.
- [x] Preserve clipboard-failure recovery.
- [x] Preserve settings recovery behavior.
- [x] Preserve Linux GNOME and portal support.

### W0 acceptance gate

- [x] Baseline and branch policy are verified.
- [x] No implementation work has touched master.

## 5. W1 — Architecture inventory and portability classification

### W1.1 Inventory Rust modules

Classify every module as portable, platform contract/composition, Linux-specific, frontend/Tauri adapter, or mixed.

At minimum inspect:

- [x] src-tauri/src/app.rs
- [x] src-tauri/src/actions.rs
- [x] src-tauri/src/cancellation.rs
- [x] src-tauri/src/capture/mod.rs
- [x] src-tauri/src/capture/environment.rs
- [x] src-tauri/src/capture/gnome.rs
- [x] src-tauri/src/capture/portal.rs
- [x] src-tauri/src/commands.rs
- [x] src-tauri/src/desktop.rs
- [x] src-tauri/src/diagnostics.rs
- [x] src-tauri/src/error.rs
- [x] src-tauri/src/image_pipeline/
- [x] src-tauri/src/models.rs
- [x] src-tauri/src/ocr/
- [x] src-tauri/src/settings.rs
- [x] src-tauri/src/state.rs
- [x] src-tauri/src/lib.rs
- [x] src-tauri/src/main.rs

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

**W1.2 source inventory work (2026-10-11):** CI now generates a
content-free platform-assumption inventory artifact with path/line/category
references and architectural owner groups. Synthetic fixtures verify
classification, repeatability, and omission of source text. Exact-head CI
must qualify the new step; each category remains unchecked until the
generated findings have been reviewed for coverage and boundary ownership.
The inventory now fails closed on symlinked first-party source paths instead
of silently omitting them; generated dependency trees remain excluded, and
a synthetic privacy regression test runs in repository-policy CI. This does not constitute category-by-category
architecture review or physical desktop acceptance. The inventory now also
rejects symlinked scan roots and linked ancestor directories, with synthetic
privacy fixtures; the remaining category reviews are still open.

**Additional W1.2 source-coverage hardening (2026-10-11):** The
content-free inventory now recognizes Rust target-family/architecture
conditionals, `cfg!` and Unix/Windows aliases, Windows native bindings and
OS-error probes, absolute Windows executable paths, and installer resource,
batch, and native-source files. Synthetic fixtures verify these categories
without exposing matched source text. A caller-supplied symlinked checkout
root now fails closed instead of scanning external content. This remains a
keyword inventory, not a completed semantic review; all W1.2 category
checkmarks and the W1 acceptance gate stay open pending source review.
Exact-head hosted CI must qualify these changes.

**W1.2 checkout-path audit (2026-10-12):** The inventory now rejects
checkout paths traversing symlinked parent directories, not only a symlink
at the final checkout root. A synthetic fixture exercises this redirection.
This narrows a privacy boundary but does not complete semantic review of
platform assumptions. Exact-head CI is required before qualification.

### W1.3 Inventory frontend assumptions

- [x] Identify visible Linux-only wording.
- [x] Identify Linux-only capture backend choices.
- [x] Identify shortcut copy that must vary by platform.
- [x] Identify diagnostics labels that must vary by platform.
- [x] Confirm OCR review/copy/settings/job-state UI is otherwise portable.

### W1.4 Produce architecture classification note

- [x] Record the final classification in architecture docs or an implementation status note.
- [x] Identify each mixed module that must be split or redirected.
- [x] Avoid inventing abstractions for components already portable.

### W1 acceptance gate

- [ ] Every known OS-specific assumption has an owner and planned boundary.

# PHASE 1 — PORTABLE CORE REFACTOR

## 6. W2 — Portable-core boundary

### W2.1 Define the portable core

- [x] Define which Rust modules constitute the portable core.
- [x] Ensure job state/cancellation remain portable.
- [x] Ensure application action routing remains portable.
- [x] Ensure image pipeline remains portable.
- [x] Ensure OCR cleanup/scoring remains portable.
- [x] Ensure request/result models remain portable.
- [x] Ensure settings schema/validation remain portable.
- [x] Ensure public error policy remains portable.

### W2.2 Remove concrete Linux dependencies from orchestration

- [x] app.rs or its successor no longer imports GnomeScreenshotBackend directly.
- [x] app.rs or its successor no longer imports PortalScreenshotBackend directly.
- [x] app.rs or its successor no longer invokes Linux EnvironmentProbe directly.
- [x] app.rs or its successor no longer selects Linux backend IDs itself.
- [x] app.rs or its successor no longer resolves Tesseract from Linux environment data.
- [x] Shared orchestration consumes platform services/contracts instead.

### W2.3 Core purity audit

- [ ] Portable modules contain no GNOME helper names.
- [ ] Portable modules contain no XDG Portal implementation code.
- [ ] Portable modules contain no Windows API code.
- [ ] Portable modules contain no Linux desktop environment parsing.
- [ ] Portable modules contain no Windows registry/path logic.
- [ ] Portable modules contain no OS-specific screen-capture mechanism.
- [ ] Portable modules contain no platform-specific installer/resource paths.
- [ ] Any remaining cfg target_os in portable modules is justified and documented.

**W2.3 error-boundary hardening (2026-10-11):** Shared `error.rs`
defines stable capture error variants on every target without `target_os`
gating, and `scripts/check-platform-boundaries.py` rejects platform-dependent
error enum compilation. All seven CI jobs passed at exact commit
`a3370f3834c7a28d7d8652c8db87f9e830664ce8` (run `38158378805`);
other core-purity and physical acceptance items stay unchecked.

**W2.3 settings storage boundary (2026-10-11):** The portable settings
store delegates private directory/file protection to the platform-services
contract. Linux retains mode `0700` for its configuration directory and
`0600` for the settings file; the Windows adapter preserves the previous
app-config ACL-inheritance policy, without claiming a reviewed Windows ACL.
The unsupported adapter fails closed. A static source guard and synthetic
fixtures reject OS-specific settings-permission code in production shared
settings. This code and the Linux permission regression test require fresh
exact-head Linux/Windows CI. Broader core-purity and physical gates remain
unchecked.

### W2 acceptance gate

- [ ] The portable core can be described without mentioning Linux or Windows implementation mechanisms.

## 7. W3 — Platform service contracts

### W3.1 Capture service contract

- [x] Define a narrow platform capture interface.
- [x] Reuse portable cancellation token semantics.
- [x] Return portable captured-image data.
- [x] Return stable backend identity.
- [x] Return safe cleanup warning state.
- [x] Keep arbitrary filesystem paths out of the contract.
- [x] Keep shell/command fragments out of the contract.

### W3.2 Platform environment/capability contract

- [ ] Define only the capabilities shared orchestration actually needs.
- [ ] Do not expose raw environment variables to the portable core.
- [ ] Do not expose raw OS version strings unless needed.
- [ ] Keep diagnostic capability values content-free.

**W3.2 Linux diagnostic normalization (2026-10-12):** The Linux adapter
now maps `/etc/os-release` IDs to a fixed distribution-label allowlist and
accepts only bounded numeric version identifiers. Arbitrary `PRETTY_NAME`
or vendor-provided release text is no longer echoed to public diagnostics.
Synthetic tests cover known, unknown, and malformed labels. This narrows the
portable diagnostics contract; exact-head CI and broader W3.2 review remain
outstanding.

### W3.3 OCR runtime resolver contract

- [x] Separate runtime discovery/resource resolution from shared OCR recognition.
- [x] Define a safe internal runtime descriptor.
- [x] Prevent frontend-supplied executable paths.
- [x] Support Linux system runtime.
- [x] Leave a clean Windows packaged-runtime implementation point.

### W3.4 Desktop integration boundary

- [x] Decide which tray/shortcut/single-instance behavior remains directly portable through Tauri.
- [x] Isolate only genuinely platform-specific setup/policy.
- [x] Keep action semantics in the shared router.
- [x] Keep cancellation semantics in shared state.

### W3.5 Contract tests

- [x] Add fakes/test doubles for platform services where useful.
- [x] Test capture success through fake service.
- [x] Test capture failure mapping through fake service.
- [x] Test cancellation through fake service.
- [x] Test cleanup-warning propagation.
- [x] Test OCR runtime unavailable mapping.

**W2/W3 structural evidence (2026-10-11):** `docs/SCREENSHOT_OCR_WINDOWS_PORTABILITY_PHASE1_AUDIT_2026-10-09.md` classifies core/adapter modules; `src-tauri/src/app.rs` consumes injected `PlatformServices`, `src-tauri/src/capture/mod.rs` defines the portable capture contract, and `src-tauri/src/ocr/mod.rs` defines the internal runtime descriptor. `src-tauri/src/app_portability_tests.rs` supplies fake-platform success, failure, cancellation, cleanup-warning and unavailable-engine tests. Hosted Linux/Windows quality jobs have exercised the shared architecture. This is structural and deterministic-test evidence only; unreviewed purity cases and physical acceptance remain unchecked.

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

- [x] Move Auto/Gnome/Portal selection into Linux platform code or an equivalent Linux-specific service.
- [x] Preserve current preference semantics.
- [x] Preserve fail-closed behavior for explicitly requested unavailable backend.
- [x] Keep portable orchestration unaware of GNOME versus portal mechanics.

### W4.4 Linux capture tests

- [ ] Preserve existing deterministic GNOME tests.
- [ ] Preserve existing deterministic portal tests.
- [x] Add tests for Linux backend-selection service.
- [x] Add test that portable orchestration accepts a generic captured image regardless of backend.

**W4.3/W4.4 deterministic evidence (2026-10-11):** Linux adapter `src-tauri/src/platform/linux/capture/mod.rs` owns Auto/Gnome/Portal selection, probes portal capability before selection, and rejects explicitly requested unavailable backends. A complete 3-preference × 2-portal × 2-GNOME matrix test covers fallback and fail-closed semantics. `src-tauri/src/app_portability_tests.rs` exercises identical shared orchestration for GNOME, portal, and Windows-native fake captures. Physical GNOME/portal regression evidence remains outstanding; W4.1, W4.2 and the W4 acceptance gate remain unchecked.

### W4 acceptance gate

- [ ] Linux capture is functionally unchanged but reachable only through the platform boundary.

## 9. W5 — Linux environment and desktop extraction

### W5.1 Environment probe

- [x] Move Linux OS guard out of portable core.
- [x] Move DISPLAY probing to Linux platform code.
- [x] Move WAYLAND_DISPLAY probing to Linux platform code.
- [x] Move XDG_CURRENT_DESKTOP/GNOME detection to Linux platform code.
- [x] Move GNOME Screenshot executable discovery to Linux platform code.
- [x] Return only required generic capability information to shared code.

### W5.2 Shortcut policy

- [x] Preserve Linux X11 direct Super+Shift+O behavior.
- [x] Preserve GNOME Wayland custom-shortcut guidance.
- [x] Keep shortcut action routed through shared action router.
- [x] Ensure Windows can later use Control+Shift+O without changing action semantics.

### W5.3 Tray/single-instance policy

- [x] Preserve current tray actions.
- [x] Preserve single-instance command allowlist.
- [x] Keep unknown second-instance behavior safe.
- [x] Keep quit cancellation/cleanup behavior shared.

### W5.4 Linux diagnostics

- [x] Keep Linux-specific capability details in Linux diagnostic provider.
- [x] Preserve content-free status strings.
- [x] Preserve direct-shortcut registration status.
- [x] Preserve capture backend capability status.

**W5 source/hosted evidence (2026-10-11):** Linux desktop probing,
GNOME helper discovery, backend capability summaries and shortcut policy
reside under `src-tauri/src/platform/linux/`. `src-tauri/src/desktop.rs`
and `src-tauri/src/actions.rs` retain shared tray, safe second-instance
allowlist, bounded quit cancellation, and shortcut action routing. Unit tests
cover the X11-only shortcut selection and content-free Linux diagnostic
construction. This establishes code/CI preservation, not physical GNOME
Wayland/X11, tray or shortcut acceptance (W11.3 remains unchecked).

### W5 acceptance gate

- [x] Shared application code no longer parses Linux desktop environment state.

## 10. W6 — OCR runtime portability

### W6.1 Separate discovery from execution

- [x] Identify existing TesseractEngine responsibilities.
- [x] Keep recognition/probe process control shared where feasible.
- [x] Extract Linux executable/tessdata location policy.
- [x] Introduce runtime descriptor/resolver if required.
- [x] Keep subprocess timeout/cancellation behavior shared.
- [x] Keep no-shell behavior shared.
- [x] Keep bounded output behavior shared.

### W6.2 Linux runtime behavior

- [x] Preserve current trusted executable-directory policy.
- [x] Preserve missing-engine error.
- [x] Preserve missing-English-language error.
- [x] Preserve Debian runtime dependency behavior.

### W6.3 Windows readiness

- [x] Confirm the shared engine can accept a packaged Windows executable/runtime descriptor without Linux assumptions.
- [x] Confirm Windows path separators do not enter OCR policy logic.
- [x] Confirm tessdata location can be supplied internally without frontend control.
- [x] Add unit tests using synthetic runtime descriptors where practical.

**W6 source/hosted evidence (2026-10-11):** Shared
`src-tauri/src/ocr/tesseract.rs` handles bounded English probing,
subprocess execution and cancellation without shell invocation. Linux
executable resolution is restricted in
`src-tauri/src/platform/linux/environment.rs`; Windows supplies a packaged
`OcrRuntime` from `src-tauri/src/platform/windows/ocr_runtime.rs` using
`PathBuf` rather than frontend paths. Synthetic missing-engine/language
and native Windows staged OCR tests are present. The full Linux/Windows
hosted suite passed at `f3c866fadc1a0ad5d086c7407f70bd2d2dc77fbc`
(run `38131090350`); physical Windows offline operation remains unproven.

### W6 acceptance gate

- [x] Shared OCR recognition code is reusable by Linux and Windows without OS branching.

## 11. W7 — Diagnostics, settings, and error boundary audit

### W7.1 Diagnostics

- [x] Separate platform-specific probes from shared diagnostic aggregation.
- [x] Keep cleanup failure counters shared.
- [x] Keep safe last-error tracking shared.
- [x] Add generic platform capability fields only when useful.
- [x] Prevent raw platform error text from escaping.

### W7.2 Settings

- [x] Review CaptureBackendPreference portability.
- [x] Decide how Linux-only stored preferences behave on Windows.
- [x] Ensure Auto remains valid cross-platform.
- [x] Ensure unsupported persisted values recover safely or are mapped truthfully.
- [x] Preserve schema migration/recovery behavior.
- [x] Add tests for loading Linux-origin settings on Windows-like platform service context if applicable.

### W7.3 Error taxonomy

- [x] Review every AppError/ErrorCode for platform assumptions.
- [x] Keep generic errors generic.
- [x] Add platform-specific internal variants only where necessary.
- [x] Map platform-native failures to stable content-free public errors.
- [x] Avoid embedding paths or raw OS messages.

**W7 source/hosted evidence (2026-10-11):** Shared
`src-tauri/src/diagnostics.rs` aggregates platform-safe capability
summaries, error codes and cleanup counters. `src-tauri/src/settings.rs`
retains one schema, quarantines invalid values and preserves migrated Linux
capture preferences; the Windows adapter ignores Linux-specific backend
selection while `src/features/settings/SettingsPanel.tsx` labels native
capture and Ctrl+Shift+O truthfully. `src-tauri/src/error.rs` maps native
failures to static path-free public codes and guidance. Unit/frontend tests
cover migration, recovery and synthetic privacy cases. Physical Windows
acceptance still remains open under W23.

**W7 Windows settings read-leaf hardening (2026-10-12):** The shared
settings loader now invokes a platform preflight on the existing file before
reading it. Windows rejects native reparse-pointed settings leaves and
ancestors, even when Rust does not classify a leaf as a symbolic link.
Rejected leaves are quarantined without opening the redirected target.
Deterministic file/directory tests and a privilege-conditional symlink test
exercise the boundary. This is a pre-open mitigation, not an atomic no-follow
open or a reviewed Windows ACL claim; exact-head CI and physical acceptance
are still required.

**Additional settings-read hardening (2026-10-12):** Shared settings loading
now bounds JSON reads to 64 KiB plus one sentinel byte, quarantines oversized
profiles, and returns safe defaults through the existing frontend recovery
contract. A synthetic oversized-file regression test was added. This is
deterministic privacy/availability protection, not physical Windows
qualification; exact-head CI remains required.

### W7 acceptance gate

- [x] Diagnostics/settings/errors can serve both platforms without misleading platform assumptions.

## 12. W8 — Tauri/platform composition root

### W8.1 Platform selection

- [x] Select Linux implementation at compile time on Linux.
- [x] Select Windows implementation point at compile time on Windows.
- [x] Keep platform selection in one narrow composition area.
- [ ] Avoid scattered cfg target_os blocks.

### W8.2 AppServices construction

- [x] Inject or construct platform services in AppServices or equivalent root.
- [x] Keep public Tauri command surface stable where possible.
- [x] Preserve frontend DTO compatibility unless a justified change is required.
- [x] Preserve before/after capture window lifecycle contract.

### W8.3 Compile proof

- [x] Linux build compiles after composition-root refactor.
- [x] Test configuration can construct fake platform services.
- [ ] No Windows implementation is required yet except minimal stubs needed to prove architecture if chosen.

**W8 source/hosted evidence (2026-10-11):**
`src-tauri/src/platform/mod.rs` composes the Linux and Windows
`PlatformServices` implementations with target-specific cfg; shared
`AppServices` uses the injected trait and retains its Tauri DTO/action
surface. `src-tauri/src/commands.rs` maintains bounded window hide/restore
and clipboard recovery. Both platforms built and tested in run
`38131090350` at `f3c866fadc1a0ad5d086c7407f70bd2d2dc77fbc`.
An exhaustive cfg-inventory review remains open, and the Phase 1
"Windows stub only" item is superseded by the ongoing Phase 2 native port,
not represented as completed physical acceptance.

### W8 acceptance gate

- [x] Platform selection is localized and application orchestration is platform-agnostic.

## 13. W9 — Cargo and target-specific dependency cleanup

### W9.1 Linux target dependencies

- [x] Move ashpd under cfg(target_os = "linux") dependencies.
- [x] Review any other Linux-only crates.
- [x] Keep Linux Tauri system requirements documented.
- [x] Ensure Linux tests still compile with locked dependencies.

### W9.2 Shared dependencies

- [x] Confirm image remains shared.
- [x] Confirm tokio remains shared.
- [x] Confirm serde remains shared.
- [x] Confirm Tauri plugins used on both platforms remain shared.
- [x] Avoid adding a dependency merely to abstract a trivial type.

### W9.3 Future Windows dependency slot

- [x] Document preferred target-specific Windows dependency section.
- [ ] Do not add large Windows bindings until the Phase 2 capture decision requires them.
- [x] Keep Cargo.lock deterministic.

**W9 structural/CI evidence (2026-10-11):** `src-tauri/Cargo.toml` declares `ashpd` and `url` only under `cfg(target_os = "linux")`, and `windows-sys` only under `cfg(target_os = "windows")`; `image`, `tokio`, `serde`, and the shared Tauri plugins remain common dependencies. The Linux build-package prerequisites remain in `README.md`; the pinned `src-tauri/Cargo.lock` is checked by both platform CI jobs. Earlier exact-head CI `38118651508` at `ecbe361f9fb7cf395a5b8be17494cd4384a35912` passed Linux and Windows builds. This does not assert that the latest commit or physical platforms are qualified. The Phase 2 Windows binding was added only for the selected Win32 capture implementation.

### W9 acceptance gate

- [x] A Windows target no longer inherits Linux-only Rust dependencies by default.

## 14. W10 — Portable-core tests and architecture guards

### W10.1 Shared orchestration tests

- [x] Fake capture success.

- [x] Fake capture cancellation.
- [x] Fake capture unavailable.
- [x] Fake capture cleanup warning.
- [x] Fake OCR runtime unavailable.
- [x] Fake OCR language missing.
- [x] One-active-job ownership.
- [x] Cancellation during OCR.
- [x] Candidate selection unchanged.
- [x] Clipboard recovery contract unchanged.

### W10.2 Architecture guard

Implement at least one practical guard:

- [x] Add source/dependency guard for Linux implementation imports in portable modules; or
- [ ] Add module visibility rules that make such imports impossible; or
- [ ] Add a portable crate boundary if justified; or
- [ ] Add equivalent compile-time/static validation.

The guard should cover at least:

- [x] GNOME implementation imports.
- [x] ashpd types.
- [x] Linux environment parsing in portable core.
- [x] Windows API imports in portable core once Windows code exists.

**W10.2 guard evidence (2026-10-11):** `scripts/check-platform-boundaries.py` runs in repository-policy CI; `scripts/test_platform_boundaries.py` supplies synthetic positive and negative guard cases for platform import, environment probing, and composition-root exceptions. Repository-policy completed successfully in run `38121203139` at `167cb5652a6043c6affdefb05730fc9785d73795` (the overall run was later cancelled by a newer branch push). This does not close W10 overall or replace final exact-head qualification.

**Additional architecture-guard hardening (2026-10-12):** The Rust
platform-boundary scanner now rejects fake inline-test-module markers embedded
in multiline raw/ordinary strings or nested comments, and continues scanning
production code after a real inline test module. Synthetic fixtures exercise
both evasion cases. This improves static guard coverage but does not replace
the outstanding category-by-category W1.2 source review. Exact-head CI
qualification remains pending.

### W10.3 Content-leakage guard

- [x] Ensure refactor does not weaken scripts/check-content-leakage.py.
- [ ] Add new platform fields to leakage review if necessary.
- [x] Keep synthetic secret tests passing.

**W10 deterministic evidence (2026-10-11):**
`src-tauri/src/app_portability_tests.rs` exercises fake-platform success,
unavailable capture/runtime/language handling, capture and OCR cancellation,
cleanup warnings, and single-job ownership. `src-tauri/src/ocr/cleanup.rs`
tests candidate ranking; `src-tauri/src/commands.rs` and the frontend tests
cover clipboard validation and recoverable warnings. The CI repository-policy
job exercises synthetic import guards in `scripts/test_platform_boundaries.py`
and tests private-text logging attempts and content-free error findings in
`scripts/test_content_leakage.py`. The repository-policy job passed on
`1cf95c7168aaa56152977323fe8250502224fc0a` in run `38132797795`;
the earlier full Linux/Windows hosted suite passed on
`f3c866fadc1a0ad5d086c7407f70bd2d2dc77fbc` in run `38131090350`.
These are deterministic/hosted tests and do not close physical desktop gates.

### W10 acceptance gate

- [x] Tests protect both behavior and the new architectural boundary.

## 15. W11 — Linux regression qualification

### W11.1 Automated quality

- [x] npm ci
- [x] npm run format:check
- [x] npm run lint
- [x] npm run typecheck
- [x] npm run test
- [x] npm run build
- [x] cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
- [x] cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets --locked -- -D warnings
- [x] cargo test --manifest-path src-tauri/Cargo.toml --workspace --locked
- [x] cargo build --manifest-path src-tauri/Cargo.toml --workspace --locked
- [x] python3 scripts/check-content-leakage.py

### W11.2 Debian packaging

- [x] Build .deb using the same intended release path.
- [x] Confirm Debian dependency metadata still includes required Linux runtime packages.
- [x] Inspect package metadata.
- [x] Generate checksum where existing process expects it.

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

- [x] Portable core contains no concrete Linux capture implementation.
- [x] Portable core contains no Linux environment-variable parsing.
- [x] Portable core contains no hard-coded Linux helper path policy.
- [x] Shared OCR engine accepts platform-resolved runtime.
- [x] Platform composition is localized.
- [x] Linux dependencies are target-scoped.
- [x] Shared frontend behavior is unchanged except truthful portability adjustments.

### W12.2 Documentation

- [x] Update docs/ARCHITECTURE.md.
- [x] Add/update ADR if the platform-service boundary is a material architectural decision.
- [x] Record Phase 1 exact commit SHA.
- [x] Record Phase 1 CI run IDs.
- [x] Record any explicitly deferred issue.

### W12.3 Hard Phase 1 gate

- [ ] All W0-W12 required tasks are complete or explicitly documented as non-blocking.
- [ ] Linux is green.
- [ ] Portable boundary is reviewable.
- [ ] Windows work can proceed without redesigning the core.

# PHASE 2 — WINDOWS APPLICATION

## 17. W13 — Windows build foundation

### W13.1 Compile target

- [ ] Add target-specific Windows dependencies only as needed.
- [x] Ensure Rust code compiles on a Windows runner.
- [x] Ensure frontend builds on Windows.
- [x] Ensure Tauri app links on Windows.
- [ ] Resolve any path-separator assumptions.
- [x] Resolve any Unix-only std::os imports.
- [ ] Resolve any Linux-only build-script assumptions.

### W13.2 Windows product metadata

- [ ] Update package description so it is no longer Linux-only once Windows support is real.
- [ ] Add Windows icon resources as needed.
- [x] Verify application identifier remains appropriate.
- [x] Verify executable name.
- [x] Keep version source consistent.

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

- [x] Windows Graphics Capture.
- [x] GDI/BitBlt.
- [x] DXGI Desktop Duplication.
- [x] Any other strongly justified API.

For each record:

- [x] arbitrary rectangular region feasibility;
- [x] Windows 10/11 support;
- [x] mixed-DPI handling;
- [x] multi-monitor handling;
- [x] negative virtual coordinates;
- [x] cancellation model;
- [x] overlay interaction;

- [x] pixel format conversion;
- [x] dependency footprint;
- [x] security/maintenance implications.

### W14.2 Choose mechanism

- [x] Select one primary capture mechanism.
- [x] Decide whether a fallback is necessary.
- [x] Document why.
- [x] Add ADR or architecture note.
- [x] Define testable coordinate contract.

### W14 acceptance gate

- [x] Capture implementation is chosen based on evidence rather than convenience alone.

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

- [x] Unit-test region normalization.
- [x] Unit-test invalid rectangle rejection.
- [x] Unit-test negative coordinates.
- [x] Unit-test overflow/resource bounds.
- [x] Unit-test native error mapping where possible.

**W15 selector-state installation guard (2026-10-12):** The overlay now
verifies the ambiguous zero return from `SetWindowLongPtrW` using a
cleared `GetLastError`, and rejects an unexpected previous user-data
pointer. If state installation fails, the owned native window is destroyed
without being displayed, avoiding an unresponsive orphan selector.
Synthetic Windows-only tests cover the return/error interpretation.
Physical desktop validation remains open.

**W15 Escape focus fallback (2026-10-12):** The Win32 selector preserves
message-driven Escape handling and additionally samples a key-down transition
during the bounded overlay pump. This allows cancellation if foreground-focus
policy prevents `WM_KEYDOWN` delivery, while avoiding immediate cancellation
when Escape was already held before overlay display. Deterministic state tests
exercise pressed/held/released transitions; physical Windows 10/11 focus and
capture acceptance remain unchecked until tested. Exact-head Windows CI
qualification is still required.

### W15 acceptance gate

- [ ] Windows produces a valid shared captured image without bypassing the core.

## 20. W16 — Windows DPI and multi-monitor correctness

### W16.1 Coordinate model

- [x] Explicitly distinguish logical and physical coordinates.
- [x] Document which coordinate space selector events use.
- [x] Document which coordinate space capture API uses.
- [x] Centralize conversion.
- [x] Avoid scattered scale-factor math.

### W16.2 DPI tests

- [x] Automated pure test for 100%.
- [x] Automated pure test for 125%.
- [x] Automated pure test for 150%.
- [x] Automated pure test for 200%.
- [x] Test fractional conversion rounding policy.
- [x] Test edge coordinates.

### W16.3 Multi-monitor tests

- [x] Test monitor right of primary.
- [x] Test monitor left of primary.
- [x] Test monitor above primary.
- [x] Test negative virtual coordinates.
- [x] Test different resolutions.
- [x] Test mixed scale factors in pure coordinate logic.
- [x] Define cross-monitor drag policy.
- [x] Test cross-monitor selection if supported.

### W16 acceptance gate

- [x] Coordinate conversion is deterministic and ready for physical validation.

## 21. W17 — Windows environment and desktop integration

### W17.1 Platform probe

- [ ] Implement Windows platform identity/capability probe.
- [x] Avoid collecting unnecessary machine details.
- [x] Detect capture capability.
- [x] Detect packaged OCR runtime.
- [x] Detect English language data.
- [x] Feed safe statuses into diagnostics.

**W17.1 hosted implementation evidence (2026-10-11):** `WindowsPlatform::diagnostics` uses fixed platform labels, a bounded native monitor-availability summary, and `packaged_runtime_status()` without machine identifiers or raw native errors. `src-tauri/src/platform/windows/ocr_runtime.rs` distinguishes missing executable, missing English data, and present resources using app-owned paths; synthetic tests exercise all three content-free statuses. This is source/hosted-test evidence only; real desktop startup, DLL loading, and physical OCR are still unverified.

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

**W18 build-input redirection guard (2026-10-12):** The checksum-pinned
Mannheim archive extraction now rejects reparse-pointed or empty executable,
English language-data, and adjacent DLL inputs before staging. The build
script also refuses a redirected Tauri source or OCR staging directory before
removing existing staged files. Hosted Windows CI must qualify this change;
it does not settle archive provenance, native DLL licenses, dynamic loading,
or physical offline installation. W18 release gates remain unchecked.

**W18 filesystem redirection mitigation (2026-10-11):** The Windows
runtime resolver rejects NTFS reparse-pointed resource directories and
executable/language files in addition to ignoring PATH and CWD.
Deterministic Windows tests cover regular-file/directory substitution, with
symlink fixtures exercised where the runner permits their creation.
This does not close native DLL loader/TOCTOU/provenance, offline installation,
or physical Windows acceptance gates; exact-head CI is still required.

**W18.1 PE inventory hardening (2026-10-12):** The static import parser
now requires normal and delay-import descriptor tables to terminate inside
their declared PE data-directory sizes. Synthetic fixtures cover missing
in-bounds terminators and undersized directories. This prevents misleading
import inventories from malformed metadata; it does not establish dynamic
DLL loading safety, required licenses, or physical Windows acceptance.
Exact-head hosted CI remains required.

**Additional Windows resource ancestry check (2026-10-12):** The packaged
OCR resolver now rejects redirected executable-directory ancestors before
constructing the app-relative runtime paths. A Windows synthetic fixture
exercises symlinked ancestors when runner privileges permit. This does not
prove DLL loader behavior, cryptographic provenance, redistribution licenses,
or physical offline installation; W18 release gates remain open.

**W18 native binary review register (2026-10-12):** A review worksheet records all 52 staged PE binaries (Tesseract plus 51 adjacent DLLs), per-file hashes, static reachability, and 17 unresolved external DLL names from passing CI run 38260144405 at 93251891f2f542661e7ebb5457663bf8b12b95af. License/NOTICE, dynamic loader, physical Windows, and redistribution gates remain open; later commits require exact-head requalification.

**W18.2 runtime-layout hardening (2026-10-12):** The Windows
runtime resolver now fails closed on unexpected adjacent files/directories,
including unrelated linked entries, and unexpected nested tessdata files.
The allowed app-owned layout remains Tesseract, English trained data, and
adjacent DLLs. Windows synthetic tests exercise extra files, directories,
language files, and linked entries where privileges permit. Missing or empty
English data retains the distinct `OcrLanguageMissing` error. This does
not prove immutable binaries, DLL load-order safety, license clearance, or
physical offline installation; those gates remain unchecked pending evidence.

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
- [x] Verify product/version metadata.

**W20.1 packaged-directory integrity hardening (2026-10-12):** The
Windows NSIS payload verifier now rejects unreviewed _directories_ in the
extracted OCR resource tree (including otherwise invisible empty folders),
alongside the already pinned file manifest; it also rejects redirected
resource entries. The separate staged PE audit now rejects unreviewed root
entries and unreviewed tessdata content before inventorying native imports.
This is additional static/hosted packaging evidence subject to fresh
exact-head CI. It does not approve native redistribution, prove the installer
runs, or satisfy physical Windows desktop acceptance.

### W20.2 WebView2 policy

- [ ] Verify supported Windows WebView2 assumption.
- [x] Decide bootstrap/download/offline installer policy.
- [x] Document user impact truthfully.
- [ ] Ensure OCR itself has no network dependency.

**W20.2 policy evidence (2026-10-11):** The Windows-only NSIS candidate uses
Tauri's `offlineInstaller` WebView2 mode rather than the default online
bootstrapper. The README and `docs/WINDOWS_HOSTED_QUALIFICATION.md` document
the approximately 127 MB installer increase and the distinction between
build-time fetching, application-time OCR, and clean offline installation.
Repository-policy CI now rejects removal of the offline-install setting.
The NSIS archive verifier also checks for one embedded standalone WebView2
installer and prints its SHA-256 for later provenance review. The
packaging verifier now also requires a valid Microsoft Corporation Authenticode
signature on the extracted WebView2 installer; its exact-head Windows CI still
needs to pass. This hosted trust check is not proof of offline installation,
redistribution rights, or physical desktop behavior.
Physical WebView2 provisioning and offline installation remain unchecked.

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
- [x] User-settings retention/removal policy is documented.

**W20.3 policy evidence (2026-10-11):** `README.md`,
`docs/WINDOWS_HOSTED_QUALIFICATION.md`, and `docs/THREAT_MODEL.md`
document the intended Windows NSIS uninstall policy: remove app-owned
binaries/resources while retaining per-user Tauri `app_config_dir()` settings
by default. This documents the policy only; generated NSIS uninstall,
reinstall, settings persistence and profile-data isolation remain untested
on physical Windows and all corresponding acceptance checkboxes stay open.

### W20.4 Upgrade/reinstall

- [ ] Reinstall same version behavior is understood.
- [ ] Upgrade from prior Windows build is tested once a prior build exists.
- [ ] Settings remain valid or recover safely.
- [ ] Bundled OCR runtime upgrades cleanly.

### W20.5 Artifact integrity

- [x] Generate installer SHA-256.
- [x] Use stable artifact naming.

- [ ] Upload only reviewed installer/checksum artifacts.
- [x] Do not claim code signing unless actually signed.

**W20.5 deterministic naming implementation (2026-10-11):** The hosted NSIS
smoke normalizes the unapproved setup filename to
`screenshot-ocr-v<version>-windows-x64-setup.exe`, writes a matching local
checksum file, and repository-policy CI checks Cargo/package/Tauri metadata
coherence. CI run `38158378805` passed at exact commit
`a3370f3834c7a28d7d8652c8db87f9e830664ce8`. The candidate filename is
`screenshot-ocr-v0.1.0-windows-x64-setup.exe` and its SHA-256 is
`ad8ce81fa7268a4f1df3006d5288dfc05778add1d6e2ecc7938aa045ec58b240`.
Installer upload and redistribution remain blocked by native-license and
physical acceptance requirements.

### W20 acceptance gate

- [ ] Windows can be installed and used without developer tools or manual Tesseract setup.

## 25. W21 — Cross-platform CI

### W21.1 Branch triggers

- [x] Ensure meaningful CI runs on ralph/windows-support pushes.
- [x] Preserve master CI behavior.
- [x] Avoid cancelling unrelated platform jobs incorrectly.

### W21.2 Linux jobs

- [x] Repository policy.
- [x] Content-leakage guard.
- [x] Frontend quality.
- [x] Layout regression where applicable.
- [x] Rust formatting.
- [x] Rust Clippy.
- [x] Rust tests.
- [x] Rust build.
- [x] Debian packaging path remains valid.

### W21.3 Windows jobs

- [x] Set up Node.
- [x] Set up pinned Rust.
- [x] npm ci.
- [x] Frontend build or required shared frontend checks.
- [x] cargo fmt check or shared formatting gate.
- [x] cargo clippy for Windows-relevant targets.
- [x] cargo test.
- [x] cargo build.
- [x] Tauri Windows build.
- [x] NSIS installer build.

- [x] Verify bundled OCR files.
- [x] Generate SHA-256.
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

- [x] Portable orchestration tests pass on Windows.
- [x] Windows coordinate tests pass.
- [x] Windows runtime resolver tests pass.
- [x] Windows error-mapping tests pass.
- [x] Cancellation/state-machine tests pass.
- [x] Settings tests pass.

### W22.2 Frontend tests

- [x] Platform-specific shortcut label tests.
- [x] Platform-specific capture backend choice tests.
- [x] Shared review/copy tests.
- [x] Shared cancellation UI tests.
- [x] Shared settings error tests.
- [x] No Linux-only copy shown as actionable Windows behavior.

### W22.3 Build tests

- [x] Debug Windows build.
- [x] Release Windows build.
- [x] Installer build.
- [x] Packaged resource audit.
- [x] No unresolved Linux library dependency.

### W22 acceptance gate

- [x] Hosted automated evidence supports a physical Windows acceptance candidate.

## 27. W23 — Physical Windows desktop acceptance

Hosted CI cannot check these boxes.
Use `docs/WINDOWS_PHYSICAL_ACCEPTANCE_RUNBOOK.md` for a content-free,
exact-commit record of clean installation, offline OCR, DPI/multi-monitor,
activation, cancellation, recovery, and uninstall checks. The runbook is a
test procedure, not evidence that any physical acceptance step passed.

**W23 evidence capture template (2026-10-12):** The physical acceptance runbook now links a content-free record template covering exact installer identity, offline installation, Windows 11/10, activation, DPI/monitors, synthetic OCR fixtures, failure recovery, privacy, and uninstall/settings retention. Every result starts as NOT RUN; this is a test instrument, not acceptance evidence. Physical W23 and owner integration gates remain open.

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

**Fixture prepared (2026-10-12):** The offline, self-contained
`docs/fixtures/windows-ocr-acceptance.html` provides public light/dark,
monospace, small/large, punctuation, whitespace, edge and grid examples.
See the physical acceptance runbook for fixture IDs and instructions.
No physical Windows OCR, selection, DPI or clipboard result is implied.
A repository-policy guard now checks that this public fixture remains
self-contained (no active HTML, external resource attributes, or CSS imports),
retains all seven unique fixture IDs, and preserves terminal whitespace.
Synthetic negative tests exercise remote resources, active markup, duplicate
IDs and formatting drift. This is source-integrity evidence only; exact-head
CI must pass and W23 physical tests remain open.

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
- [ ] Cancel during active Tesseract OCR; child exits/reaps and UI recovers.
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

- [x] Update docs/ARCHITECTURE.md with portable core/platform adapters.
- [x] Add/update platform architecture ADR.
- [x] Add Windows capture API ADR.
- [x] Document coordinate/DPI model.
- [x] Document OCR runtime resolution.

### W24.2 User docs

- [ ] Update README with Windows installation.
- [x] Update README with Control+Shift+O.
- [x] Update README with tray behavior.
- [x] Update README with offline OCR statement.
- [x] Separate Linux and Windows dependency/install instructions.
- [x] Document Windows minimum version truthfully.
- [x] Document unsigned installer status if applicable.

### W24.3 Privacy/security docs

- [x] Update docs/PRIVACY.md.
- [x] Update docs/THREAT_MODEL.md.
- [x] Document bundled native runtime.
- [x] Document DLL search/provenance decision.
- [x] Document Windows temporary capture behavior if any.

### W24.4 CI/release docs

- [x] Document Linux versus Windows hosted gates.
- [x] Document physical acceptance boundary.
- [ ] Record final Linux CI run IDs.
- [ ] Record final Windows CI run IDs.
- [ ] Record installer artifact name.
- [ ] Record installer SHA-256.
- [ ] Record final branch commit SHA.
- [ ] Record remaining non-goal/deferred items.

**W24 documentation evidence (2026-10-11):** `README.md` now separates Linux release/install/build commands from unreleased Windows developer prerequisites and links the Windows-only PowerShell/NSIS workflow in `docs/WINDOWS_HOSTED_QUALIFICATION.md`. That document describes Linux and Windows hosted gates, their exact-head limitations, and the unpublished-installer policy. This does **not** check Windows installation, publication, license clearance, or physical acceptance.

### W24.5 TODO reconciliation

**W24 structural protection (2026-10-11):** A repository-policy check
validates that all W0–W25 milestones, final acceptance, physical Windows,
and explicit owner-approved integration requirements remain present and
ordered. Synthetic tests reject truncation, duplicate sections, and missing
safety gates. This protects the checklist document but does not qualify
any unrun desktop, installer, or redistribution acceptance tasks.

- [ ] Check only tasks supported by evidence.
- [x] Leave unrun physical tests unchecked.
- [x] Do not delete incomplete tasks to make the project look complete.
- [x] Add evidence notes where useful.
- [x] Re-read companion spec for untracked normative requirements.

**W24.5 normative-spec reconciliation (2026-10-12):** Reviewed the companion specification's architecture, Windows requirements, testing, release, and change-control sections against the TODO. The physical Windows acceptance matrix explicitly requires cancellation during **OCR**, not just region selection; W23.6 and the physical acceptance record now track a bounded OCR-cancel/reap/recovery case. This is a checklist-coverage correction, not evidence that the physical test ran. All physical and final merge gates remain unchecked.

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

**Targeted hosted evidence (2026-10-11):** CI run `38123649350` at `5e8fa5e95b2b68a2581be6817b01dd46765950a4` passed its Linux Debian package job `114423945914`; the package metadata explicitly requires `gnome-screenshot`, `tesseract-ocr`, `tesseract-ocr-eng`, `libayatana-appindicator3-1`, `libwebkit2gtk-4.1-0`, and `libgtk-3-0`. The same run passed Windows Rust quality gates for the content-free monitor-enumeration capture-capability summary. Pure mixed-DPI geometry and compositing tests already cover 125%/200% cross-monitor offsets. The PE inventory now distinguishes statically reachable binaries from other staged DLLs to prioritize review; this is **not** proof that other DLLs are unnecessary or redistributable. The new graph tests and final branch head require fresh exact-head CI. Physical Windows behavior remains unproven.

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
