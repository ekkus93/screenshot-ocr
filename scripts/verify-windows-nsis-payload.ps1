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
$dlls = @(Get-ChildItem -LiteralPath $root -File -Filter "*.dll")
if ($dlls.Count -ne $manifest.adjacent_dll_count -or
    @($manifest.files).Count -ne ($dlls.Count + 2)) { throw "Runtime DLL inventory count mismatch." }
foreach ($dll in $dlls) {
    if (-not $seen.ContainsKey($dll.Name.ToLowerInvariant())) { throw "Unreviewed native DLL." }
}
$checksum = (Get-FileHash -LiteralPath $installers[0].FullName -Algorithm SHA256).Hash.ToLowerInvariant()
Write-Output "NSIS runtime inventory verified; installer SHA-256: $checksum"
Write-Output "Not publishing: native-license review and physical acceptance pending."
