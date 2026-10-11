# Windows OCR native binary redistribution review queue

**Status:** preliminary evidence; **no component is approved for redistribution**  
**Evidence:** Windows NSIS smoke run `38118651508`, job `114409746175`,
at exact commit `ecbe361f9fb7cf395a5b8be17494cd4384a35912` (2026-10-11 UTC).  
**Input:** pinned Mannheim Tesseract 5.4.0.20240606 candidate from
[ADR 0005](adr/0005-windows-tesseract-provenance.md).

## Scope and limits

The CI runtime staging step reported **51 adjacent DLL files**. The static
PE import audit enumerated **52 PE binaries including `tesseract.exe`**.
This list is extracted from the audit log, not independently verified against
a clean installed system. It may contain libraries not actually needed by the
application. Do not infer licenses from DLL filenames, assume a library is
safe merely because it appears in the installer, or treat a successful
synthetic OCR invocation as proof of redistribution compliance.

## Per-binary review queue

For each entry, record the actual binary version/hash, upstream source,
applicable license and copyright, required NOTICE/attribution text, and
whether the binary is truly needed in the packaged runtime. All entries
remain open until supported by evidence.

- [ ] `libarchive-13.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libb2-1.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libbrotlicommon.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libbrotlidec.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libbz2-1.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libcairo-2.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libcrypto-3-x64.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libdatrie-1.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libdeflate.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libexpat-1.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libffi-8.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libfontconfig-1.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libfreetype-6.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libfribidi-0.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libgcc_s_seh-1.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libgif-7.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libgio-2.0-0.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libglib-2.0-0.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libgmodule-2.0-0.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libgobject-2.0-0.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libgraphite2.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libharfbuzz-0.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libiconv-2.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libicudt75.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libicuin75.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libicuuc75.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libintl-8.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libjbig-0.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libjpeg-8.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libleptonica-6.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libLerc.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `liblz4.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `liblzma-5.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libopenjp2-7.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libpango-1.0-0.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libpangocairo-1.0-0.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libpangoft2-1.0-0.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libpangowin32-1.0-0.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libpcre2-8-0.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libpixman-1-0.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libpng16-16.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libsharpyuv-0.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libstdc++-6.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libtesseract-5.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libthai-0.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libtiff-6.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libwebp-7.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libwebpmux-3.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libwinpthread-1.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `libzstd.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `tesseract.exe` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.
- [ ] `zlib1.dll` — identify exact version, upstream source, applicable license/NOTICE, and redistribution obligations.

## Release-blocking reconciliation

- [ ] Reproduce this inventory from the final pinned build input and record
  exact SHA-256 values for each retained binary.
- [ ] Identify which of the 51 adjacent DLLs are actually required and
  remove only demonstrably unnecessary DLLs without breaking offline OCR.
- [ ] Classify every `external_unreviewed` import and establish that it
  resolves safely on supported clean Windows installations.
- [ ] Review the transitive DLL dependency graph and Windows loader search
  behavior; reject arbitrary current-directory/PATH resolution.
- [ ] Package all required third-party licenses, attribution, and NOTICE
  materials in the actual NSIS payload.
- [ ] Verify final payload checksums and physical offline OCR on Windows
  10/11 before publishing a release or CI distribution artifact.

**This is a review queue, not a license determination.** Keep W18, W19,
W20, and W25 redistribution/physical acceptance gates unchecked until
each required piece of evidence exists.
