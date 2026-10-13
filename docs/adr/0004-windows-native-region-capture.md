# ADR 0004: Windows native rectangle capture with GDI

**Status:** Accepted  
**Date:** 2026-10-10  
**Decision owners:** Screenshot OCR maintainers

## Context

Screenshot OCR needs an application-owned Windows region selector that returns one bounded image for the shared OCR pipeline. The initial Windows release targets Windows 10 22H2 and Windows 11 on x86-64, must work offline, must support multiple monitors and mixed DPI, and must not automate the Windows Snipping Tool or introduce a separate Windows OCR orchestration path.

The Phase 2 feasibility gate compared three Windows-native capture families:

1. Windows Graphics Capture (WGC);
2. GDI screen capture with `BitBlt`;
3. DXGI Desktop Duplication.

The selector itself is separate from pixel acquisition. Screenshot OCR needs arbitrary rectangle selection, not merely selection of a whole monitor or top-level window.

## Decision

Use an application-owned native Win32 selector and GDI `BitBlt` as the primary Windows pixel-capture mechanism for the initial release.

Do not add a DXGI or WGC fallback initially. A fallback may be added later only if physical acceptance reveals a concrete class of supported desktops that the GDI implementation cannot capture reliably.

The Windows adapter remains behind the existing `PlatformServices` / `CaptureBackend` contracts. Shared application orchestration, cancellation, image preprocessing, OCR, settings, clipboard recovery, and public error policy remain unchanged.

## Feasibility comparison

| Criterion                    | Windows Graphics Capture                                                           | GDI / BitBlt                                                                 | DXGI Desktop Duplication                                                                      |
| ---------------------------- | ---------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| Arbitrary rectangle          | Requires app-side crop after choosing/capturing a window or display                | Direct rectangle source coordinates                                          | Requires app-side crop from duplicated output                                                 |
| Built-in picker              | System picker selects a display or application window, not arbitrary OCR rectangle | No; app owns selector                                                        | No; app owns selector                                                                         |
| Windows 10/11                | Supported on desktop Windows with capability check                                 | Long-standing Win32 desktop API                                              | Windows 8+ desktop API                                                                        |
| Multi-monitor                | Capture item is window/display oriented                                            | Virtual-screen coordinates and monitor enumeration are native Win32 concepts | One duplication interface per active output; whole-desktop capture requires combining outputs |
| Mixed DPI                    | App must still map selector coordinates correctly                                  | Explicit physical-pixel contract under Per-Monitor-v2 awareness              | App must still map selector coordinates and output surfaces correctly                         |
| Negative virtual coordinates | App-side coordinate handling required                                              | Directly represented by virtual-screen metrics / monitor rectangles          | App-side output-to-virtual-desktop mapping required                                           |
| Cancellation                 | Picker/session lifecycle plus selector lifecycle                                   | Selector message-loop cancellation is under app control                      | Frame acquisition plus selector lifecycle                                                     |
| Pixel format                 | GPU/WinRT frame surfaces                                                           | GDI bitmap converted to RGBA                                                 | BGRA DXGI surface                                                                             |
| Dependency footprint         | WinRT + graphics capture APIs                                                      | User32/GDI32 only                                                            | D3D11 + DXGI                                                                                  |
| Complexity for one-shot OCR  | Higher than needed                                                                 | Lowest of the viable options                                                 | Highest; optimized for frame-by-frame duplication/streaming                                   |

Microsoft documents WGC's standard `GraphicsCapturePicker` as selecting a display or application window. That does not replace Screenshot OCR's arbitrary rectangular selection UX.

Microsoft documents Desktop Duplication as output-oriented: `IDXGIOutput1::DuplicateOutput` duplicates an adapter output, and an application that wants the entire desktop must create an interface for each active output and combine them. That capability is useful for streaming/remote-desktop workloads but is unnecessary complexity for a one-shot OCR rectangle.

Microsoft documents `BitBlt` as copying a rectangular set of pixels between device contexts, which directly matches the acquisition operation needed after the app has a selected rectangle.

### Overlay and security/maintenance considerations

All three native pixel-acquisition options still require an application-owned arbitrary-rectangle selector. With GDI the overlay is destroyed before BitBlt; WGC and DXGI would additionally require capture-session/frame lifecycle and an explicit overlay-exclusion/cropping policy. No option removes the need to validate overlay absence on a physical desktop.

GDI limits the initial native dependency and maintenance surface to Win32 User32/GDI32 and owned handles, at the cost of compositor/protected-content limitations requiring physical validation. WGC adds WinRT capture permissions and session lifecycle; DXGI adds D3D11 device/output-duplication state, GPU resource cleanup, and device-loss handling. None justifies an additional capture mechanism for this one-shot OCR workload without evidence. Automating the Snipping Tool is not a viable native API alternative under the no-external-picker requirement.

## Coordinate contract

The Windows adapter uses **physical virtual-screen pixels** as its internal coordinate space.

### Process/window awareness

The Windows application must run with Per-Monitor-v2 DPI awareness on supported Windows versions. Logical CSS/webview coordinates must never be assumed to equal capture pixels.

The selector reports normalized physical coordinates:

```text
PhysicalRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32
}
```

with these invariants:

- `right > left`;
- `bottom > top`;
- coordinates may be negative;
- width/height arithmetic is checked before allocation;
- the rectangle must intersect the current virtual screen;
- resource limits are applied before pixel allocation.

### Virtual desktop

Use the Windows virtual-screen coordinate system. `SM_XVIRTUALSCREEN` and `SM_YVIRTUALSCREEN` identify the virtual screen origin; `SM_CXVIRTUALSCREEN` and `SM_CYVIRTUALSCREEN` identify its size.

Monitor enumeration uses `EnumDisplayMonitors` / monitor rectangles in virtual-screen coordinates.

### Monitor stitching

A selected rectangle may intersect more than one monitor. The implementation must:

1. enumerate monitor rectangles intersecting the normalized selection;
2. compute the physical-pixel intersection for each monitor;
3. capture each intersection using a device context compatible with that display;
4. convert the captured pixels to the shared RGBA image representation;
5. place each captured intersection at the correct offset in one bounded output buffer.

This avoids assuming every multi-monitor topology behaves as one homogeneous GDI device and makes negative-coordinate layouts explicit.

Cross-monitor selection is therefore supported by design rather than silently clipped to the primary monitor.

## Selector behavior

The selector is an application-owned topmost Win32 overlay spanning the virtual desktop.

Required behavior:

- begin only after shared job ownership has been acquired;
- show a visible drag-selection state;
- accept left-button drag in either direction and normalize the result;
- cancel on Escape;
- reject zero-size and pathologically small rectangles;
- close/hide the overlay before pixel acquisition so the overlay is not captured;
- return `CaptureCancelled` for user cancellation;
- release every HWND/HDC/HBITMAP/GDI resource on success, cancellation, and failure;
- integrate the shared `CancellationToken` so tray/shortcut/app cancellation dismisses the selector;
- never create a second independent job state machine.

The main Tauri window remains controlled by the existing shared before/after-region-capture lifecycle. The selector itself belongs entirely to the Windows platform adapter.

## Pixel acquisition

The initial backend uses:

- `GetDC` / monitor-compatible device contexts;
- compatible memory DCs and bitmaps;
- `BitBlt` with `SRCCOPY` and, where appropriate, `CAPTUREBLT`;
- `GetDIBits` for a top-down 32-bit DIB;
- explicit conversion to the image crate's shared RGBA representation.

The code must validate every native handle/result and map failures to stable `AppError` values without returning raw Win32 error text.

GDI color management is not required for OCR correctness; the existing preprocessing pipeline operates on the captured pixel values.

## DPI policy

Per-Monitor-v2 awareness is required because the app must preserve physical-pixel correspondence as the pointer and selector move across monitors with different scale factors.

Pure coordinate code must be separated from native calls and unit-tested for:

- 100%, 125%, 150%, and 200% scale factors;
- negative virtual-screen origins;
- monitor layouts left/right/above the primary display;
- mixed scale factors;
- edge intersections;
- reversed drag direction;
- overflow and invalid rectangles.

Physical mixed-DPI validation remains mandatory because hosted CI cannot prove desktop compositor behavior.

## Cancellation model

The existing shared `CancellationToken` remains authoritative.

The selector message loop checks cancellation on a bounded cadence and dismisses the overlay when cancellation becomes true. Escape requests the same capture-cancel outcome. Native capture after a completed selection is a bounded synchronous operation and must check cancellation before and after acquisition.

No native callback may mark the shared job complete independently; shared application/command code retains ownership until the platform backend returns.

## Security and privacy

The Windows capture adapter:

- performs no network access;
- invokes no shell;
- writes no screenshot to disk;
- logs no pixels or selected coordinates;
- returns no raw Win32 error string to the frontend;
- accepts no HWND, executable path, or native pointer from frontend input;
- keeps all native handles process-local and RAII-owned where possible.

## Rejected alternatives

### Windows Graphics Capture as primary backend

Rejected for the initial release because its standard picker selects a display or application window, while Screenshot OCR requires arbitrary rectangle selection. Using WGC would still require an application-owned region selector/crop while adding WinRT and graphics-session machinery.

WGC remains a possible future fallback if physical testing identifies a protected/compositor scenario where GDI capture is inadequate and WGC can satisfy the same privacy and UX contract.

### DXGI Desktop Duplication as primary backend

Rejected for the initial release because it is designed for repeated frame acquisition and exposes one duplicated output at a time. Capturing a region spanning outputs requires per-output duplication and composition plus D3D11 resource handling. That is disproportionate complexity for one-shot OCR.

DXGI remains a possible future optimization/fallback if a demonstrated requirement justifies it.

### Automating Windows Snipping Tool

Rejected because it would make correctness and cancellation depend on another application's UI and version-specific behavior.

## Microsoft references

- BitBlt: https://learn.microsoft.com/windows/win32/api/wingdi/nf-wingdi-bitblt
- GetDC: https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-getdc
- GetSystemMetrics virtual-screen metrics: https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-getsystemmetrics
- EnumDisplayMonitors: https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-enumdisplaymonitors
- DPI awareness contexts / Per-Monitor-v2: https://learn.microsoft.com/windows/win32/hidpi/dpi-awareness-context
- Default DPI awareness: https://learn.microsoft.com/windows/win32/hidpi/setting-the-default-dpi-awareness-for-a-process
- Windows Graphics Capture: https://learn.microsoft.com/windows/apps/develop/media-authoring-processing/screen-capture
- DXGI DuplicateOutput: https://learn.microsoft.com/windows/win32/api/dxgi1_2/nf-dxgi1_2-idxgioutput1-duplicateoutput

## Consequences

Positive:

- smallest native dependency surface for the required one-shot workflow;
- arbitrary rectangle capture maps directly to the capture primitive;
- selector cancellation remains entirely application-controlled;
- no external screenshot utility;
- multi-monitor and negative-coordinate behavior have a concrete coordinate contract;
- portable Rust core remains unchanged.

Costs:

- the project owns a native Win32 overlay and GDI resource lifecycle;
- mixed-DPI/multi-monitor correctness requires physical validation;
- future protected-content/compositor edge cases may require a second backend;
- monitor stitching is more code than a primary-monitor-only implementation, but it prevents silent multi-monitor clipping.
