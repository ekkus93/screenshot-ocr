# Threat model

## Protected assets

Captured pixels, recognized text, clipboard contents, terminal secrets, filesystem integrity, command-execution boundaries, settings integrity, desktop-session state, and CI/test artifacts.

## Principal threats and mitigations

| Threat                                       | Mitigation                                                                                                                                                       | TODO mapping |
| -------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ |
| Screen content contains shell metacharacters | No shell invocation; fixed process arguments                                                                                                                     | M6.2, M13.3  |
| Replaced helper executable                   | PATH discovery only at startup, no frontend-provided path; diagnostics expose availability but not path                                                          | M5.2         |
| Symlink or temporary-file race               | Private random directory, random filename, application-owned marker, post-capture regular-file/symlink checks, immediate deletion, marker-gated stale scavenging | M6.1–M6.3    |
| Malformed/oversized image                    | Encoded byte and decoded dimension/pixel limits before full decode; generated preprocessing variants are rechecked before resize                                  | M8.1, M13.4  |
| OCR language probing hangs or floods output  | Bounded async probe, no shell, null stdin/stderr, stdout limit, timeout, cancellation, and child kill/reap                                                        | M9.3, M13.3  |
| OCR recognition hangs                        | OCR subprocess timeout and child cleanup                                                                                                                         | M9.3, M13.3  |
| Sensitive data enters logs                   | Content-free errors/diagnostics; no OCR stdout/stderr logging; source leakage guard; synthetic-secret tests                                                      | M3.2, M13.1  |
| Frontend requests arbitrary execution        | Typed DTOs, fixed language and backend enums, no path/argument fields                                                                                            | M11.1        |
| Overlapping selectors                        | Rust-owned capture state machine rejects a second job; toggle action cancels the active job instead of starting another selector                                  | M2.3, M12.2  |
| Corrupt settings silently discarded          | Corrupt file is quarantined and a safe recovery warning is returned to the UI                                                                                     | M3.1         |
| Clipboard mutation on failure                | Clipboard command rejects empty text; immediate-copy clipboard failure returns recognized text with `copied = false` and a retryable warning                      | M10.2        |
| Test fixture leaks private data              | OCR cleanup fixtures must be synthetic text; private screenshots, real OCR output, clipboard text, temporary paths, portal URIs, and helper output are disallowed | M13.1, M14   |
| CI publishes content-bearing artifacts       | CI should upload only reviewed package/checksum artifacts; test screenshots and OCR output artifacts remain disallowed                                           | M14, M16     |

## Residual risks

The GNOME screenshot helper, Tesseract, the desktop clipboard, and tray/shortcut integrations are trusted system dependencies. Physical validation is required for helper cancellation semantics, clipboard ownership, Wayland/X11 behavior, tray behavior, global shortcut behavior, package install/remove behavior, multi-monitor behavior, and fractional scaling.

Portal capture is not enabled until explicit area-target capability is implemented and proven. FIX1 documents the current portal lifecycle decision but does not replace physical GNOME portal validation.

Third-party GitHub Actions are not yet pinned by immutable commit SHA. That remains a CI/security hardening decision rather than a completed release gate.
