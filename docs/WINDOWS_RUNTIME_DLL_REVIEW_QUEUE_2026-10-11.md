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

The Windows NSIS smoke now emits a content-free `windows-ocr-pe-inventory-<SHA>`
CI artifact with per-binary SHA-256, byte length, static import edges, and
static reachability. These digests can be joined to the separately generated
pinned-input manifest for review without distributing third-party binaries.
They establish file identity only; upstream provenance, dynamic loading,
licenses, and physical acceptance remain independent release gates.

## Additional deterministic payload gate

The staged PE audit now rejects a non-AMD64 or non-PE32+ executable or
adjacent DLL, rather than merely listing its imports. Synthetic fixtures cover
x86 executable and dependency substitution as well as mismatched COFF/optional
headers. This check passed the repository-policy and Windows staging jobs at
exact branch SHA `7af9043c71002237f0aa9a172f41e9a747391a6f` in CI run
`38125955492` (2026-10-11 UTC). It establishes static architecture
compatibility only, not DLL loader safety, completeness, redistribution rights,
or physical Windows behavior.

## Adjacent-DLL filesystem guard (2026-10-11)

The Windows runtime resolver now rejects empty, directory-masquerading, and
NTFS reparse-pointed adjacent `.dll` entries (case-insensitive), and fails
closed if its resource-directory listing is unreadable or exceeds 256 entries.
A synthetic Windows test covers empty files, directory substitution, and
symlink substitution where the runner permits symlink creation. This is a
filesystem layout check only: it does **not** verify DLL hashes, guarantee
native loader behavior, prevent post-check substitution, or establish legal
redistribution rights. Its exact-head CI qualification remains required.

The static PE parser now bounds both ordinary and delay-import descriptor
scans by the declared data-directory size, rejecting a table whose null
terminator is outside that bound. Synthetic fixtures exercise this failure
mode. This is parser-integrity hardening only, not native DLL loader,
provenance, or redistribution approval. Exact-head CI must qualify it.

## Version metadata collection (2026-10-12)

The pinned-input PowerShell audit records bounded numeric PE `FileVersion`
resource strings for the executable and adjacent DLLs when present. Missing,
non-numeric, or unusually long values remain null; the source manifest still
uses SHA-256 as the file-identity key. These are **self-reported binary
metadata**, not verified upstream versions, provenance, licenses, or
redistribution permission. Compare them with authoritative upstream sources
before checking any W18 license or dependency acceptance box. Hosted Windows
packaging CI must qualify the manifest change on its exact commit.

## Preliminary upstream licensing references (2026-10-12)

These references establish **project-level** licensing statements, not the
identity or redistribution rights of any binary extracted from the pinned
third-party Windows installer:

- The upstream Tesseract documentation describes the OCR engine as
  Apache-2.0 licensed:
  https://tesseract-ocr.github.io/tessdoc/Home.html
- Tesseract's own dependency notice states that Leptonica uses essentially
  BSD-2-Clause terms and that other dependencies can carry different licenses:
  https://sources.debian.org/src/tesseract/5.3.0-2/README.md
- SPDX documents that older Leptonica releases used a distinct Leptonica
  license, whereas current Leptonica uses BSD-2-Clause. The actual bundled
  DLL version and corresponding license text must be established:
  https://spdx.github.io/license-list-data/Leptonica.html
- Upstream Tesseract documentation explicitly distinguishes the third-party
  UB Mannheim Windows installer from an official Tesseract Windows installer:
  https://tesseract-ocr.github.io/tessdoc/Downloads.html
- The official traineddata documentation distinguishes the three model sets;
  the source, checksum, and applicable license of the **actual** bundled
  `eng.traineddata` must be matched before publication:
  https://tesseract-ocr.github.io/tessdoc/Data-Files.html

Do not assume that the Mannheim-built `libtesseract-5.dll`,
`libleptonica-6.dll`, or language model is the same release as the upstream
documentation. Resolve each pinned binary to an upstream version and license
record, obtain required copyright/NOTICE texts, and review all other adjacent
DLLs individually. No W18 licensing checkbox is satisfied by this preliminary
research; no installer publication is authorized.

## Unresolved release gates

- [ ] Reproduce final input hashes and decide which DLLs are required.
- [ ] Classify every `external_unreviewed` PE import and loader resolution.
- [ ] Validate Windows DLL search behavior against current-directory hijacking.
- [ ] Include required licenses, attribution, and NOTICE in NSIS resources.
- [ ] Verify the final payload and offline OCR on physical Windows 10/11.

The audit is an inventory only. Keep the W18–W25 redistribution and physical
acceptance gates unchecked. See
[ADR 0005](adr/0005-windows-tesseract-provenance.md).
