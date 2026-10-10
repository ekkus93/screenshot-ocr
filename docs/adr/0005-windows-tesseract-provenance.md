# ADR 0005: Reproducible Windows Tesseract runtime input

**Status:** Proposed; dependency-license review and bundled runtime acceptance pending  
**Date:** 2026-10-10  
**Applies to:** Windows x86-64 application and NSIS installer only

## Context

Screenshot OCR must bundle Tesseract and English language data so an offline
installation needs neither PATH modifications nor a separate OCR installation.
The Windows resolver already requires an app-owned `resources/ocr/tesseract.exe`
and `resources/ocr/tessdata/eng.traineddata`; those paths are not proof that
the actual executable, language file, and transitive DLLs are packaged.

Unpinned downloads, shell-based discovery, and ad hoc copies from a developer
machine are unacceptable for a redistributable installer.

## Proposed reproducible input

- Source: the University of Mannheim Tesseract Windows distribution index,
  <https://digi.bib.uni-mannheim.de/tesseract/>.
- Candidate filename: `tesseract-ocr-w64-setup-5.4.0.20240606.exe`.
- Source URL:
  <https://digi.bib.uni-mannheim.de/tesseract/tesseract-ocr-w64-setup-5.4.0.20240606.exe>.
- Candidate release version: `5.4.0.20240606`.
- **Required SHA-256**, hexadecimal:
  `c885fff6998e0608ba4bb8ab51436e1c6775c2bafc2559a19b423e18678b60c9`.

The digest independently appears in the Chocolatey package verification record
for the same University of Mannheim download:

<https://community.chocolatey.org/packages/tesseract/5.4.0.20240606>.

**Do not execute or extract the downloaded installer until its SHA-256 matches
the pinned digest.** A checksum is an integrity check, not a malware audit or
publisher signature. A trusted-source review and review of third-party library
notices remain required before distribution.

## Packaging contract

The approved installer build must, in a controlled build environment:

1. Fetch only the pinned URL over HTTPS; verify the exact SHA-256.
2. Extract the distribution into disposable build-time storage without
   installing Tesseract globally or mutating machine PATH.
3. Inventory all required runtime DLLs using an actual dependency audit of
   `tesseract.exe`. Preserve required adjacent DLLs in the owned OCR directory.
4. Include English trained data at
   `resources/ocr/tessdata/eng.traineddata`, record its digest, and verify it
   exists in the final NSIS payload. Other languages are not required.
5. Include applicable notices and attribution for Tesseract, Leptonica, and
   native runtime libraries. Review redistribution licenses before approval.
6. Build a self-contained Windows NSIS installer with a stable filename,
   emit an installer SHA-256, and upload both to CI artifacts.
7. Verify, on an offline clean Windows environment, that the extracted
   application finds and runs its own OCR binary and English language data,
   rather than an executable or DLL from current working directory or PATH.

Do not silently auto-install the external University of Mannheim setup as a
separate system-wide requirement, and do not run any network download at
application startup. Package-content validation should fail closed if required
resources are missing, rather than publish a misleading working installer.

## Remaining qualification

The checksum-pinned candidate is a build-input decision, **not an acceptance
claim**. These items remain unproven:

- independent review of the executable and native-library dependency chain;
- transitive DLL and license inventory;
- deterministic extraction method and actual installer resource payload;
- physical Windows 10/11 offline OCR, uninstall, and upgrade behavior;
- final Windows + Linux exact-head CI including NSIS publication gates.

Until those checks pass, the Windows runtime and installer checklist gates stay
unchecked.
