# Privacy

Screenshot OCR performs OCR locally. It does not upload screenshots or recognized text and does not require network access for capture or OCR.

The GNOME compatibility backend temporarily asks `gnome-screenshot` to write one image inside a private, randomly named runtime directory. The image is read, deleted immediately, decoded in application memory, and the directory is removed. OCR preprocessing stays in memory, and PNG data is sent to Tesseract through standard input.

GNOME capture directories created by the application contain a static `.screenshot-ocr-owned` marker so startup cleanup can distinguish application-owned stale directories from unrelated files. The marker is content-free and must not contain usernames, screenshot text, paths, command arguments, timestamps, or OCR output.

Version 0.1 has no persistent OCR history. Settings contain preferences only. Corrupt settings are quarantined and recovered with defaults through a safe warning. That warning is intended to include only stable code, message, guidance, and recovery status; it must not include filesystem paths, raw settings JSON, or user content.

Diagnostics include application version, operating-system/desktop/session labels, dependency availability, installed OCR language codes, shortcut/tray status, and safe counters. Diagnostics exclude screenshots, recognized text, clipboard contents, executable paths, temporary paths, portal result URIs, raw helper stderr, and raw helper stdout.

The clipboard changes only after an explicit copy action in preview mode or after successful non-empty OCR in immediate-copy mode. Cancellation, empty output, and OCR failure must leave the clipboard unchanged. If OCR succeeds but an immediate clipboard write fails, the recognized text is returned to the active UI for review/retry with `copied = false`; the app must not claim that the text was copied.

Automated OCR cleanup fixtures are synthetic text only. Private screenshots, real OCR output, clipboard text, temporary paths, portal result URIs, and helper stdout/stderr from a real machine must not be committed as test fixtures or uploaded as CI artifacts.

System components involved are GNOME screenshot facilities, Tesseract, the desktop clipboard, and Tauri/WebKitGTK. Their operating-system behavior remains outside the application's direct control and is covered by the required platform validation matrix.

## Windows candidate privacy boundary

The experimental Windows adapter selects a region with an application-owned Win32 overlay, captures pixels into memory with GDI, and passes the image to the same local preprocessing and OCR pipeline. The selector does not invoke external screenshot utilities, write image files, or send captured pixels over the network. Cancellation and capture failures must release native handles and restore the application window; physical desktop acceptance is still pending.

The packaged Windows Tesseract candidate runs from the application's own `resources/ocr/` directory with English trained data alongside it. Its path is resolved internally, not supplied by frontend input or discovered from PATH. Runtime discovery and recognition errors map to stable public codes rather than raw Windows error text, filesystem paths, or process output.

Build-time CI may download a checksum-pinned Tesseract distribution and extract it to inspect or stage a candidate installer; this is separate from application runtime and must not upload private captures. Transitive native dependency/license review, installer publication approval, and offline physical acceptance remain incomplete.
