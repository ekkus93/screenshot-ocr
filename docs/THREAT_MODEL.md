# Threat model

## Protected assets

Captured pixels, recognized text, clipboard contents, terminal secrets, filesystem integrity, command-execution boundaries, and settings integrity.

## Principal threats and mitigations

| Threat                                       | Mitigation                                                                                              | TODO mapping |
| -------------------------------------------- | ------------------------------------------------------------------------------------------------------- | ------------ |
| Screen content contains shell metacharacters | No shell invocation; fixed process arguments                                                            | M6.2, M13.3  |
| Replaced helper executable                   | PATH discovery only at startup, no frontend-provided path; diagnostics expose availability but not path | M5.2         |
| Symlink or temporary-file race               | Private random directory, random filename, post-capture regular-file/symlink checks, immediate deletion | M6.1–M6.3    |
| Malformed/oversized image                    | Encoded byte and decoded dimension/pixel limits before full decode                                      | M8.1, M13.4  |
| OCR hangs or floods output                   | 30-second timeout, fixed stdout size limit, child killed on drop                                        | M9.3, M13.3  |
| Sensitive data enters logs                   | Content-free errors/diagnostics; no OCR stdout/stderr logging; synthetic-secret tests                   | M3.2, M13.1  |
| Frontend requests arbitrary execution        | Typed DTOs, fixed language and backend enums, no path/argument fields                                   | M11.1        |
| Overlapping selectors                        | Rust-owned capture state machine rejects a second job                                                   | M2.3, M12.2  |
| Corrupt settings silently discarded          | Corrupt file quarantined and visible error returned                                                     | M3.1         |
| Clipboard mutation on failure                | Clipboard command is separate and empty text is rejected                                                | M10.2        |

## Residual risks

The GNOME screenshot helper and Tesseract are trusted system dependencies. Physical validation is required for helper cancellation semantics, clipboard ownership, Wayland/X11 behavior, tray integration, and fractional scaling. Portal capture is not enabled until explicit area-target capability is implemented and proven.
