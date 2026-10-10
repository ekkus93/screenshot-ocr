# Screenshot OCR Windows Portability and Windows Application Specification

**Repository:** ekkus93/screenshot-ocr  
**Document:** docs/SCREENSHOT_OCR_WINDOWS_PORTABILITY_SPEC_2026-10-09.md  
**Companion TODO:** docs/SCREENSHOT_OCR_WINDOWS_PORTABILITY_TODO_2026-10-09.md  
**Date:** 2026-10-09  
**Baseline branch:** ralph/windows-support  
**Baseline commit:** bbfd605d5c65b83735a9795622d92062c6c9c59d  
**Status:** normative implementation specification for the portable-core refactor and Windows application port

## 1. Purpose

This specification defines the architecture, implementation boundaries, validation requirements, and completion criteria for adding a Windows version of Screenshot OCR without turning the shared Rust application core into a collection of operating-system conditionals.

The work is deliberately divided into two phases.

**Phase 1 refactors the existing Linux application so the portable Rust core contains only logic that is valid on both Linux and Windows.** Linux-specific capture, environment probing, desktop integration, executable discovery, and other operating-system behavior must live behind explicit platform boundaries. Phase 1 must preserve current Linux behavior.

**Phase 2 implements the Windows application against those boundaries.** Windows support must use the same portable OCR pipeline, job ownership, cancellation, settings model, action routing, error policy, privacy policy, and frontend wherever those behaviors are truly platform-independent.

This is a portability project, not a rewrite.

## 2. Target outcome

When this project is complete:

- the Linux application still behaves as the existing product intends;
- the shared Rust core has no Linux-only or Windows-only implementation assumptions;
- Linux-specific behavior is isolated in Linux platform modules or adapters;
- Windows-specific behavior is isolated in Windows platform modules or adapters;
- the React frontend remains shared except where truthful platform-specific copy or capability presentation is required;
- one application state machine remains authoritative for capture ownership and cancellation;
- one OCR/image-processing pipeline serves Linux and Windows;
- Windows users can select a screen region, OCR it locally, review or copy the text, cancel active work, and use normal application activation paths;
- Windows packaging is self-contained enough that users do not need a separate developer toolchain;
- hosted CI validates shared behavior on Linux and Windows;
- physical Windows desktop acceptance validates behavior that hosted CI cannot prove;
- master remains untouched by this project until the Windows branch is complete and explicitly approved for integration.

## 3. Scope

This specification covers:

- refactoring Rust boundaries for Linux/Windows portability;
- preserving the current Linux GNOME/portal behavior;
- defining platform service contracts;
- target-specific Cargo dependencies;
- Windows compilation;
- Windows native screen-region selection and capture;
- Windows DPI and multi-monitor behavior;
- Windows activation, shortcut, tray, single-instance, window-focus, and cancellation behavior;
- Windows Tesseract and English language-data distribution;
- Windows clipboard integration;
- Windows diagnostics and safe error reporting;
- Windows installer production;
- hosted Linux and Windows CI;
- shared automated tests;
- platform-specific automated tests;
- physical Windows validation;
- documentation and release-readiness evidence.

## 4. Non-goals

The following are out of scope unless separately approved:

- macOS implementation;
- mobile platforms;
- cloud OCR;
- online OCR fallback;
- generative rewriting of OCR text;
- OCR history or cloud synchronization;
- additional OCR languages beyond the language policy already supported by the project;
- redesigning the React application solely to look more Windows-native;
- replacing Tauri;
- rewriting the portable OCR pipeline in another language;
- supporting Windows ARM64 in the initial Windows release;
- automating or driving the Windows Snipping Tool as an external dependency;
- broadening Tauri filesystem or shell capabilities;
- weakening current privacy or bounded-process requirements.

The architecture should not make a future macOS port unnecessarily difficult, but no macOS abstraction should be invented solely for hypothetical future use.

## 5. Normative language and evidence

The words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative.

A TODO checkbox may be marked complete only when its implementation and required evidence exist.

Hosted CI may prove compilation, tests, linting, packaging, and deterministic application behavior. Hosted CI must not be treated as proof of real desktop region-selection behavior, tray availability, global-shortcut reliability, DPI correctness, multi-monitor behavior, clipboard ownership, or installer behavior on a physical user desktop.

Manual validation evidence must identify the exact commit tested.

## 6. Branch and integration policy

All work defined by this specification MUST occur on:

ralph/windows-support

The branch was created from:

bbfd605d5c65b83735a9795622d92062c6c9c59d

Project rules:

1. This project MUST NOT modify master while Phase 1 or Phase 2 is in progress.
2. The portability refactor and Windows implementation belong on the same long-lived branch unless the repository owner explicitly changes the policy.
3. Do not create a branch or pull request per task or subtask.
4. Commits SHOULD be coherent, reviewable vertical slices.
5. Exact-head compare-and-swap writes SHOULD be used for automated repository mutations.
6. If master changes independently during the project, reconcile that divergence only when needed; do not silently overwrite or force-update either history.
7. Final integration into master is outside the automatic completion action of this specification. The branch becomes merge-ready only after all required gates pass and the repository owner explicitly approves integration.

## 7. Baseline architecture

At the baseline commit the project already has substantial portable behavior:

- Rust-owned capture job state and cancellation;
- a platform-neutral application action router;
- typed requests and results;
- bounded image decoding and preprocessing;
- OCR candidate cleanup and scoring;
- Tesseract subprocess orchestration;
- settings persistence and safe recovery behavior;
- content-free diagnostics policy;
- React/Tauri command boundaries;
- single-instance routing;
- tray and shortcut integration through Tauri;
- Linux capture through GNOME Screenshot and XDG Desktop Portal;
- Linux packaging as a Debian package.

The main portability problem is not the OCR algorithm. The main problem is dependency direction and placement of operating-system decisions.

At baseline, app.rs directly:

- probes the Linux environment;
- chooses between XDG Portal and GNOME Screenshot;
- constructs Tesseract from environment information;
- imports Linux capture implementations.

The baseline Cargo manifest also declares Linux-specific dependencies unconditionally, including ashpd.

These facts make a clean Windows build more difficult than necessary even though much of the application logic is already portable.

## 8. Architectural principles

### 8.1 Portable core means neither Linux nor Windows

The portable core MUST contain logic that is valid without knowing which supported operating system is running.

It MUST NOT contain:

- GNOME-specific names or behavior;
- XDG Portal implementation details;
- Windows API calls;
- Windows registry assumptions;
- Linux filesystem conventions;
- Windows filesystem conventions;
- hard-coded platform executable locations;
- platform-specific global shortcut implementation;
- platform-specific screen-capture implementation;
- platform-specific permission UI;
- direct selection between Linux and Windows implementations.

Conditional compilation inside the portable core SHOULD be treated as an architectural smell. Limited cfg use for unavoidable type or crate compatibility MAY exist at a narrow boundary, but platform choice belongs in the platform layer.

### 8.2 Platform adapters implement capabilities

Operating-system integrations are adapters. They provide capabilities to the shared application.

The shared application owns policy such as:

- one active capture;
- cancellation;
- OCR sequencing;
- preprocessing;
- candidate selection;
- clipboard policy;
- settings policy;
- public error policy;
- action routing;
- privacy.

The platform adapter owns mechanisms such as:

- how screen pixels are acquired;
- how region selection is implemented;
- how platform capability is probed;
- how a platform-specific OCR executable/resource location is resolved;
- how OS-specific diagnostics are measured;
- any OS-specific permission interaction not abstracted safely by Tauri.

### 8.3 Dependency direction

The required dependency direction is:

```text
React / TypeScript UI
        |
Tauri command/event adapters
        |
Portable Rust application services
        |
Platform service contracts
      /   \
 Linux     Windows
 adapters  adapters
```

The Linux and Windows implementations may depend on the portable contracts.

The portable application MUST NOT depend on concrete Linux or Windows implementations.

### 8.4 Preserve one orchestration path

There MUST NOT be separate Linux and Windows OCR orchestrators.

A Windows capture should produce the same portable captured-image contract consumed by the same image preprocessing and OCR flow as Linux.

## 9. Recommended Rust module shape

The exact filenames may evolve, but the code SHOULD converge on a shape similar to:

```text
src-tauri/src/
  app.rs
  actions.rs
  cancellation.rs
  commands.rs
  diagnostics.rs
  error.rs
  image_pipeline/
  models.rs
  ocr/
  settings.rs
  state.rs

  platform/
    mod.rs
    services.rs

    linux/
      mod.rs
      capture/
      environment.rs
      desktop.rs
      ocr_runtime.rs

    windows/
      mod.rs
      capture/
      environment.rs
      desktop.rs
      ocr_runtime.rs
```

The project does not need to follow this tree literally. It does need to preserve the boundary.

## 10. Portability classification

The Phase 1 audit MUST classify every relevant Rust module and dependency.

The expected classification is:

| Area                                       | Expected classification                             |
| ------------------------------------------ | --------------------------------------------------- |
| Capture job ownership/state machine        | Portable core                                       |
| Cancellation token and cancellation policy | Portable core                                       |
| Application action enum/router             | Portable core                                       |
| Typed capture/OCR requests and results     | Portable core                                       |
| Image decode, limits, preprocessing        | Portable core                                       |
| OCR cleanup and scoring                    | Portable core                                       |
| OCR result selection                       | Portable core                                       |
| Settings schema and validation             | Portable core                                       |
| Public error taxonomy                      | Portable core                                       |
| Privacy/content-leakage policy             | Portable core                                       |
| Tauri command DTO wiring                   | Mostly portable adapter                             |
| Clipboard policy                           | Portable; mechanism may use Tauri                   |
| Single-instance policy                     | Portable; mechanism may use Tauri                   |
| Tray action policy                         | Portable; availability/setup may be platform-aware  |
| Shortcut action policy                     | Portable; registration policy may be platform-aware |
| GNOME Screenshot                           | Linux adapter                                       |
| XDG Desktop Portal                         | Linux adapter                                       |
| DISPLAY/XDG/GNOME probing                  | Linux adapter                                       |
| Linux helper executable discovery          | Linux adapter                                       |
| Windows region selector/capture            | Windows adapter                                     |
| Windows OS/runtime probing                 | Windows adapter                                     |
| Windows OCR resource/executable discovery  | Windows adapter                                     |
| Windows-specific installer/resource paths  | Windows adapter/build layer                         |

If an implementation does not fit this table, the implementation must document why.

## 11. Platform service contracts

Phase 1 MUST define narrow contracts for capabilities that genuinely vary by platform. Avoid speculative interfaces.

At minimum, the architecture must provide a clean boundary for:

1. capture backend selection and region capture;
2. platform environment/capability probing;
3. OCR runtime/resource resolution where platform paths differ;
4. platform diagnostics needed by the shared diagnostics report;
5. platform desktop integration only where Tauri does not already provide a portable mechanism.

Interfaces MUST use portable domain types.

Interfaces MUST NOT expose shell fragments, raw platform command lines, arbitrary executable paths from the frontend, or user-controlled process arguments.

## 12. Capture contract

### WIN-R1: Portable capture result

Every platform capture implementation MUST return an equivalent portable result containing:

- decoded image pixels or the existing bounded in-memory image type;
- a stable backend identifier;
- content-free cleanup or warning state where applicable.

The rest of the OCR pipeline MUST NOT need to know whether those pixels came from GNOME Screenshot, XDG Portal, or Windows.

### WIN-R2: Capture backend identity

CaptureBackendId or its successor MUST support Windows without making Windows pretend to be a Linux backend.

Stable serialized identifiers SHOULD be explicit, for example:

- xdg_portal;
- gnome_screenshot;
- windows_native.

If the actual Windows implementation uses more than one internal mechanism, public identity may remain windows_native unless exposing the distinction helps diagnostics without leaking sensitive data.

### WIN-R3: Capture preference policy

Existing Linux capture preferences MUST remain truthful.

A Linux-only preference such as forcing GNOME Screenshot MUST NOT become a misleading active Windows option.

The settings model may:

- keep a portable Auto option;
- retain Linux-specific persisted values for compatibility;
- map unsupported platform-specific values to a safe fallback with a visible warning; or
- expose platform-filtered UI choices.

The chosen policy must preserve settings migration safety.

## 13. Linux Phase 1 requirements

### WIN-R4: Linux behavior preservation

Phase 1 is a refactor, not a Linux feature rewrite.

The following behavior MUST be preserved unless an existing bug is separately documented:

- GNOME Screenshot backend;
- XDG Portal backend and area-selection fail-closed policy;
- backend preference semantics;
- existing capture cancellation semantics;
- existing private temporary-directory ownership and cleanup policy;
- current Linux shortcut policy;
- current tray actions;
- current settings behavior;
- current OCR behavior;
- current privacy behavior;
- current Debian packaging behavior.

### WIN-R5: Linux-specific dependencies

Linux-only crates SHOULD be target-scoped in Cargo.toml so Windows does not compile them unnecessarily.

ashpd MUST become a Linux-target dependency unless there is a demonstrated cross-platform reason to keep it unconditional.

Any new Windows API crate MUST be target-scoped to Windows.

### WIN-R6: Linux environment probing

DISPLAY, WAYLAND_DISPLAY, XDG_CURRENT_DESKTOP, GNOME helper discovery, portal capability probing, and equivalent Linux concepts MUST live in Linux platform code.

The portable application may consume a generic capability description, but it MUST NOT parse Linux desktop environment variables itself.

## 14. OCR runtime architecture

### WIN-R7: Shared OCR engine behavior

The Tesseract recognition logic SHOULD remain shared.

Shared behavior includes:

- bounded subprocess lifetime;
- no shell execution;
- cancellation;
- bounded stdout/stderr handling;
- image-to-Tesseract input handling;
- English-language availability verification;
- public error mapping;
- OCR candidate production.

### WIN-R8: Platform-specific runtime resolution

Locating the Tesseract executable and tessdata is platform-specific policy and MUST be separated from shared recognition behavior.

Linux may continue to rely on approved system installations and current package dependencies.

Windows SHOULD ship an application-controlled OCR runtime rather than require users to manually install Tesseract.

The Windows resolver MUST prefer the packaged runtime for normal release builds.

The frontend MUST NOT supply executable paths.

### WIN-R9: Bundled runtime integrity and licensing

If Windows bundles Tesseract, Leptonica, language data, or related native DLLs:

- versions must be pinned or otherwise reproducible;
- provenance must be documented;
- required licenses and notices must be shipped;
- CI must verify required runtime files exist;
- runtime resolution must not search arbitrary current-working-directory locations;
- DLL search behavior must avoid unsafe preloading from untrusted directories;
- the installer must place resources in application-controlled locations;
- missing/corrupt packaged resources must fail with safe public errors.

## 15. Application orchestration requirements

### WIN-R10: AppServices portability

AppServices and capture_inner or their successors MUST orchestrate through injected/selected platform services rather than importing concrete Linux or Windows implementations.

The orchestration sequence remains conceptually:

1. validate request;
2. reserve/begin one capture job;
3. load settings;
4. obtain platform capability/capture service;
5. hide UI at the correct capture boundary;
6. capture region;
7. restore UI according to existing policy;
8. check cancellation;
9. resolve/probe OCR runtime;
10. prepare image variants;
11. recognize candidates;
12. select best candidate;
13. return portable result;
14. apply clipboard/UI policy through existing command layer.

### WIN-R11: Single ownership and cancellation

Windows MUST reuse the existing one-active-capture state machine.

No Windows selector, thread, process, or native callback may create a second independent job-ownership model.

Cancellation MUST be able to terminate or dismiss:

- active region selection;
- capture acquisition;
- Tesseract OCR;
- pending shared orchestration.

The application MUST retain job ownership until active platform resources are cleaned up.

## 16. Desktop activation and window lifecycle

### WIN-R12: Shared action router

ADR 0002 remains authoritative.

Windows MUST use the same constrained Rust-owned action router for:

- capture;
- cancel;
- toggle;
- show;
- quit.

No Windows-specific adapter may bypass the action router and invoke OCR directly.

### WIN-R13: Windows shortcut

The initial Windows shortcut policy is Control+Shift+O.

The implementation SHOULD use the existing Tauri global-shortcut mechanism where reliable.

Registration failure must be nonfatal, visible in diagnostics, and must not be represented as success.

### WIN-R14: Windows tray

Windows SHOULD expose the existing tray action set:

- Capture text;
- Cancel capture;
- Show Screenshot OCR;
- Quit.

Tray actions must route through the shared application action router.

### WIN-R15: Window hide/show behavior

The app window must not appear inside the selected capture region because of incorrect timing.

The platform capture interface must preserve the existing before-region-capture and after-region-capture lifecycle semantics or replace them with an equivalent explicit lifecycle contract.

Window restoration must occur on:

- capture success;
- capture cancellation where appropriate;
- capture failure;
- clipboard failure recovery where the current product expects review.

## 17. Windows capture requirements

### WIN-R16: Native implementation

Windows capture MUST use a Windows-native implementation.

It MUST NOT require:

- GNOME utilities;
- XDG Portal;
- WSL;
- Python;
- PowerShell scripts;
- shell commands;
- automation of the Windows Snipping Tool;
- a separate screenshot application.

### WIN-R17: Implementation spike

Before locking the Windows capture mechanism, Phase 2 MUST perform and document a bounded feasibility comparison of viable Windows-native approaches.

Candidate mechanisms may include Windows Graphics Capture, GDI/BitBlt, DXGI Desktop Duplication, or another appropriate supported API.

The decision must consider:

- arbitrary rectangular region capture;
- multiple monitors;
- mixed DPI;
- negative virtual-screen coordinates;
- performance;
- selector-overlay interaction;
- cancellation;
- Windows 10/11 compatibility;
- implementation complexity;
- dependency/security footprint.

The chosen mechanism must be documented in an ADR or equivalent architecture note.

### WIN-R18: Region-selection UX

The Windows version MUST provide an application-owned rectangular selection experience unless the chosen native API provides an equivalent reliable region-selection contract.

Required behavior:

- initiate selection from Capture or the shortcut;
- visually indicate selection mode;
- allow click-drag rectangular selection;
- allow cancellation with Escape;
- reject empty or pathologically small regions safely;
- avoid capturing the selection overlay itself;
- avoid leaving hidden/fullscreen overlay windows after cancellation or failure;
- return pixels directly or through a private application-owned transient path that obeys the existing privacy model.

### WIN-R19: Multi-monitor and virtual desktop coordinates

Windows capture MUST handle:

- a primary monitor;
- secondary monitors;
- monitors positioned left or above the primary monitor;
- different monitor resolutions;
- region selection crossing monitor boundaries if the chosen UX permits it;
- negative virtual-screen coordinates where Windows reports them.

If cross-monitor dragging is intentionally unsupported, the UI and tests must make that limitation explicit rather than producing wrong pixels.

### WIN-R20: DPI scaling

Windows capture and selection coordinates MUST be correct at common scaling levels including:

- 100%;
- 125%;
- 150%;
- 200%.

Mixed-DPI multi-monitor configurations require physical acceptance testing.

Logical CSS/Tauri coordinates MUST NOT be assumed to equal physical capture pixels without explicit conversion.

### WIN-R21: Color and image correctness

Captured pixels must preserve sufficient color fidelity for the existing preprocessing pipeline.

The Windows adapter must produce a supported pixel format and orientation.

Tests should cover:

- normal light text;
- dark-mode text;
- high-DPI text;
- very small selection rejection;
- large but allowed selection;
- edge-of-screen regions.

## 18. Windows environment and diagnostics

### WIN-R22: Windows capability probe

A Windows platform probe SHOULD report only the capabilities needed for behavior and diagnostics.

Examples include:

- supported Windows version family;
- capture backend availability;
- packaged Tesseract runtime availability;
- English tessdata availability;
- shortcut registration status;
- tray setup status.

Diagnostics MUST remain content-free.

### WIN-R23: No sensitive platform details

Public diagnostics and errors MUST NOT expose:

- usernames;
- user profile paths;
- installation paths;
- temporary paths;
- screenshot pixels;
- OCR text;
- clipboard text;
- raw Windows API error strings if they may contain paths or content;
- raw child-process output.

Stable safe error codes and bounded static guidance are preferred.

## 19. Windows settings and frontend behavior

### WIN-R24: Shared settings schema

The project SHOULD keep one settings schema where practical.

Platform-specific UI choices must be truthful.

Linux-only capture backend choices must not appear as working Windows choices.

Existing persisted Linux settings must not corrupt or crash Windows startup if a user profile is migrated.

Unknown or unsupported persisted enum values must fail safely according to the current settings recovery policy.

### WIN-R25: Shared frontend

The React frontend SHOULD remain shared.

Platform-specific frontend branches are appropriate only for:

- platform-specific shortcut labels;
- platform-specific capture backend choices;
- platform-specific setup guidance;
- platform capability/diagnostic labels.

Business logic such as job state, OCR review, copy retry, cancellation, settings-save behavior, and error rendering should remain shared.

## 20. Windows clipboard policy

### WIN-R26: Preserve clipboard contract

Windows must preserve the current clipboard behavior:

- no copy before OCR success;
- no whitespace-only copy;
- immediate-copy success sets copied state truthfully;
- clipboard failure after successful OCR preserves recognized text for recovery;
- retry copy remains available.

Use the existing Tauri clipboard plugin if it satisfies Windows requirements.

A Windows-only clipboard implementation should be added only if required by demonstrated behavior.

## 21. Windows build and packaging

### WIN-R27: Target support

Initial Windows release target:

- Windows x86-64;
- Windows 10 22H2 or later where the required Tauri/WebView2/runtime behavior is supported;
- Windows 11.

Windows ARM64 is deferred.

The exact minimum Windows version must be verified against the final Tauri and capture API choices before release documentation is finalized.

### WIN-R28: WebView2

The installer/build strategy must account for Microsoft WebView2 as required by Tauri.

The release documentation must truthfully state whether the installer bootstraps WebView2, relies on the Evergreen runtime normally present on supported Windows versions, or packages an approved alternative.

### WIN-R29: Installer

The Windows release MUST produce at least one normal end-user installer.

NSIS is the preferred initial installer unless implementation evidence shows MSI is clearly preferable.

The installer must:

- install the application and required OCR runtime/resources;
- create no unexpected network dependency for OCR;
- support normal uninstall;
- remove application binaries/resources it owns;
- preserve or remove user settings only according to documented installer policy;
- avoid requiring a developer toolchain.

An MSI MAY also be produced if low-cost after the primary installer is reliable.

### WIN-R30: Version/resource metadata

Windows artifacts must have:

- application name;
- version;
- publisher metadata where available;
- icons appropriate for Windows;
- executable metadata;
- reproducible artifact naming;
- SHA-256 checksum generation in CI.

Code signing may be deferred if signing credentials are not available, but unsigned status must be explicit and must not be presented as signed.

## 22. Cargo and build-system requirements

### WIN-R31: Target-specific dependencies

Cargo.toml SHOULD use target-specific dependency sections so platform-only crates do not pollute the other build.

The project must build the shared Rust library/tests on both Linux and Windows.

Cargo.lock remains committed.

### WIN-R32: No all-features trap

CI commands must not accidentally enable mutually exclusive Linux and Windows platform dependencies in a way that creates false failures.

If Cargo features are introduced, their semantics must be documented and deterministic.

Prefer target cfg over user-selectable features for actual OS identity.

### WIN-R33: Tauri configuration

Tauri bundle configuration must support Linux Debian packaging and Windows installer packaging without making one platform's package requirements mandatory on the other.

Linux package dependencies such as gnome-screenshot and tesseract-ocr must remain Linux-only.

Windows bundled OCR resources must remain Windows-only.

## 23. CI requirements

### WIN-R34: Branch CI

During development, pushes to ralph/windows-support SHOULD receive meaningful CI coverage.

If the existing workflow only runs on master, update the workflow so this long-lived branch is validated without weakening master gates.

### WIN-R35: Shared quality gates

Both Linux and Windows CI must run, where applicable:

- Rust formatting;
- Rust Clippy with warnings denied;
- Rust tests;
- Rust build;
- frontend formatting/lint/typecheck/tests;
- frontend build;
- repository policy/content-leakage checks.

Frontend-only work need not be duplicated unnecessarily if one job already proves deterministic platform-neutral frontend behavior, but Windows must at least prove the Tauri/Rust application compiles.

### WIN-R36: Windows CI

Windows hosted CI must verify:

- cargo check/build/test for the Windows target on a Windows runner;
- Tauri application build;
- packaged OCR runtime/resource presence;
- Windows installer generation;
- installer artifact checksum;
- no Linux-only dependency leaks into Windows compilation.

Hosted CI must not claim real interactive region-selection acceptance.

### WIN-R37: Linux CI preservation

Linux CI must continue to verify:

- existing Rust/frontend quality gates;
- Debian package production;
- Linux-specific dependencies;
- Linux tests.

A green Windows build with broken Linux behavior is not acceptable.

## 24. Automated testing requirements

### WIN-R38: Portable core tests

Portable core tests must run without a real desktop wherever possible.

Use test doubles/fakes for platform services so tests can prove:

- capture orchestration ordering;
- cancellation;
- backend failure mapping;
- OCR runtime failure mapping;
- cleanup warning propagation;
- one-active-job ownership;
- settings behavior;
- shared error policy.

### WIN-R39: Platform adapter tests

Linux adapter tests should preserve current deterministic coverage.

Windows adapter tests should isolate pure coordinate conversion, monitor layout, DPI conversion, region validation, backend selection, runtime path construction, and error mapping from real desktop integration where possible.

### WIN-R40: Compile-time architecture guard

The project SHOULD add a lightweight automated guard or reviewable test that helps prevent Linux-specific symbols from migrating back into the portable core.

Acceptable approaches include:

- module placement plus dependency rules;
- source guard script for known Linux/Windows implementation namespaces;
- a separate portable crate if justified by complexity;
- compilation tests.

Creating a separate Cargo crate is optional, not mandatory. Do not restructure into multiple crates solely for aesthetics.

## 25. Privacy and security requirements

All existing privacy and threat-model requirements remain in force.

Additionally:

1. Windows capture pixels must remain local.
2. OCR must remain local.
3. No telemetry is introduced by this project.
4. No shell execution is introduced.
5. No arbitrary executable path is accepted from the frontend.
6. No screenshot or OCR text is written to logs.
7. Temporary capture storage, if any, must be application-owned, private, bounded, and deleted.
8. Windows native error values must be mapped to stable safe public errors.
9. Packaged DLL loading must avoid unsafe search behavior.
10. Installer build inputs and bundled OCR binaries must have documented provenance.
11. CI artifacts must not include captured user content.
12. Test fixtures must remain synthetic/public and non-sensitive.

## 26. Phase 1 completion gate

Phase 1 is complete only when all of the following are true on the exact branch head:

- the portable Rust core no longer imports concrete Linux capture implementations;
- the portable Rust core does not parse Linux desktop environment variables;
- Linux capture/environment/runtime resolution is behind explicit platform boundaries;
- Linux-only Cargo dependencies are target-scoped;
- shared OCR/image-processing/job/cancellation logic remains shared;
- architecture tests/guards exist at an appropriate level;
- Linux unit/integration tests pass;
- frontend tests pass;
- Linux application build passes;
- Debian package build remains viable;
- existing Linux behavior has no known regression caused by the refactor;
- documentation describes the new boundary;
- Phase 1 TODO items are reconciled with evidence.

Windows feature implementation SHOULD NOT begin before this gate except for tiny compile spikes needed to validate the abstraction itself.

## 27. Phase 2 completion gate

Phase 2 is complete only when:

- Windows builds from the shared branch;
- Windows uses the shared portable core;
- native region selection/capture works;
- cancellation works during selection and OCR;
- Control+Shift+O works or a documented nonfatal fallback exists;
- tray actions work;
- single-instance routing works;
- DPI conversion is correct on required test configurations;
- multi-monitor behavior is validated;
- packaged Tesseract and English tessdata work offline;
- clipboard success and recoverable failure behavior are validated;
- Windows diagnostics remain content-free;
- Windows installer installs and uninstalls successfully;
- Windows CI is green on the exact final commit;
- Linux CI remains green on the same exact commit;
- required physical Windows acceptance is recorded;
- documentation matches reality;
- no required TODO item remains unchecked except explicitly deferred non-goals.

## 28. Physical Windows acceptance matrix

At minimum, final acceptance SHOULD cover:

| Area             | Required coverage                                                             |
| ---------------- | ----------------------------------------------------------------------------- |
| Windows versions | Windows 11 and at least one supported Windows 10 22H2 environment if feasible |
| Architecture     | x86-64                                                                        |
| Scaling          | 100%, 125%, 150%, 200% across the matrix                                      |
| Displays         | single monitor and multi-monitor                                              |
| Mixed DPI        | required physical validation                                                  |
| Capture          | light and dark content                                                        |
| Cancellation     | selection and OCR                                                             |
| Activation       | UI button, shortcut, tray, second instance                                    |
| Clipboard        | success and recoverable failure behavior                                      |
| Installer        | clean install, launch, reinstall/upgrade policy, uninstall                    |
| Offline OCR      | network disconnected or otherwise proven unnecessary                          |
| Diagnostics      | no sensitive content                                                          |
| Linux regression | current supported Linux validation remains separate and honest                |

Not every scaling value needs every Windows version, but the combined matrix must cover each required behavior meaningfully.

## 29. Release and documentation requirements

Before the branch is declared merge-ready:

- README must explain Linux and Windows installation separately;
- README must state Windows shortcut and capture behavior;
- ARCHITECTURE.md must document portable core and platform adapters;
- PRIVACY.md must cover Windows capture/runtime behavior;
- THREAT_MODEL.md must cover packaged native runtime/DLL risks;
- relevant ADRs must be updated or a new Windows-capture ADR added;
- CI documentation must distinguish hosted CI from physical desktop validation;
- the authoritative TODO must contain exact final evidence;
- versioning/release notes must not claim Windows support before qualified artifacts exist.

## 30. Deferred future work

Unless separately approved, leave the following deferred:

- macOS;
- Windows ARM64;
- Microsoft Store packaging;
- automatic update service;
- cloud OCR;
- additional OCR languages;
- installer code signing if credentials are unavailable;
- MSI if NSIS fully satisfies the initial release;
- platform-specific UI redesign.

## 31. Change-control rule

If Phase 2 requires repeated changes to portable core interfaces, stop and determine whether:

1. the Phase 1 abstraction is genuinely incomplete; or
2. Windows-specific behavior is leaking into shared policy.

A small shared model extension is normal. A growing set of Windows conditionals in portable orchestration is not.

Any material deviation from this specification should be documented in the companion TODO and architecture documentation rather than silently implemented.
