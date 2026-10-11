# Windows physical desktop acceptance record (blank template)

**Project:** Screenshot OCR Windows x86-64 candidate  
**Source checklist:** [Windows portability TODO](../SCREENSHOT_OCR_WINDOWS_PORTABILITY_TODO_2026-10-09.md), W23  
**Status:** No physical acceptance results have been recorded in this template.  
**Restriction:** Do not publish or redistribute the Windows installer before the native DLL/license and final release gates pass.

## Test record identity

Create a separate copy of this template for each tested installer build. Record only
non-sensitive system facts; keep real screenshots, OCR text, clipboard contents,
personal file paths, and machine-identifying diagnostics out of the repository.

- Tested **full Git commit SHA**: `NOT TESTED`
- CI run ID(s) for that exact SHA: `NOT TESTED`
- NSIS installer filename (basename only): `NOT TESTED`
- Installer SHA-256: `NOT TESTED`
- OCR runtime manifest provenance/digest verified: `NOT TESTED`
- Windows edition/build (11 or 10 22H2): `NOT TESTED`
- CPU architecture (must be x86-64): `NOT TESTED`
- WebView2 baseline (present, provisioned by installer, or absent): `NOT TESTED`
- Test date and tester reference (non-identifying): `NOT TESTED`
- Result: `NOT TESTED` / `PASS` / `FAIL` / `BLOCKED`

Recompute the downloaded setup executable hash on the **test machine** before
executing it, and compare with the recorded build-time SHA-256:

```powershell
Get-FileHash -Algorithm SHA256 -LiteralPath "<local-installer-path>"
```

A matching checksum proves only artifact identity, not software safety,
redistribution authorization, or clean installation.

## Clean installation and offline boundary

Use a disposable clean Windows test environment that has **not** installed
Tesseract or added it to PATH. Ensure the test only uses approved synthetic,
non-sensitive content. Record `PASS`, `FAIL`, or `BLOCKED` for each row.

| Check | Result | Safe evidence reference |
| --- | --- | --- |
| Installer opens without development tools | NOT TESTED | |
| Clean install succeeds | NOT TESTED | |
| Application starts and main window renders | NOT TESTED | |
| Bundled application-controlled OCR resources exist | NOT TESTED | |
| OCR succeeds without a system Tesseract install | NOT TESTED | |
| Offline install from a locally transferred installer succeeds | NOT TESTED | |
| Offline OCR succeeds without fetching language data | NOT TESTED | |
| WebView2 availability/provisioning is verified | NOT TESTED | |
| Uninstall removes app-owned binaries/resources | NOT TESTED | |
| User-settings retention/removal policy is recorded | NOT TESTED | |
| Same-version reinstall behavior is recorded | NOT TESTED | |

**Offline test:** Transfer the installer to the clean environment, disconnect
networking **before running setup**, and keep it disconnected for the initial
application launch and first English OCR. Distinguish a WebView2 setup failure
from an OCR runtime failure. Do not claim offline installation based on the
hosted NSIS extraction smoke or a machine with preexisting system components.

## Native selection and activation

Use generated public test text, not personal applications or terminal output.
For every item, record only its test status and a synthetic fixture identifier.

| Check | Result | Safe evidence reference |
| --- | --- | --- |
| Capture button opens one selector | NOT TESTED | |
| Click-drag rectangle captures expected pixels | NOT TESTED | |
| Escape cancels selection and restores app | NOT TESTED | |
| App cancellation dismisses a selector | NOT TESTED | |
| Zero-size and tiny regions fail safely | NOT TESTED | |
| Repeated activation does not create overlapping selectors | NOT TESTED | |
| Ctrl+Shift+O works, or registration failure is reported truthfully | NOT TESTED | |
| Tray capture/cancel/show/quit route through shared actions | NOT TESTED | |
| Second-instance capture/cancel/show/quit work | NOT TESTED | |
| Unknown second-instance arguments only focus/show | NOT TESTED | |
| Clipboard immediate and preview/manual retry paths work | NOT TESTED | |
| Quit cleans an active job and leaves no overlay/process | NOT TESTED | |

## Display geometry and OCR fixtures

Test at 100%, 125%, 150%, and 200% scaling. At each scale, verify region
alignment at monitor edges; do not equate pointer/CSS coordinates to GDI
physical pixels. Include a second monitor on the **left/above** primary (negative
virtual coordinates), a second monitor to the right, mixed resolution, and
mixed-DPI configurations. Record whether cross-monitor dragging was actually
observed and whether the captured output agrees with the selected rectangle.

| Scenario | Result | Safe evidence reference |
| --- | --- | --- |
| Single monitor at 100% | NOT TESTED | |
| Single monitor at 125% | NOT TESTED | |
| Single monitor at 150% | NOT TESTED | |
| Single monitor at 200% | NOT TESTED | |
| Two monitors, secondary to the right | NOT TESTED | |
| Two monitors, secondary left/above primary | NOT TESTED | |
| Mixed resolutions | NOT TESTED | |
| Mixed DPI with cross-monitor selection | NOT TESTED | |
| Selection at virtual-desktop edges | NOT TESTED | |
| Light/dark text, browser, terminal and large/small synthetic text | NOT TESTED | |
| Punctuation and whitespace/terminal mode | NOT TESTED | |

## Failure recovery and privacy

On a disposable system only, simulate missing/corrupt packaged runtime or
English language data using a reversible backup of test resources. Restore
the original resources immediately afterward and retest normal startup.

| Check | Result | Safe evidence reference |
| --- | --- | --- |
| Missing runtime yields stable safe public error | NOT TESTED | |
| Missing English data yields stable safe public error | NOT TESTED | |
| Clipboard failure permits manual recovery | NOT TESTED | |
| Native capture failure restores main window | NOT TESTED | |
| No orphan selector or Tesseract process after cancellation | NOT TESTED | |
| No OCR text, clipboard text, screenshot pixels, raw paths, or OS messages in public logs/diagnostics | NOT TESTED | |
| Native DLL loader behavior and app-owned search root verified | NOT TESTED | |

## Windows 10 support decision

If Windows 10 22H2 remains within scope, repeat install, launch, capture,
OCR, shortcut, tray, and uninstall acceptance on that platform. Otherwise
record the supporting compatibility evidence and update the support matrix
before release. Do not silently label Windows 10 supported from CI on a
Windows Server runner.

## Reconciliation and sign-off

- Exact tested commit and checksum confirmed: `NOT TESTED`
- All required tests have objective pass/fail records: `NOT TESTED`
- Native DLL provenance, license, and NOTICE review approved: `NOT TESTED`
- Failure findings linked to code fixes and requalification: `NOT TESTED`
- Final Linux and Windows hosted CI pass on the exact final branch head: `NOT TESTED`
- Repository owner explicitly approved final integration: `NOT TESTED`

Do not mark W23 or W25 complete until this evidence exists for the final
candidate and the owner approves integration to `master`.
