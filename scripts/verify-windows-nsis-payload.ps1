# Check all native OCR payload files against the pinned build-input inventory.
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$manifest = Get-Content -LiteralPath "windows-ocr-runtime-audit.json" -Raw | ConvertFrom-Json
$installers = @(Get-ChildItem -Path "src-tauri/target/release/bundle/nsis" -Filter "*-setup.exe" -File)
if ($installers.Count -ne 1) { throw "Expected exactly one NSIS smoke installer." }
$sevenZip = Join-Path $env:ProgramFiles "7-Zip/7z.exe"
if (-not (Test-Path -LiteralPath $sevenZip -PathType Leaf)) { throw "7-Zip unavailable." }
$tempRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [IO.Path]::GetTempPath() }
$extracted = Join-Path $tempRoot ("screenshot-ocr-nsis-smoke-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $extracted | Out-Null
& $sevenZip x "-o$extracted" -y $installers[0].FullName | Out-Null
if ($LASTEXITCODE -ne 0) { throw "NSIS extraction failed." }
$binary = @(Get-ChildItem -LiteralPath $extracted -Recurse -File -Filter "tesseract.exe")
if ($binary.Count -ne 1) { throw "Expected one packaged Tesseract binary." }
$root = $binary[0].DirectoryName
# The resolver uses current_exe()/resources/ocr, not a recursive search.
# A payload can contain valid OCR files but still fail on a clean install if
# Tauri places them beside the wrong executable or in a different directory.
$app = @(Get-ChildItem -LiteralPath $extracted -Recurse -File -Filter "screenshot-ocr.exe")
if ($app.Count -ne 1) { throw "Expected exactly one packaged application executable." }
$expectedRoot = Join-Path $app[0].DirectoryName "resources/ocr"
if ([IO.Path]::GetFullPath($root) -ine [IO.Path]::GetFullPath($expectedRoot)) {
    throw "Packaged OCR directory does not match the application runtime resolver."
}
$seen = @{}
foreach ($entry in @($manifest.files)) {
    $name = [string]$entry.name
    if ($name -notmatch '^(tesseract\.exe|tessdata/eng\.traineddata|[A-Za-z0-9_.+-]+\.dll)$' -or
        $seen.ContainsKey($name.ToLowerInvariant())) { throw "Invalid runtime inventory entry." }
    $seen[$name.ToLowerInvariant()] = $true
    $path = Join-Path $root $name.Replace('/', [IO.Path]::DirectorySeparatorChar)
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Missing runtime component." }
    $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($hash -cne $entry.sha256) { throw "Runtime component digest mismatch." }
}
if (-not $seen.ContainsKey("tesseract.exe") -or -not $seen.ContainsKey("tessdata/eng.traineddata")) {
    throw "Runtime inventory missing required OCR components."
}
$dlls = @(Get-ChildItem -LiteralPath $root -File -Force -Filter "*.dll")
if ($dlls.Count -ne $manifest.adjacent_dll_count -or
    @($manifest.files).Count -ne ($dlls.Count + 2)) { throw "Runtime DLL inventory count mismatch." }
foreach ($dll in $dlls) {
    if (-not $seen.ContainsKey($dll.Name.ToLowerInvariant())) { throw "Unreviewed native DLL." }
}
# Fail closed if the OCR resource tree contains files absent from the pinned
# manifest, including nested DLLs and executable/data payloads.
$payloadFiles = @(Get-ChildItem -LiteralPath $root -Recurse -File -Force)
if ($payloadFiles.Count -ne @($manifest.files).Count) {
    throw "Unexpected files in packaged OCR runtime."
}
foreach ($file in $payloadFiles) {
    $relative = [IO.Path]::GetRelativePath($root, $file.FullName).Replace('\', '/')
    if (-not $seen.ContainsKey($relative.ToLowerInvariant())) {
        throw "Unreviewed file in packaged OCR runtime."
    }
}
$checksum = (Get-FileHash -LiteralPath $installers[0].FullName -Algorithm SHA256).Hash.ToLowerInvariant()
Write-Output "NSIS runtime inventory verified; installer SHA-256: $checksum"
Write-Output "Not publishing: native-license review and physical acceptance pending."
