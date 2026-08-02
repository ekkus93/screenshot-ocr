# ADR 0002: Cross-platform activation and cancellation routing

**Status:** Accepted  
**Date:** 2026-08-01  
**Decision owners:** Screenshot OCR maintainers

## Context

Screenshot OCR hides its main window while a native region selector and local OCR are active. A Cancel button inside that hidden window is therefore not a sufficient cancellation mechanism.

The Linux v0.1 implementation must work on GNOME Wayland and X11. A future version may add macOS and Windows capture backends. Activation and cancellation must not be coupled to GNOME, D-Bus, one tray implementation, or a React component.

Tauri 2 provides desktop support for single-instance routing, global shortcuts, and system tray or menu-bar icons on Linux, macOS, and Windows. GNOME Wayland still needs a compatibility path because direct global-shortcut registration is not treated as authoritative there.

## Decision

The application uses one Rust-owned, platform-neutral application action router.

Supported actions are deliberately constrained to:

- toggle capture;
- start capture;
- cancel capture;
- show the main window;
- quit after active capture cleanup.

Activation sources are adapters only:

- command-line invocation;
- a second process routed through Tauri single-instance support;
- a directly registered global shortcut where reliable;
- a tray or macOS menu-bar action;
- the existing main-window controls.

No adapter may independently launch a selector, run OCR, write the clipboard, or bypass the capture state machine.

### Capture reservation

For an external start action, Rust reserves a new capture job before notifying the React webview. React consumes the reserved job ID and creates the normal typed `CaptureRequest`. The Rust state machine accepts that matching reservation exactly once.

This prevents rapid shortcut presses, repeated CLI invocations, or concurrent second instances from launching overlapping selectors. An unconsumed reservation expires after a bounded interval.

### Cancellation

Cancellation remains Rust-authoritative. Toggle or cancel actions signal the active cancellation token directly, including while the main window is hidden. Job ownership is retained until the active helper or OCR process is killed, reaped, and the normal cleanup path finishes.

### Single-instance behavior

The single-instance plugin is registered before other plugins. A second invocation may use only these commands:

```text
screenshot-ocr toggle
screenshot-ocr capture
screenshot-ocr cancel
screenshot-ocr show
screenshot-ocr quit
```

Unknown or absent second-instance commands only show and focus the existing window. They cannot inject arbitrary commands, paths, executable names, or OCR arguments.

### Platform policy

#### Linux GNOME Wayland

The required compatibility path is a GNOME custom shortcut that invokes:

```text
screenshot-ocr toggle
```

Direct Tauri shortcut registration is intentionally deferred on Wayland. The tray menu is an optional fallback; tray absence must not make the main application unusable.

#### Linux X11

The application attempts to register `Super+Shift+O` directly. The GNOME custom shortcut and tray menu remain fallback paths.

#### Windows

A future Windows build should use the same action router with Tauri single-instance routing, `Control+Shift+O`, and a system-tray menu. Only screen capture and platform permission integration should be Windows-specific.

#### macOS

A future macOS build should use the same action router with Tauri single-instance routing, `Command+Shift+O`, and a menu-bar status item. Only screen capture, Screen Recording permission handling, and bundle behavior should be macOS-specific.

### Tray and menu-bar behavior

The tray or menu-bar menu exposes:

- Capture text;
- Cancel capture;
- Show Screenshot OCR;
- Quit.

Quit first signals cancellation. If a capture is active, the process waits for the normal kill, reap, and cleanup path before exiting.

### Diagnostics

Tray availability and direct-shortcut registration status are reported using content-free status strings. Registration failure is nonfatal and must not be hidden as success.

## Rejected alternatives

### React Cancel button as the only cancellation mechanism

Rejected because the main window is hidden during the interval when cancellation matters most.

### Progress window as the primary mechanism

Rejected for v0.1 because it can interfere with native selectors, focus, multi-monitor placement, scaling, and captured content. A post-selection OCR progress window may be reconsidered later.

### Platform-specific orchestration

Rejected because it would duplicate job ownership, cancellation, clipboard policy, cleanup, and error handling across Linux, macOS, and Windows.

### Tray-only activation

Rejected because tray availability varies on Linux and keyboard activation is the primary developer workflow.

## Consequences

Positive consequences:

- one capture state machine remains authoritative;
- hidden-window cancellation is possible;
- repeated activations cannot create overlapping selectors;
- macOS and Windows can add thin platform adapters without replacing orchestration;
- tray failure is recoverable;
- CLI and second-instance arguments remain constrained.

Costs and follow-up work:

- macOS and Windows still require native capture backends and permission handling;
- GNOME Wayland requires documented custom-shortcut setup;
- shortcut and tray behavior require physical validation on every supported desktop;
- autostart remains a separate decision and implementation;
- dynamic tray labels may be added later, but both Capture and Cancel remain safe when shown together.

## References

- Tauri 2 single-instance plugin: <https://v2.tauri.app/plugin/single-instance/>
- Tauri 2 global-shortcut plugin: <https://v2.tauri.app/plugin/global-shortcut/>
- Tauri 2 system tray guide: <https://v2.tauri.app/learn/system-tray/>
