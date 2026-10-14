param([switch]$DebugBuild)
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskCache = Join-Path (Split-Path -Parent $taskRoot) '.build-cache'
$taskCargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
$taskOldCargoHome = $env:CARGO_HOME
$taskOldTemp = $env:TEMP
$taskOldTmp = $env:TMP
$taskOldPath = $env:PATH
$taskOldRustup = $env:RUSTUP_HOME
try {
    New-Item -ItemType Directory -Force (Join-Path $taskCache 'cargo'),(Join-Path $taskCache 'tmp') | Out-Null
    $env:CARGO_HOME = Join-Path $taskCache 'cargo'
    $env:TEMP = Join-Path $taskCache 'tmp'
    $env:TMP = $env:TEMP
    $env:PATH = "$(Join-Path $env:USERPROFILE '.cargo\bin');$env:PATH"
    if (Test-Path -LiteralPath (Join-Path $taskCache 'rustup\settings.toml')) { $env:RUSTUP_HOME = Join-Path $taskCache 'rustup' }
    Push-Location $taskRoot
    & $taskCargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'Formatting check failed' }
    & $taskCargo test --workspace --locked
    if ($LASTEXITCODE -ne 0) { throw 'Tests failed' }
    & $taskCargo clippy --workspace --all-targets --locked -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Static checks failed' }
    if ($DebugBuild) {
        & $taskCargo build -p pattern-atlas --locked
        $taskProfile = 'debug'
    } else {
        & $taskCargo build -p pattern-atlas --release --locked
        $taskProfile = 'release'
    }
    if ($LASTEXITCODE -ne 0) { throw 'Build failed' }
    New-Item -ItemType Directory -Force (Join-Path $taskRoot 'dist') | Out-Null
    $taskVersionMatch = Select-String -LiteralPath (Join-Path $taskRoot 'Cargo.toml') -Pattern '^version = "([^"]+)"'
    $taskVersion = $taskVersionMatch.Matches[0].Groups[1].Value
    $taskVersionedExe = Join-Path $taskRoot "dist\pattern-atlas-$taskVersion.exe"
    Copy-Item -LiteralPath (Join-Path $taskRoot "target\$taskProfile\pattern-atlas.exe") -Destination $taskVersionedExe -Force
    try {
        Copy-Item -LiteralPath $taskVersionedExe -Destination (Join-Path $taskRoot 'dist\pattern-atlas.exe') -Force -ErrorAction Stop
        Write-Output "Built $(Join-Path $taskRoot 'dist\pattern-atlas.exe')"
    } catch {
        Write-Output "The existing executable could not be replaced (it may be running). Built $taskVersionedExe; save/close the previous app before launching this version."
    }
} finally {
    Pop-Location
    $env:CARGO_HOME = $taskOldCargoHome
    $env:TEMP = $taskOldTemp
    $env:TMP = $taskOldTmp
    $env:PATH = $taskOldPath
    $env:RUSTUP_HOME = $taskOldRustup
}
