# Windows native capture and packaging audit

**Branch:** `ralph/windows-support`  
**Observed head:** `130c0326ea26ce38f7a8763ef022994585a94e3d`  
**Hosted CI:** run `38104602131` (2026-10-11 UTC), successful on the exact observed head.

## Implementation evidence

The native Windows selector uses physical virtual-desktop coordinates and a
Win32 overlay. The GDI capture adapter enumerates physical monitor rectangles,
captures each selected monitor intersection, and composes the resulting pixels
at their virtual-desktop offsets. Pure tests exercise negative origins,
selection bounds, invalid tile placement, and color conversion.

The previous Windows Clippy errors in `monitors.rs` (invalid `BOOL` import
and missing `null` import) were corrected before the successful exact-head
CI run. All seven jobs passed: Windows Rust/build/link, Windows NSIS smoke,
Linux Rust/build, Debian packaging, frontend quality, Playwright layout,
and repository policy.

The NSIS smoke extracts the installer as data and checks that the app-relative
`resources/ocr` directory matches the runtime resolver. It verifies the
checksum-pinned Tesseract executable, English language data, and each adjacent
DLL against the generated manifest, and prints an installer SHA-256. It does
not execute the installer or Tesseract and does not publish the installer.

## Remaining acceptance boundaries

Hosted Windows runners do not establish that the selector receives Escape,
that capture works with mixed-DPI/negative-coordinate monitors, that the
overlay is absent from captured pixels, or that cancellation and focus restore
work on a physical desktop. These require the W23 physical Windows matrix.

The pinned Mannheim Tesseract input and adjacent DLL inventory are not a
completed transitive dependency, redistribution-license, or DLL loader search
audit. Physical offline OCR, installation, upgrade, and uninstall remain
unverified. Keep corresponding W18-W25 gates unchecked until evidenced.

`master` remains at `bbfd605d5c65b83735a9795622d92062c6c9c59d`;
this work has not been merged.
