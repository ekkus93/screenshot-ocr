# Threat model

## Protected assets

Captured pixels, recognized text, clipboard contents, terminal secrets, filesystem integrity, command-execution boundaries, settings integrity, desktop-session state, and CI/test artifacts.

## Principal threats and mitigations

| Threat                                       | Mitigation                                                                                                                                                        | TODO mapping |
| -------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ |
| Screen content contains shell metacharacters | No shell invocation; fixed process arguments                                                                                                                      | M6.2, M13.3  |
| Replaced helper executable                   | PATH discovery only at startup, no frontend-provided path; diagnostics expose availability but not path                                                           | M5.2         |
| Symlink or temporary-file race               | Private random directory, random filename, application-owned marker, post-capture regular-file/symlink checks, immediate deletion, marker-gated stale scavenging  | M6.1–M6.3    |
| Malformed/oversized image                    | Encoded byte and decoded dimension/pixel limits before full decode; generated preprocessing variants are rechecked before resize                                  | M8.1, M13.4  |
| OCR language probing hangs or floods output  | Bounded async probe, no shell, null stdin/stderr, stdout limit, timeout, cancellation, and child kill/reap                                                        | M9.3, M13.3  |
| OCR recognition hangs                        | OCR subprocess timeout and child cleanup                                                                                                                          | M9.3, M13.3  |
| Sensitive data enters logs                   | Content-free errors/diagnostics; no OCR stdout/stderr logging; source leakage guard; synthetic-secret tests                                                       | M3.2, M13.1  |
| Frontend requests arbitrary execution        | Typed DTOs, fixed language and backend enums, no path/argument fields                                                                                             | M11.1        |
| Overlapping selectors                        | Rust-owned capture state machine rejects a second job; toggle action cancels the active job instead of starting another selector                                  | M2.3, M12.2  |
| Corrupt settings silently discarded          | Corrupt file is quarantined and a safe recovery warning is returned to the UI                                                                                     | M3.1         |
| Clipboard mutation on failure                | Clipboard command rejects empty text; immediate-copy clipboard failure returns recognized text with `copied = false` and a retryable warning                      | M10.2        |
| Test fixture leaks private data              | OCR cleanup fixtures must be synthetic text; private screenshots, real OCR output, clipboard text, temporary paths, portal URIs, and helper output are disallowed | M13.1, M14   |
| CI publishes content-bearing artifacts       | CI should upload only reviewed package/checksum artifacts; test screenshots and OCR output artifacts remain disallowed                                            | M14, M16     |

## Residual risks

The GNOME screenshot helper, Tesseract, the desktop clipboard, and tray/shortcut integrations are trusted system dependencies. Physical validation is required for helper cancellation semantics, clipboard ownership, Wayland/X11 behavior, tray behavior, global shortcut behavior, package install/remove behavior, multi-monitor behavior, and fractional scaling.

Portal capture is not enabled until explicit area-target capability is implemented and proven. FIX1 documents the current portal lifecycle decision but does not replace physical GNOME portal validation.

Third-party GitHub Actions are not yet pinned by immutable commit SHA. That remains a CI/security hardening decision rather than a completed release gate.

## Windows candidate threats and residual risk

| Threat                                                                             | Current mitigation                                                                                                           | Unverified acceptance                                                                               |
| ---------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| Selector overlays remain open or leak capture resources                            | One Rust-owned capture job; cancellation token polled by native selector; Win32 handles owned and released                   | Escape, tray cancellation, quit, failure recovery, and real desktop lifecycle                       |
| DPI virtualization, negative coordinates, and monitor gaps corrupt captured pixels | Per-Monitor-v2 worker context, bounded physical virtual-screen rectangles, per-monitor tiles and pure geometry tests         | Mixed-DPI and multi-monitor physical selection/capture accuracy                                     |
| Untrusted executable or DLL resolution                                             | App-owned `resources/ocr/tesseract.exe`, adjacent staged DLLs, internal resolver; no frontend executable path or PATH search | Full transitive dependency/DLL-search and license audit; clean-install offline execution            |
| Tampered OCR build input or incomplete NSIS bundle                                 | Pinned source installer SHA-256 and hash inventory; CI extracts candidate without executing setup                            | Exact final installer payload verification, independent provenance review, and publication approval |
| Raw Windows errors or screen content escape diagnostics                            | Stable public error taxonomy, content-free platform statuses, no screenshot/OCR/clipboard payload logging                    | Physical failure-path and diagnostic audit                                                          |
| Windows WebView2 bootstrap requires network                                        | No runtime network dependency for OCR itself                                                                                 | Verify installer bootstrap/offline provisioning policy on clean Windows 10/11                       |

### Windows native loader and uninstall review

The Windows OCR resolver constructs an absolute path under the application's
own `resources/ocr/` directory and never accepts a frontend path or searches
`PATH`. The shared OCR command sets its child working directory to that
resource directory. For packaged OCR it also removes inherited PATH and
native configuration/plugin search overrides (GLib/GDK, Fontconfig, OpenSSL,
ICU, Qt and TESSDATA_PREFIX), then sets TESSDATA_PREFIX to the internally
resolved English-data directory. A deterministic child-command test checks
this policy without reading or changing the user's environment. System OCR
on Linux retains its existing environment behavior. This reduces exposure
to caller-controlled native configuration paths but does not prove the
Windows loader cannot search untrusted directories or load dynamic modules. The Windows resolver also rejects filesystem reparse
points (including NTFS symlinks/junctions) in the `resources/ocr` and
`tessdata` directories and the selected executable/language files, so those
paths cannot silently redirect outside the installed OCR tree. This is a
fail-closed _pre-launch path check_, not a defense against modifications
between check and execution or DLL files redirected independently.
These choices reduce accidental execution of unrelated
Tesseract binaries, but do **not** prove Windows DLL loader resolution:
application-directory precedence, system DLLs, delay-loads, dynamic
`LoadLibrary` calls, install-directory permissions, and DLL side-loading
still require a full audit on supported Windows versions. Static PE import
inventory is prioritization evidence only, not proof of safe runtime loading
or redistribution licensing. Keep the installer unpublished until reviewed.

Windows uninstall should remove installed binaries and OCR resources but
retain per-user Tauri configuration by default; neither uninstall behavior
nor reinstall/upgrade persistence has been physically validated. Uninstall
must never recursively remove unrelated user-profile data.

The Windows NSIS build is a **non-published smoke candidate**, not a qualified release. No code signing or physical Windows acceptance is claimed.
