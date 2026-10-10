# ADR 0003: Portable core and platform service boundary

**Status:** Accepted  
**Date:** 2026-10-09  
**Decision owners:** Screenshot OCR maintainers

## Context

The original Linux implementation already kept capture ownership, cancellation, image processing, OCR scoring, settings, and application action routing in Rust, but `app.rs` also selected concrete GNOME Screenshot and XDG Portal implementations and consumed Linux environment probing directly. Tesseract executable discovery was coupled to that Linux environment structure.

Adding Windows without changing this dependency direction would spread target checks through shared orchestration and create a second application path.

## Decision

Screenshot OCR uses one portable Rust application core and a narrow `PlatformServices` boundary.

The portable core owns:

- capture job reservation and one-active-job state;
- cancellation;
- typed requests and results;
- image decoding and preprocessing;
- OCR subprocess policy, cleanup, and candidate scoring;
- settings and public error policy;
- clipboard recovery policy;
- application action routing.

Platform adapters own:

- region-capture mechanisms;
- platform capability/environment probing;
- target-specific shortcut selection;
- OCR executable and language-resource location;
- platform diagnostic summaries.

The compile-time composition root in `platform/mod.rs` selects exactly one adapter for the target. Shared orchestration receives a trait object and does not select GNOME, XDG Portal, or Windows mechanisms.

Linux keeps the existing GNOME Screenshot and XDG Portal implementations under `platform/linux/`. Windows implements the same service contract under `platform/windows/`.

Tesseract recognition remains shared. A platform adapter supplies an internal `OcrRuntime` descriptor containing the executable and, when required, a tessdata directory. No frontend request can choose an executable path.

## Dependency rule

```text
frontend
   |
Tauri adapters
   |
portable application policy
   |
PlatformServices
  /       \
Linux    Windows
```

Concrete platform modules may depend on portable contracts. Portable modules must not import concrete platform adapters.

A repository-policy guard checks for known concrete platform imports and OS-probing patterns outside `src-tauri/src/platform/`. The guard supplements code review; it intentionally does not attempt to infer every possible platform-specific string.

## Consequences

Positive consequences:

- Linux and Windows use one OCR and cancellation path;
- Linux behavior can be regression-qualified independently of Windows capture work;
- target-only dependencies such as `ashpd` do not need to compile on Windows;
- Windows support can be added by implementing capabilities rather than cloning orchestration;
- platform diagnostics remain content-free and generic at the frontend boundary.

Costs:

- platform capabilities must stay narrow enough to avoid becoming a generic service locator;
- a new OS may need a small composition-root entry and adapter module;
- settings that contain historical Linux backend preferences need truthful behavior on Windows.

## Rejected alternatives

### Platform conditionals throughout shared modules

Rejected because repeated `cfg(target_os)` branches in orchestration would make policy diverge between operating systems.

### Separate Windows application core

Rejected because job state, cancellation, OCR processing, settings, and clipboard recovery are not Windows-specific.

### Separate Cargo workspace crate solely for portability

Deferred because module visibility plus the platform-boundary CI guard provide an adequate boundary at the current project size. A separate crate remains available if compile-time isolation becomes necessary.
