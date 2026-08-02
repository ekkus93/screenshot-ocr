# Screenshot OCR FIX1 Portal Lifecycle Review — 2026-08-02

**Repository:** `ekkus93/screenshot-ocr`  
**FIX1 TODO:** `docs/SCREENSHOT_OCR_FIX1_TODO_2026-08-02.md`  
**Scope:** F1.8 portal lifecycle review  
**Status:** review completed; code behavior unchanged pending physical portal validation

## Summary

The current portal backend uses `ashpd` to perform a bounded capability probe and an interactive modal area-target screenshot request. The backend accepts the portal path only when the portal advertises Screenshot interface version 3 or newer and the `Area` target is available.

The implementation keeps the existing fail-closed policy:

- `Auto` uses the portal only after capability is proven.
- `Auto` falls back to the GNOME helper before any selector opens.
- Explicit `Portal` selection fails when area support is unavailable.
- Portal result data is accepted only from absolute `file:` URIs.
- Returned files are validated as regular, non-symlink, nonempty, and size-bounded before decode.
- Raw portal URIs and paths are not serialized to the frontend.

## Lifecycle decision

FIX1 does not add a separate manual request-close operation because the current `ashpd` request API used by this backend returns after the portal request completes or errors. The backend has no retained request handle after `send()`/`response()` completes.

The remaining lifecycle work is therefore physical behavior validation, not a source-level closure hook at this layer:

- confirm user dismissal maps to cancellation;
- confirm permission denial maps to permission denied;
- confirm timed-out or cancelled app operations do not leave user-visible portal prompts in supported GNOME sessions;
- confirm the portal result file lifecycle on Ubuntu 22.04/24.04 GNOME Wayland systems;
- confirm multiple-monitor and fractional-scaling behavior.

## Explicit non-claims

This review does not claim portal capture is physically validated. Hosted CI cannot validate GNOME portal UI behavior, compositor behavior, monitor selection, permission UI, or result-file lifecycle.

## Follow-up requirements

Keep the v0.1 manual validation TODO items unchecked until there is evidence from real Ubuntu/GNOME Wayland and X11 sessions. If physical testing shows stale portal artifacts or prompts, add a targeted portal cleanup/cancellation task with exact reproduction steps before release signoff.
