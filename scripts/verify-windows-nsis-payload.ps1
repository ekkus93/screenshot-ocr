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
try {
& $sevenZip x "-o$extracted" -y $installers[0].FullName | Out-Null
if ($LASTEXITCODE -ne 0) { throw "NSIS extraction failed." }
# Tauri offlineInstaller embeds Microsoft's standalone WebView2 runtime setup
# as an NSIS File entry. Validate the actual archive, not only Tauri config.
# This is a static payload check; it does not run setup or prove offline install.
$webview2 = @(Get-ChildItem -LiteralPath $extracted -Recurse -File -Force -Filter "MicrosoftEdgeWebView2RuntimeInstaller.exe")
if ($webview2.Count -ne 1) {
    throw "Expected exactly one embedded WebView2 offline runtime installer."
}
if ($webview2[0].Length -lt 30000000) {
    throw "Embedded WebView2 payload is unexpectedly small."
}
$webview2Hash = (Get-FileHash -LiteralPath $webview2[0].FullName -Algorithm SHA256).Hash.ToLowerInvariant()
Write-Output "Embedded WebView2 offline payload present; SHA-256: $webview2Hash"
$signature = Get-AuthenticodeSignature -LiteralPath $webview2[0].FullName
if ($null -eq $signature -or
    $signature.Status -ne [System.Management.Automation.SignatureStatus]::Valid -or
    $null -eq $signature.SignerCertificate) {
    throw "Embedded WebView2 offline installer does not have a valid trusted signature."
}
# A valid signature alone is insufficient: a third party could sign a
# substituted payload. Require Microsoft's signing organization as well.
$publisher = $signature.SignerCertificate.Subject
if ($publisher -notmatch '(^|,\s*)O\s*=\s*"?Microsoft Corporation"?(,|$)') {
    throw "Embedded WebView2 offline installer is not signed by Microsoft Corporation."
}
Write-Output "Embedded WebView2 Authenticode signature and Microsoft publisher verified."
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
# Do not accept extra empty directories: a recursive file-only inventory
# misses them, but the application's runtime resolver refuses such layouts.
# An NSIS payload must have exactly one OCR subdirectory, tessdata.
$payloadDirectories = @(Get-ChildItem -LiteralPath $root -Recurse -Directory -Force)
if ($payloadDirectories.Count -ne 1 -or
    [IO.Path]::GetRelativePath($root, $payloadDirectories[0].FullName) -ine "tessdata") {
    throw "Unexpected directories in packaged OCR runtime."
}
foreach ($directory in $payloadDirectories) {
    if (($directory.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Redirected directory in packaged OCR runtime."
    }
}
# Fail closed if the OCR resource tree contains files absent from the pinned
# manifest, including nested DLLs and executable/data payloads.
$payloadFiles = @(Get-ChildItem -LiteralPath $root -Recurse -File -Force)
if ($payloadFiles.Count -ne @($manifest.files).Count) {
    throw "Unexpected files in packaged OCR runtime."
}
foreach ($file in $payloadFiles) {
    if (($file.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Redirected file in packaged OCR runtime."
    }
    $relative = [IO.Path]::GetRelativePath($root, $file.FullName).Replace('\', '/')
    if (-not $seen.ContainsKey($relative.ToLowerInvariant())) {
        throw "Unreviewed file in packaged OCR runtime."
    }
}
# Exercise the actual extracted NSIS OCR payload with synthetic English text.
# This checks runtime DLL loading and OCR execution from the packaged layout,
# not installer execution, physical desktop capture, or redistribution rights.
& (Join-Path $PSScriptRoot "smoke-windows-ocr-runtime.ps1") -RuntimeRoot $root -ExtractedNsisPayload
# Normalise the candidate's basename so a physical-test record can bind its
# installer/checksum to the source version without depending on Tauri defaults.
# This stays local to the smoke job: no Windows binaries are uploaded.
$config = Get-Content -LiteralPath "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$package = Get-Content -LiteralPath "package.json" -Raw | ConvertFrom-Json
if ($config.productName -cne "Screenshot OCR" -or
    $config.identifier -cne "io.github.ekkus93.screenshot-ocr" -or
    $config.version -cne $package.version -or
    $config.version -notmatch '^\d+\.\d+\.\d+$') {
    throw "Windows installer metadata is inconsistent."
}
$canonicalName = "screenshot-ocr-v$($config.version)-windows-x64-setup.exe"
$canonicalPath = Join-Path $installers[0].DirectoryName $canonicalName
if ($installers[0].Name -cne $canonicalName) {
    if (Test-Path -LiteralPath $canonicalPath) {
        throw "Conflicting canonical Windows installer filename."
    }
    Move-Item -LiteralPath $installers[0].FullName -Destination $canonicalPath
}
$checksum = (Get-FileHash -LiteralPath $canonicalPath -Algorithm SHA256).Hash.ToLowerInvariant()
$checksumLine = "$checksum  $canonicalName"
Set-Content -LiteralPath "windows-installer.sha256" -Value $checksumLine -Encoding ascii
Write-Output "NSIS runtime inventory verified; candidate filename: $canonicalName"
Write-Output "NSIS installer SHA-256: $checksum"
Write-Output "Not publishing: native-license review and physical acceptance pending."
} finally {
    # Even failed signature, digest, or native-runtime checks must not leave
    # unpacked executable payloads in the runner/developer temp directory.
    if (Test-Path -LiteralPath $extracted) {
        Remove-Item -LiteralPath $extracted -Recurse -Force
    }
}
