# Privacy

Screenshot OCR performs OCR locally. It does not upload screenshots or recognized text and does not require network access for capture or OCR.

The GNOME compatibility backend temporarily asks `gnome-screenshot` to write one image inside a private, randomly named runtime directory. The image is read, deleted immediately, decoded in application memory, and the directory is removed. OCR preprocessing stays in memory, and PNG data is sent to Tesseract through standard input.

Version 0.1 has no persistent OCR history. Settings contain preferences only. Diagnostics include application version, operating-system/desktop/session labels, dependency availability, installed OCR language codes, and safe counters. Diagnostics exclude screenshots, recognized text, clipboard contents, executable paths, and temporary paths.

The clipboard changes only after an explicit copy action in preview mode or after successful non-empty OCR in immediate-copy mode. Cancellation, empty output, and OCR failure must leave the clipboard unchanged.

System components involved are GNOME screenshot facilities, Tesseract, the desktop clipboard, and Tauri/WebKitGTK. Their operating-system behavior remains outside the application's direct control and is covered by the required platform validation matrix.
