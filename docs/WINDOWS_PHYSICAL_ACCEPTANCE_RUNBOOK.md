# Windows physical acceptance record

Use only synthetic, non-sensitive on-screen text. This runbook does not
substitute for physical Windows acceptance or authorize installer publication.

## Evidence identity

Record the exact 40-character branch commit SHA, CI run ID, candidate
installer filename and SHA-256, Windows version/build, display scaling,
monitor arrangement, and whether networking was disconnected. Confirm the
installer checksum before executing it. Do not upload screenshots containing
personal information, OCR text, clipboard data, user paths, or machine IDs.

## Offline synthetic test fixture

Open `docs/fixtures/windows-ocr-acceptance.html` directly from disk in a
browser at **100% browser zoom**, without a network connection. It contains
no JavaScript, remote assets, personal information, or system-specific paths.
Use fixture IDs LIGHT-01, DARK-02, MONO-03, SIZE-04, PUNCT-05,
EDGE-06, and GRID-07 to record content-free pass/fail observations.
The repeated `SAMPLE 123` marker provides a simple recognition sanity
check; evaluate whitespace/punctuation and selection alignment separately.
Change **Windows display scaling** independently of browser zoom, and move
the browser window between monitors for mixed-DPI and cross-monitor tests.
The fixture is test input, not evidence that any test has passed.

## Clean-install exercise

On a disposable clean Windows 11 x64 desktop or VM with a real interactive
session, disconnect networking, run the unsigned NSIS candidate only after
local trust review, launch the application, select a synthetic text region,
review OCR before copying, verify clipboard output, cancel with Escape,
repeat capture, exercise tray and Control+Shift+O actions, then uninstall.
Record each result as pass/fail/not run with a content-free explanation.
Check that bundled resources are removed but app-owned settings persist.

## Display and recovery matrix

Repeat region selection at 100%, 125%, 150%, and 200% scaling. Include
monitors left and right of primary, negative virtual coordinates, mixed DPI,
and a cross-monitor drag. Check that selected pixels match the rectangle,
that cancellation releases the selector, and that failed capture or clipboard
operations restore the main window. Exercise second-instance commands and
rapid repeated shortcuts without allowing overlapping selectors.

## Windows 10

Repeat install, launch, capture, OCR, shortcut, tray, and uninstall on
Windows 10 22H2 if that OS remains in the support matrix. Otherwise record
the compatibility evidence and update the supported-version claim before
release.

## Completion boundary

Keep W23 and final merge-readiness boxes unchecked until observed results
are recorded against the exact tested commit. Complete native dependency
license and NOTICE review before distributing or publishing any installer.
Final integration to master requires separate repository-owner approval.
