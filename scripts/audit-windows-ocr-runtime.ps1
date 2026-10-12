# Read-only audit of the pinned Windows Tesseract distribution.
# No installer is executed, no system-wide installation is performed, and
# this script does not publish a redistributable application bundle.
param([switch]$StageRuntime)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$uri = "https://digi.bib.uni-mannheim.de/tesseract/tesseract-ocr-w64-setup-5.4.0.20240606.exe"
$expectedSha256 = "c885fff6998e0608ba4bb8ab51436e1c6775c2bafc2559a19b423e18678b60c9"
# Local acceptance uses the same staging path as CI, without requiring RUNNER_TEMP.
$tempRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [IO.Path]::GetTempPath() }
$workspace = Join-Path $tempRoot ("screenshot-ocr-tesseract-audit-" + [guid]::NewGuid().ToString("N"))
$setup = Join-Path $workspace "pinned-tesseract-setup.exe"
$extracted = Join-Path $workspace "extracted"
New-Item -ItemType Directory -Force -Path $extracted | Out-Null
try {

Write-Output "Downloading checksum-pinned Tesseract build input."
for ($attempt = 1; $attempt -le 3; $attempt++) {
    try {
        Invoke-WebRequest -Uri $uri -OutFile $setup -TimeoutSec 240
        break
    } catch {
        if (Test-Path -LiteralPath $setup) { Remove-Item -LiteralPath $setup -Force }
        if ($attempt -eq 3) { throw "Pinned OCR source unavailable after three attempts." }
        Start-Sleep -Seconds (5 * $attempt)
    }
}
$actual = (Get-FileHash -LiteralPath $setup -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actual -cne $expectedSha256) {
    throw "Windows Tesseract installer SHA-256 mismatch; refusing extraction."
}
Write-Output "Pinned installer SHA-256 verified."

$sevenZip = Join-Path $env:ProgramFiles "7-Zip\7z.exe"
if (-not (Test-Path -LiteralPath $sevenZip -PathType Leaf)) {
    $command = Get-Command "7z.exe" -ErrorAction SilentlyContinue
    if ($null -eq $command) { throw "7-Zip executable unavailable for read-only extraction." }
    $sevenZip = $command.Source
}

# Reject redirected or empty build inputs before hashing or copying them.
# Walk from the trusted extraction root outward so a linked parent is
# rejected before its child is accessed. Never emit private source paths.
function Assert-ArchiveRegularFile([string]$Path) {
    $root = [IO.Path]::GetFullPath($extracted)
    $full = [IO.Path]::GetFullPath($Path)
    $prefix = $root + [IO.Path]::DirectorySeparatorChar
    if (-not $full.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "OCR archive input escapes the extraction root."
    }
    $current = $root
    $entry = $null
    foreach ($part in $full.Substring($prefix.Length).Split([IO.Path]::DirectorySeparatorChar)) {
        if ([string]::IsNullOrEmpty($part) -or $part -eq "." -or $part -eq "..") {
            throw "OCR archive input has an invalid component."
        }
        $current = Join-Path $current $part
        $entry = Get-Item -LiteralPath $current -Force -ErrorAction Stop
        if (($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "OCR archive input contains a redirected path."
        }
    }
    if (-not ($entry -is [IO.FileInfo]) -or $entry.Length -le 0) {
        throw "OCR archive input must be a nonempty regular file."
    }
}

# 7-Zip extracts the verified NSIS archive as data, without launching setup.
& $sevenZip x "-o$extracted" -y $setup | Out-Null
if ($LASTEXITCODE -ne 0) { throw "Pinned installer could not be extracted by 7-Zip." }

$candidates = @(Get-ChildItem -LiteralPath $extracted -Recurse -File -Force -Filter "tesseract.exe")
if ($candidates.Count -eq 0) { throw "No tesseract.exe found in verified archive." }

$matches = @(
    foreach ($candidate in $candidates) {
        $eng = Join-Path $candidate.DirectoryName "tessdata\eng.traineddata"
        if (Test-Path -LiteralPath $eng -PathType Leaf) {
            [pscustomobject]@{ Binary = $candidate; English = $eng }
        }
    }
)
if ($matches.Count -ne 1) {
    throw "Expected exactly one executable with adjacent English tessdata; found $($matches.Count)."
}
$selected = $matches[0]
$binaryDirectory = $selected.Binary.DirectoryName
Assert-ArchiveRegularFile $selected.Binary.FullName
Assert-ArchiveRegularFile $selected.English.FullName
$dlls = @(Get-ChildItem -LiteralPath $binaryDirectory -File -Force -Filter "*.dll" | Sort-Object Name)
foreach ($dll in $dlls) { Assert-ArchiveRegularFile $dll.FullName }
if ($dlls.Count -eq 0) { throw "No adjacent runtime DLLs found; packaging inventory incomplete." }

$entries = @(
    [pscustomobject]@{
        name = "tesseract.exe"
        sha256 = (Get-FileHash -LiteralPath $selected.Binary.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    [pscustomobject]@{
        name = "tessdata/eng.traineddata"
        sha256 = (Get-FileHash -LiteralPath $selected.English -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    foreach ($dll in $dlls) {
        [pscustomobject]@{
            name = $dll.Name
            sha256 = (Get-FileHash -LiteralPath $dll.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        }
    }
)
$manifest = [ordered]@{
    schema_version = 1
    provenance = "University of Mannheim Tesseract 5.4.0.20240606"
    input_sha256 = $actual
    binary_count = 1
    english_data_count = 1
    adjacent_dll_count = $dlls.Count
    files = $entries
    qualification = "input inventory only; transitive DLL/license and physical offline acceptance pending"
}
$manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath "windows-ocr-runtime-audit.json" -Encoding utf8
Write-Output "Verified input inventory: $($dlls.Count) adjacent DLL files; no installer was executed."

if ($StageRuntime) {
    # Stage only for CI packaging qualification. Do not publish this bundle
    # until the transitive DLL/license review and physical acceptance pass.
    $stage = Join-Path $PSScriptRoot "..\src-tauri\windows-ocr-runtime"
    $tauriRoot = Join-Path $PSScriptRoot "..\\src-tauri"
    $tauriItem = Get-Item -LiteralPath $tauriRoot -Force -ErrorAction Stop
    if (($tauriItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Refusing to stage OCR files through a redirected source directory."
    }
    if (Test-Path -LiteralPath $stage) {
        $stageItem = Get-Item -LiteralPath $stage -Force -ErrorAction Stop
        if (($stageItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Refusing to replace a redirected OCR staging directory."
        }
        Remove-Item -LiteralPath $stage -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path (Join-Path $stage "tessdata") | Out-Null
    Copy-Item -LiteralPath $selected.Binary.FullName -Destination (Join-Path $stage "tesseract.exe")
    Copy-Item -LiteralPath $selected.English -Destination (Join-Path $stage "tessdata\eng.traineddata")
    foreach ($dll in $dlls) {
        Copy-Item -LiteralPath $dll.FullName -Destination (Join-Path $stage $dll.Name)
    }
    Write-Output "Staged checksum-verified runtime for non-published NSIS smoke build."
}
} finally {
    # The verified source installer and extracted native binaries are
    # build-time inputs, not persistent developer/runner temp artifacts.
    # Preserve only the explicit staged resources and content-free manifest.
    if (Test-Path -LiteralPath $workspace) {
        Remove-Item -LiteralPath $workspace -Recurse -Force
    }
}
