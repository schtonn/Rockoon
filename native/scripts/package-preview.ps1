param([switch]$SkipBuild)

$ErrorActionPreference = 'Stop'
$repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
$previewRoot = Join-Path $repoRoot 'artifacts\preview'
$binary = Join-Path $repoRoot 'native\target\release\rockoon-native.exe'
$utf8 = New-Object System.Text.UTF8Encoding $false

function Stop-PreviewAtPath([string]$ExecutablePath) {
    $running = Get-CimInstance Win32_Process | Where-Object {
        $_.ExecutablePath -and [string]::Equals($_.ExecutablePath, $ExecutablePath, [StringComparison]::OrdinalIgnoreCase)
    }
    foreach ($entry in $running) {
        $previewProcess = Get-Process -Id $entry.ProcessId -ErrorAction SilentlyContinue
        if (-not $previewProcess -or $previewProcess.HasExited) { continue }
        if ($previewProcess.CloseMainWindow() -and $previewProcess.WaitForExit(3000)) { continue }
        if (-not $previewProcess.HasExited) {
            Stop-Process -Id $previewProcess.Id -Force
            $previewProcess.WaitForExit(3000) | Out-Null
        }
    }
}

if (-not $SkipBuild) {
    Stop-PreviewAtPath $binary
    $cargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
    if ($cargoCommand) {
        $cargoExecutable = $cargoCommand.Source
    } else {
        $localTools = Join-Path (Split-Path $repoRoot -Parent) '.rockoon-tools'
        $cargoExecutable = Join-Path $localTools 'cargo\bin\cargo.exe'
        if (-not (Test-Path -LiteralPath $cargoExecutable -PathType Leaf)) {
            throw 'Install Rust or add cargo to PATH before packaging.'
        }
        $env:CARGO_HOME = Join-Path $localTools 'cargo'
        $env:RUSTUP_HOME = Join-Path $localTools 'rustup'
        $env:PATH = (Split-Path $cargoExecutable -Parent) + ';' + $env:PATH
    }
    & $cargoExecutable build --release --locked --manifest-path (Join-Path $repoRoot 'native\Cargo.toml')
    if ($LASTEXITCODE -ne 0) { throw 'Native build failed.' }
}
if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) {
    throw 'No release binary exists. Run without -SkipBuild first.'
}

New-Item -ItemType Directory -Path $previewRoot -Force | Out-Null
$destination = Join-Path $previewRoot 'Rockoon.exe'
$pending = Join-Path $previewRoot '.Rockoon.exe.tmp'
Stop-PreviewAtPath $destination
try {
    Copy-Item -LiteralPath $binary -Destination $pending -Force
    Move-Item -LiteralPath $pending -Destination $destination -Force
} catch {
    if (Test-Path -LiteralPath $pending) { Remove-Item -LiteralPath $pending -Force }
    throw 'Cannot replace artifacts\preview\Rockoon.exe after closing the preview process.'
}

$licenses = Join-Path $previewRoot 'licenses'
New-Item -ItemType Directory -Path $licenses -Force | Out-Null
foreach ($license in @(
    @('native\resources\fonts\LICENSE.txt', 'SourceHanSans.txt'),
    @('native\vendor\iced_tiny_skia\LICENSE', 'Iced.txt'),
    @('native\docs\LibCmo-LICENSE.txt', 'LibCmo.txt')
)) {
    Copy-Item -LiteralPath (Join-Path $repoRoot $license[0]) -Destination (Join-Path $licenses $license[1]) -Force
}
$manifest = Get-Content -LiteralPath (Join-Path $repoRoot 'native\Cargo.toml') -Raw
$version = [regex]::Match($manifest, '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
$commit = (& git -C $repoRoot rev-parse HEAD).Trim()
$info = [ordered]@{
    version = $version
    packaged_at_commit = $commit
    binary_modified_utc = [IO.File]::GetLastWriteTimeUtc($binary).ToString('o')
    sha256 = (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash.ToLowerInvariant()
}
[IO.File]::WriteAllText((Join-Path $previewRoot 'build-info.json'), ($info | ConvertTo-Json) + "`n", $utf8)
Write-Output $destination
