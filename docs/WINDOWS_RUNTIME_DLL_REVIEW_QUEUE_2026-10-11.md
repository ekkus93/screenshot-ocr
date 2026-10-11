# Windows OCR binary redistribution review queue

**Status:** preliminary inventory, not redistribution approval.  
**Evidence:** CI run `38118651508`, job `114409746175`, commit
`ecbe361f9fb7cf395a5b8be17494cd4384a35912` (2026-10-11 UTC).

The staged Mannheim Tesseract candidate contains 51 adjacent DLLs and
`tesseract.exe`. The static PE audit identified these 52 binaries. Their
presence does not prove they are all needed, safe, or legally redistributable.

## Binaries requiring individual review

```text
libarchive-13.dll
libb2-1.dll
libbrotlicommon.dll
libbrotlidec.dll
libbz2-1.dll
libcairo-2.dll
libcrypto-3-x64.dll
libdatrie-1.dll
libdeflate.dll
libexpat-1.dll
libffi-8.dll
libfontconfig-1.dll
libfreetype-6.dll
libfribidi-0.dll
libgcc_s_seh-1.dll
libgif-7.dll
libgio-2.0-0.dll
libglib-2.0-0.dll
libgmodule-2.0-0.dll
libgobject-2.0-0.dll
libgraphite2.dll
libharfbuzz-0.dll
libiconv-2.dll
libicudt75.dll
libicuin75.dll
libicuuc75.dll
libintl-8.dll
libjbig-0.dll
libjpeg-8.dll
libleptonica-6.dll
libLerc.dll
liblz4.dll
liblzma-5.dll
libopenjp2-7.dll
libpango-1.0-0.dll
libpangocairo-1.0-0.dll
libpangoft2-1.0-0.dll
libpangowin32-1.0-0.dll
libpcre2-8-0.dll
libpixman-1-0.dll
libpng16-16.dll
libsharpyuv-0.dll
libstdc++-6.dll
libtesseract-5.dll
libthai-0.dll
libtiff-6.dll
libwebp-7.dll
libwebpmux-3.dll
libwinpthread-1.dll
libzstd.dll
tesseract.exe
zlib1.dll
```

For **each** binary, identify the actual version, SHA-256, upstream origin,
license/copyright, NOTICE obligations, and whether it is required. Do not
infer licensing or DLL loader safety from the filename alone.

## Additional deterministic payload gate

The staged PE audit now rejects a non-AMD64 or non-PE32+ executable or
adjacent DLL, rather than merely listing its imports. Synthetic fixtures cover
x86 executable and dependency substitution as well as mismatched COFF/optional
headers. This check passed the repository-policy and Windows staging jobs at
exact branch SHA `7af9043c71002237f0aa9a172f41e9a747391a6f` in CI run
`38125955492` (2026-10-11 UTC). It establishes static architecture
compatibility only, not DLL loader safety, completeness, redistribution rights,
or physical Windows behavior.

## Unresolved release gates

- [ ] Reproduce final input hashes and decide which DLLs are required.
- [ ] Classify every `external_unreviewed` PE import and loader resolution.
- [ ] Validate Windows DLL search behavior against current-directory hijacking.
- [ ] Include required licenses, attribution, and NOTICE in NSIS resources.
- [ ] Verify the final payload and offline OCR on physical Windows 10/11.

The audit is an inventory only. Keep the W18–W25 redistribution and physical
acceptance gates unchecked. See
[ADR 0005](adr/0005-windows-tesseract-provenance.md).
