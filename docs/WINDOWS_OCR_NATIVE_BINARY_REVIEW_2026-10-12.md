# Windows OCR native binary and license review register

**Status: PRELIMINARY INVENTORY — REDISTRIBUTION NOT APPROVED.**
Derived from the static PE inventory printed by passing
[CI run 38260144405](https://github.com/ekkus93/screenshot-ocr/actions/runs/38260144405)
at exact commit `93251891f2f542661e7ebb5457663bf8b12b95af`,
Windows NSIS packaging smoke job `114857787618` (October 12, 2026).
This is staged input evidence, not a release approval, installed-desktop
test, or guarantee that a later build uses identical bytes.

**Pinned source:** Mannheim Tesseract `5.4.0.20240606`,
installer SHA-256 `c885fff6998e0608ba4bb8ab51436e1c6775c2bafc2559a19b423e18678b60c9`.
The CI input audit reported **51 adjacent DLLs plus tesseract.exe**.
The static import graph identified **27 of 52 binaries as reachable**
(including the executable) and **25 as not statically reachable**.
Dynamic/optional loads may require additional DLLs. Do not remove files
merely because they are absent from the static reachability graph.

## Packaged binary review register

Every row needs an independent exact-version, origin, license, attribution,
NOTICE, dynamic-load, and redistribution review. Hashes identify only the
staged binaries in the cited CI run.

| Binary | Statically reachable | Bytes | SHA-256 in cited CI | License / NOTICE |
| --- | --- | ---: | --- | --- |
| `libarchive-13.dll` | YES | 769154 | `98b9b7e9259f23eab6749ee25cca2c0643f09df6b23c58229fd43bf6249195e9` | UNREVIEWED |
| `libb2-1.dll` | YES | 34372 | `7fb8418a1172042dab5227fcd466b1bf293737f5b5cf35359e851516227f109c` | UNREVIEWED |
| `libbrotlicommon.dll` | NO | 143397 | `62f363f6aac0d54ce7b115c0412b19e0fe5480f4e6fc1254f663b0ab6868e46f` | UNREVIEWED |
| `libbrotlidec.dll` | NO | 60412 | `7d6c0f954b6f2db1c1c5fd5904886ae055859a02ca31b6dde5f96f6b0259d23f` | UNREVIEWED |
| `libbz2-1.dll` | YES | 100964 | `b79cd3a7102e359ef6324e98966a67939e58ec2b651ae2847cae84a4dfa453f7` | UNREVIEWED |
| `libcairo-2.dll` | NO | 1213715 | `c531e50be733f641e39419bf62cb799698b20086afb958d73f56677ea0c3f9da` | UNREVIEWED |
| `libcrypto-3-x64.dll` | YES | 5130880 | `c0674a225d30f1642ca1da45ac040a9c1885d8f23883532b42987bae458edc4d` | UNREVIEWED |
| `libdatrie-1.dll` | NO | 34386 | `f760aef8a31b7c3670484ca4eabab1ab8e95c14052167e191463f6371c3f0850` | UNREVIEWED |
| `libdeflate.dll` | YES | 91035 | `b6ee7b31a23605d8e8b9be5eae782a3ddf7adf2b7ca2c8e0041bdfd4fe4659ee` | UNREVIEWED |
| `libexpat-1.dll` | YES | 188909 | `d9ea333175b1e995bbe6a062567f38ff362f1b2f369fa8ebdb0eeed21813bdc6` | UNREVIEWED |
| `libffi-8.dll` | NO | 33708 | `f1ed51652b3746c1b4bce497b7041e677e5d80c5ab8198efebb8fc6f75329ae8` | UNREVIEWED |
| `libfontconfig-1.dll` | NO | 350185 | `d6c70b7e205f24ff96b29a80849b59c84f8701b311434f64f2bf93bd36bdb163` | UNREVIEWED |
| `libfreetype-6.dll` | NO | 776690 | `5297f754dfca04abdb0d21705d5afc3c4122aa25c7afbdd87900313cb780c868` | UNREVIEWED |
| `libfribidi-0.dll` | NO | 144627 | `449085380e54089871ccc9f32b715f65a2e53eb9f5125793fed900461d29a420` | UNREVIEWED |
| `libgcc_s_seh-1.dll` | YES | 150707 | `b22b954397a52703579d92db64b57812af70f2afcafe2e742a009c1640b9ec1a` | UNREVIEWED |
| `libgif-7.dll` | YES | 41367 | `a1a179853ae6fad8eb045b8e9f12c55e6fc99929446b02fa345512354cf5e582` | UNREVIEWED |
| `libgio-2.0-0.dll` | NO | 1814226 | `07bda2471ca0490598d1e6fbdfc8958cd91926d68effc90d9eda0ea2f9279c7c` | UNREVIEWED |
| `libglib-2.0-0.dll` | NO | 1456928 | `12b2f4d609a5d3cf714a8c502074409d32f21082a682373f4e1bca36821f08bc` | UNREVIEWED |
| `libgmodule-2.0-0.dll` | NO | 26783 | `8cf9bb9be24c88a66ad478214a55357a9d44702636789dfa436ca509457d781c` | UNREVIEWED |
| `libgobject-2.0-0.dll` | NO | 359001 | `4fcf4fb2d5ef234e717b389f3b637f05ee6ff111b698ef59664394f10622aef6` | UNREVIEWED |
| `libgraphite2.dll` | NO | 152538 | `f927efecd958f0e54a24ae297f86cd6aed05e7fc20e5da0882050ab9ec464395` | UNREVIEWED |
| `libharfbuzz-0.dll` | NO | 1329363 | `c1fa05f089041e348c9fcf18ae115942c989ac5259069d0ae83aa63e9a7c66ea` | UNREVIEWED |
| `libiconv-2.dll` | YES | 1118202 | `967189adfbc889fde89aafc867f7a1f02731f8592cf6fd5a4ace1929213e2e13` | UNREVIEWED |
| `libicudt75.dll` | NO | 30743046 | `6a60829e1c13fd73043af6f475f85971a42eb5553e88c92f26b8802bde92c2f4` | UNREVIEWED |
| `libicuin75.dll` | NO | 3381215 | `9fad2353a168134305e67a62fedff21cd43293055b7f4b9c39ff30cc78f30be0` | UNREVIEWED |
| `libicuuc75.dll` | NO | 1932464 | `60e6f91536efd0a8e408b060edb76a2148a7a397399b76ee39e4ed61de0774b0` | UNREVIEWED |
| `libintl-8.dll` | NO | 188681 | `42a63cb4c3c28a683d9f6c3510de5ec17849eb18c097fa02cd78aeb800bff202` | UNREVIEWED |
| `libjbig-0.dll` | YES | 62083 | `275a827ba6839d47cfdbe059db1b1b403a92ff26e2b76a29362915487868bef2` | UNREVIEWED |
| `libjpeg-8.dll` | YES | 912746 | `e44514c4e7753635e79bdb668f1e10315c1a93287f8d044ae74594ed57de619d` | UNREVIEWED |
| `libleptonica-6.dll` | YES | 2721006 | `26861baff4d25446a03ed1f6064d2eae7435eddf51bb36af4e634c0eb4eb3af0` | UNREVIEWED |
| `libLerc.dll` | YES | 761261 | `4ab688c7c5c4afdfee0f519f942759137067a7876a1c1bb825ee575b0a5760b1` | UNREVIEWED |
| `liblz4.dll` | YES | 147105 | `1b64d6429ae4aacf9eb35b8158d98556859a5c5e8ee817e155da343e194dd8e3` | UNREVIEWED |
| `liblzma-5.dll` | YES | 184667 | `3a599d97ea4dd002166ee7ffdc527c129d2cf8b05e9152ca7cae006c4187977e` | UNREVIEWED |
| `libopenjp2-7.dll` | YES | 462194 | `5ff762ff6d672a06ee502819daf1494aa3aa3acf408805f0429303e026705011` | UNREVIEWED |
| `libpango-1.0-0.dll` | NO | 401912 | `834caf40403a447634b0e80ff4ea290f1ee361364cf24ffc8bb55b048f9254a9` | UNREVIEWED |
| `libpangocairo-1.0-0.dll` | NO | 74540 | `ee706c4561af24de6cbe7ad10b46852fc8b0f328534ea2beb5f1147696e87b55` | UNREVIEWED |
| `libpangoft2-1.0-0.dll` | NO | 102261 | `7c80bc1949255a8bc62fe05ab003b777aa844ac8f6c5da1ebc4c49ee35f3b9b8` | UNREVIEWED |
| `libpangowin32-1.0-0.dll` | NO | 114044 | `d940a831f2a9be33dd36ff3d8b97488a068b1699fb6b2e874633d4d5564f301c` | UNREVIEWED |
| `libpcre2-8-0.dll` | NO | 399737 | `5ffa31709c275120a745f9f714974db1e86c8fd54f8ca6889617530dffbff95b` | UNREVIEWED |
| `libpixman-1-0.dll` | NO | 714812 | `8f8b8f09ec8fe9945ba76fadf8a04183056afd2c35aa7be1808e1a1301d87757` | UNREVIEWED |
| `libpng16-16.dll` | YES | 247380 | `f1b0a9d6cf3b8809c22fa0e27b5b82d9357e5bca43ab0ce8762f55fb303ed23f` | UNREVIEWED |
| `libsharpyuv-0.dll` | YES | 57852 | `60bcc87ac64a834e967e9ec9461be37395dbc090c579f3e23497ef69a5d8323e` | UNREVIEWED |
| `libstdc++-6.dll` | YES | 2409896 | `0d9c01ab1771df439c75f144b2d855b4ac400243f006d14d9a043493de628f0d` | UNREVIEWED |
| `libtesseract-5.dll` | YES | 101471336 | `279488c41bd341edaa03dcd097faa33604ba330e6254ac97bcaf2229d8d7ac1a` | UNREVIEWED |
| `libthai-0.dll` | NO | 69739 | `420e08cedaf9daaabcf8f6c92349d9dadb7b5ad9e4e79d1126bb2a55a7cdbe09` | UNREVIEWED |
| `libtiff-6.dll` | YES | 576887 | `e000306d3de299b05ebef443e48c1486d9fa291b0244f6ca6225843b75fdf581` | UNREVIEWED |
| `libwebp-7.dll` | YES | 743580 | `687b8859e1f739dad5800fdc293a6ee1fd0dd4b33380ab283681a5a12eb3d4fa` | UNREVIEWED |
| `libwebpmux-3.dll` | YES | 78730 | `ad10bacecd37b44c7c2c813da098a8bf6a42a3bf0152f7f33726d216815558f9` | UNREVIEWED |
| `libwinpthread-1.dll` | YES | 60798 | `bbcbe5eb8e806f3537585d4c9832b4ee15f86617328a566695d7d3728aa370ee` | UNREVIEWED |
| `libzstd.dll` | YES | 1179848 | `f216ea8efb5b3bb9ad3d938fb9ac07e3b8f78c2c92faab03d2c468d0b6798a5a` | UNREVIEWED |
| `tesseract.exe` | YES | 1585024 | `babb405f4366b480d02cd8ff2bac8d497170f6c1711ce6f3d5d8bf0fb7fa6ed9` | UNREVIEWED |
| `zlib1.dll` | YES | 120814 | `cb7ab3788d10940df874acd97b1821bbb5ee4a91f3eec11982bb5bf7a3c96443` | UNREVIEWED |

## External imports still unreviewed

The parser reported **17 external DLL basenames** not
adjacent to the staged executable, of which
**8** appear in the
statically reachable graph. Their presence and loader resolution on clean
Windows 10/11 require separate verification. None are automatically trusted.

All external basenames:
- `advapi32.dll`
- `bcrypt.dll`
- `crypt32.dll`
- `dnsapi.dll`
- `dwrite.dll`
- `gdi32.dll`
- `iphlpapi.dll`
- `kernel32.dll`
- `msimg32.dll`
- `msvcrt.dll`
- `ole32.dll`
- `rpcrt4.dll`
- `shell32.dll`
- `shlwapi.dll`
- `user32.dll`
- `usp10.dll`
- `ws2_32.dll`

External basenames from statically reachable binaries:
- `advapi32.dll`
- `bcrypt.dll`
- `crypt32.dll`
- `gdi32.dll`
- `kernel32.dll`
- `msvcrt.dll`
- `user32.dll`
- `ws2_32.dll`

## Required independent review before redistribution

1. Reproduce pinned source and staged hashes at the final release-candidate SHA.
2. Identify exact origin, version, license text, copyright, and NOTICE
   obligations for the executable, each adjacent DLL, and English tessdata.
3. Check dynamic loads before considering any DLL removal or packaging trim.
4. Verify external imports, DLL loader search, and writable-directory risks
   on clean Windows 10 and Windows 11 installations.
5. Verify required license/NOTICE files are included in the actual NSIS
   payload and remain available after installation.
6. Complete offline physical OCR, install/uninstall and settings tests
   against the exact installer SHA-256.
7. Obtain explicit redistribution and final integration approval.

**Gate status:** native licensing, DLL loader, physical acceptance,
installer publication and owner authorization remain unchecked.
