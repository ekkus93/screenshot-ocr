# Hosted Windows OCR smoke: execute the checksum-verified staged runtime only.
# Use synthetic text, a bounded process, and no user-provided screenshots.
param(
    [string]$RuntimeRoot,
    [switch]$SkipSharedEngine
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$root = if ($RuntimeRoot) {
    [IO.Path]::GetFullPath($RuntimeRoot)
} else {
    [IO.Path]::GetFullPath((Join-Path $PSScriptRoot "../src-tauri/windows-ocr-runtime"))
}
$binary = Join-Path $root "tesseract.exe"
$tessdata = Join-Path $root "tessdata"
if (-not (Test-Path -LiteralPath $binary -PathType Leaf) -or
    -not (Test-Path -LiteralPath (Join-Path $tessdata "eng.traineddata") -PathType Leaf)) {
    throw "Staged OCR executable or English language data missing."
}

$tempRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [IO.Path]::GetTempPath() }
$workspace = Join-Path $tempRoot ("screenshot-ocr-synthetic-smoke-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $workspace -Force | Out-Null
try {
    # Windows-hosted CI has System.Drawing; no external screenshot application
    # and no private content is used to construct this deterministic fixture.
    Add-Type -AssemblyName System.Drawing
    $bitmap = [Drawing.Bitmap]::new(640, 140)
    $graphics = $null
    $font = $null
    try {
        $graphics = [Drawing.Graphics]::FromImage($bitmap)
        $font = [Drawing.Font]::new("Arial", 52, [Drawing.FontStyle]::Bold, [Drawing.GraphicsUnit]::Pixel)
        $graphics.Clear([Drawing.Color]::White)
        $graphics.DrawString("SAMPLE 123", $font, [Drawing.Brushes]::Black, 18.0, 25.0)
        $imagePath = Join-Path $workspace "synthetic.png"
        $bitmap.Save($imagePath, [Drawing.Imaging.ImageFormat]::Png)
    } finally {
        if ($null -ne $font) { $font.Dispose() }
        if ($null -ne $graphics) { $graphics.Dispose() }
        $bitmap.Dispose()
    }

    # Direct process creation (no shell), with bounded runtime and reaping.
    $start = [Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $binary
    $start.WorkingDirectory = $root
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    foreach ($argument in @($imagePath, "stdout", "--tessdata-dir", $tessdata, "-l", "eng", "--psm", "7")) {
        [void]$start.ArgumentList.Add($argument)
    }
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $start
    try {
        try {
            if (-not $process.Start()) { throw "Could not start OCR." }
        } catch {
            throw "Staged Windows OCR runtime could not start."
        }
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        if (-not $process.WaitForExit(20000)) {
            $process.Kill($true)
            $process.WaitForExit()
            throw "Staged Windows OCR runtime exceeded the 20-second limit."
        }
        $recognized = $stdoutTask.GetAwaiter().GetResult()
        [void]$stderrTask.GetAwaiter().GetResult()
        if ($process.ExitCode -ne 0) { throw "Staged Windows OCR runtime returned a nonzero exit status." }
        if ($recognized -notmatch '(?i)SAMPLE\s+123') {
            throw "Staged Windows OCR runtime did not recognize the synthetic fixture."
        }
    } finally {
        $process.Dispose()
    }
    if ($SkipSharedEngine) {
        # Run the real shared Rust OCR probe/recognition path against the
        # extracted NSIS payload, not merely a direct Tesseract invocation.
        $env:SCREENSHOT_OCR_WINDOWS_OCR_RUNTIME_ROOT = $root
        $env:SCREENSHOT_OCR_WINDOWS_OCR_SMOKE_IMAGE = $imagePath
        try {
            $manifest = Join-Path $PSScriptRoot "../src-tauri/Cargo.toml"
            & cargo test --manifest-path $manifest --locked windows_staged_shared_engine_integration -- --ignored
            if ($LASTEXITCODE -ne 0) { throw "Extracted NSIS shared OCR engine smoke failed." }
        } finally {
            Remove-Item Env:SCREENSHOT_OCR_WINDOWS_OCR_RUNTIME_ROOT -ErrorAction SilentlyContinue
            Remove-Item Env:SCREENSHOT_OCR_WINDOWS_OCR_SMOKE_IMAGE -ErrorAction SilentlyContinue
        }
    }
    # The extracted NSIS payload is tested as shipped, without running setup.
    # The shared-engine fixture is separately exercised against the staged
    # runtime earlier in the same CI job.
    if (-not $SkipSharedEngine) {
    # Compile a deterministic native child that waits until cancelled.
    $fakeBinary = Join-Path $workspace "synthetic-slow-tesseract.exe"
    $fakeSource = Join-Path $PSScriptRoot "fixtures/windows-slow-tesseract.rs"
    & rustc --edition 2021 $fakeSource -o $fakeBinary
    if ($LASTEXITCODE -ne 0) { throw "Synthetic Windows cancellation fixture failed to compile." }
    # Exercise the application's shared Rust OCR engine, not only the native
    # executable. The fixture is synthetic and is deleted by this script.
    $env:SCREENSHOT_OCR_WINDOWS_FAKE_TESSERACT = $fakeBinary
    $env:SCREENSHOT_OCR_WINDOWS_FAKE_MARKER_DIR = $workspace
    $env:SCREENSHOT_OCR_WINDOWS_FAKE_EXPECTED_CWD = $workspace
    $env:SCREENSHOT_OCR_WINDOWS_OCR_SMOKE_IMAGE = $imagePath
    try {
        $manifest = Join-Path $PSScriptRoot "../src-tauri/Cargo.toml"
        & cargo test --manifest-path $manifest --locked windows_staged_shared_engine -- --ignored
        if ($LASTEXITCODE -ne 0) { throw "Shared Windows OCR engine integration smoke failed." }
        # A cancelled task must also terminate its spawned native process.
        # The synthetic fixture records each PID before reporting readiness.
        foreach ($mode in @("probe", "recognize")) {
            $pidFile = Join-Path $workspace "$mode.pid"
            if (-not (Test-Path -LiteralPath $pidFile -PathType Leaf)) {
                throw "Synthetic Windows $mode child did not report its process ID."
            }
            $pidText = (Get-Content -LiteralPath $pidFile -Raw).Trim()
            $childProcessId = [int]::Parse($pidText, [Globalization.CultureInfo]::InvariantCulture)
            if ($childProcessId -le 0) { throw "Invalid synthetic child process ID." }
            if ($null -ne (Get-Process -Id $childProcessId -ErrorAction SilentlyContinue)) {
                throw "Shared Windows OCR cancellation left a synthetic $mode child running."
            }
        }
    } finally {
        Remove-Item Env:SCREENSHOT_OCR_WINDOWS_OCR_SMOKE_IMAGE -ErrorAction SilentlyContinue
        Remove-Item Env:SCREENSHOT_OCR_WINDOWS_FAKE_TESSERACT -ErrorAction SilentlyContinue
        Remove-Item Env:SCREENSHOT_OCR_WINDOWS_FAKE_MARKER_DIR -ErrorAction SilentlyContinue
        Remove-Item Env:SCREENSHOT_OCR_WINDOWS_FAKE_EXPECTED_CWD -ErrorAction SilentlyContinue
    }
    }
    Write-Output "Synthetic English OCR succeeded using checksum-verified runtime files."
    Write-Output "Hosted process smoke is not physical Windows selector/installer acceptance."
} finally {
    if (Test-Path -LiteralPath $workspace) {
        Remove-Item -LiteralPath $workspace -Recurse -Force
    }
}
