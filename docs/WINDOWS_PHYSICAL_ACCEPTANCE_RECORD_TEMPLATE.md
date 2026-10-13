# Windows physical acceptance record template

**Status: NOT RUN.** Copy this template to a private test record before
testing. It contains no acceptance evidence and does not authorize
redistribution or a merge. Use only synthetic public content; never paste
recognized text, clipboard contents, private paths, screenshots, usernames,
machine identifiers, or raw native errors into this record.

## Exact artifact and environment identity

| Field                                                       | Value (fill before testing) |
| ----------------------------------------------------------- | --------------------------- |
| Branch and exact 40-character commit SHA                    | NOT RECORDED                |
| GitHub Actions CI run ID for that exact SHA                 | NOT RECORDED                |
| NSIS installer filename                                     | NOT RECORDED                |
| Installer SHA-256 from qualified build                      | NOT RECORDED                |
| Independently computed SHA-256 of local installer           | NOT RECORDED                |
| Native DLL provenance and license/NOTICE review approved?   | NO                          |
| Windows edition and version (11 or 10 22H2)                 | NOT RECORDED                |
| Physical desktop or interactive VM; screen count            | NOT RECORDED                |
| Display scales, monitor order, and resolutions (no serials) | NOT RECORDED                |
| Network disconnected throughout installation/OCR?           | NOT RECORDED                |
| Reviewer, review date, and release authorization            | NOT RECORDED                |

**Stop before running the installer** if its locally calculated SHA-256
does not match the qualified build, its origin is not approved, or the
native dependency license/NOTICE review is incomplete. Record only the
reason category; do not record raw private diagnostic data.

## Test recording convention

For each case, replace `NOT RUN` with `PASS`, `FAIL`, or
`BLOCKED`. Attach a short content-free observation and the exact
artifact/commit identity above. A hosted CI pass does not change these
physical-test results. Do not mark W23 TODO checkboxes until the relevant
cases have passed on the required Windows systems.

## Windows 11: offline clean-install and core flow

| Case  | Expected behavior                                                                       | Result  | Safe observation |
| ----- | --------------------------------------------------------------------------------------- | ------- | ---------------- |
| 11-01 | Clean, disconnected NSIS installation succeeds without developer tools                  | NOT RUN |                  |
| 11-02 | WebView2 prerequisite is provisioned offline where absent                               | NOT RUN |                  |
| 11-03 | Application launches and settings/diagnostics load without private details              | NOT RUN |                  |
| 11-04 | UI capture selects LIGHT-01 and OCR succeeds locally                                    | NOT RUN |                  |
| 11-05 | DARK-02 and browser text produce reviewable results                                     | NOT RUN |                  |
| 11-06 | Preview-before-copy requires deliberate copy and clipboard receives correct public text | NOT RUN |                  |
| 11-07 | Immediate-copy success and recoverable clipboard failure are truthful                   | NOT RUN |                  |
| 11-08 | Escape cancels selection without leaving an overlay                                     | NOT RUN |                  |
| 11-09 | Shared Cancel action dismisses selection; a new capture succeeds                        | NOT RUN |                  |
| 11-10 | Native capture failure restores the window and exposes only safe error codes            | NOT RUN |                  |
| 11-11 | Application quits cleanly with no orphaned native resources                             | NOT RUN |                  |
| 11-12 | Uninstall removes app binaries, bundled OCR files, and shortcuts                        | NOT RUN |                  |
| 11-13 | Uninstall preserves app-owned settings but leaves unrelated profile data untouched      | NOT RUN |                  |
| 11-14 | Reinstall preserves valid settings; corrupt settings recover safely                     | NOT RUN |                  |
| 11-15 | No network access or Tesseract/PATH dependency was needed for OCR                       | NOT RUN |                  |

## Activation and lifecycle

| Case   | Expected behavior                                                               | Result  | Safe observation |
| ------ | ------------------------------------------------------------------------------- | ------- | ---------------- |
| ACT-01 | Ctrl+Shift+O activates capture or reports nonfatal registration failure         | NOT RUN |                  |
| ACT-02 | Rapid repeated shortcut cannot create overlapping selectors                     | NOT RUN |                  |
| ACT-03 | Tray Capture, Cancel, Show, and Quit use shared action semantics                | NOT RUN |                  |
| ACT-04 | Second-instance capture/cancel/show route correctly                             | NOT RUN |                  |
| ACT-05 | Unknown second-instance arguments only show/focus; no arbitrary command runs    | NOT RUN |                  |
| ACT-06 | Window hide/restore and focus are correct on success, cancellation, and failure | NOT RUN |                  |

## Scaling, monitor geometry, and synthetic content

Keep browser zoom at 100%; vary Windows display scaling. For each
scaling value, test EDGE-06 and GRID-07 near display boundaries. Use a
second display to exercise negative coordinates and cross-monitor drags.

| Case    | Expected behavior                                                              | Result  | Safe observation |
| ------- | ------------------------------------------------------------------------------ | ------- | ---------------- |
| DPI-100 | 100% selection aligns with captured pixels                                     | NOT RUN |                  |
| DPI-125 | 125% selection aligns with captured pixels                                     | NOT RUN |                  |
| DPI-150 | 150% selection aligns with captured pixels                                     | NOT RUN |                  |
| DPI-200 | 200% selection aligns with captured pixels                                     | NOT RUN |                  |
| MON-01  | Secondary display right of primary                                             | NOT RUN |                  |
| MON-02  | Secondary display left/above primary; negative virtual coordinates             | NOT RUN |                  |
| MON-03  | Different resolutions and mixed DPI preserve geometry                          | NOT RUN |                  |
| MON-04  | Cross-monitor drag yields correct pixels, or an explicit documented limitation | NOT RUN |                  |
| TXT-01  | LIGHT-01 and DARK-02 light/dark contrast                                       | NOT RUN |                  |
| TXT-02  | MONO-03 terminal/monospace whitespace                                          | NOT RUN |                  |
| TXT-03  | SIZE-04 small and large text                                                   | NOT RUN |                  |
| TXT-04  | PUNCT-05 punctuation and code symbols                                          | NOT RUN |                  |
| TXT-05  | EDGE-06 and GRID-07 edge/alignment accuracy                                    | NOT RUN |                  |

## Recovery and privacy

| Case   | Expected behavior                                                                     | Result  | Safe observation |
| ------ | ------------------------------------------------------------------------------------- | ------- | ---------------- |
| ERR-01 | Missing/corrupt packaged OCR executable reports safe failure                          | NOT RUN |                  |
| ERR-02 | Missing/corrupt English language data reports safe failure                            | NOT RUN |                  |
| ERR-03 | Clipboard write failure preserves recognized text for retry                           | NOT RUN |                  |
| ERR-04 | Cancellation and native failure restore UI and release selector                       | NOT RUN |                  |
| ERR-05 | Diagnostics/logs contain no screenshot, OCR, clipboard, profile or installation paths | NOT RUN |                  |
| ERR-06 | Offline operation never downloads OCR language data                                   | NOT RUN |                  |
| ERR-07 | Cancel during an active OCR subprocess; child is reaped and UI can capture again      | NOT RUN |                  |

## Windows 10 22H2 support decision

If Windows 10 is retained, run the same exact installer on a real
interactive Windows 10 22H2 session and record installation, launch,
capture, OCR, shortcut, tray, and uninstall separately. Otherwise,
document a supported-version decision and update the spec/support matrix
_before_ removing Windows 10 from the release claim.

| Case   | Expected behavior                                      | Result  | Safe observation |
| ------ | ------------------------------------------------------ | ------- | ---------------- |
| W10-01 | Installation and offline WebView2 prerequisites        | NOT RUN |                  |
| W10-02 | Launch, native selection, OCR, and clipboard           | NOT RUN |                  |
| W10-03 | Ctrl+Shift+O and tray activation                       | NOT RUN |                  |
| W10-04 | Uninstall and app-owned settings retention             | NOT RUN |                  |
| W10-05 | Windows 10 support decision is documented and approved | NOT RUN |                  |

## Final reconciliation

Record the exact tested SHA, installer checksum match, all failed or
blocked case IDs, whether any fixes changed the head, and the final
reviewer's explicit approval. A code change invalidates previous
exact-head qualification until retested. Leave W23/W25 open if any
required physical test, native license review, or owner authorization
remains missing. Never publish or merge automatically from this template.
