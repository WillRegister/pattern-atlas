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
    & $taskCargo run -p pattern-atlas --locked -- --data-dir (Join-Path $taskRoot 'data')
    if ($LASTEXITCODE -ne 0) { throw 'App failed' }
} finally {
    Pop-Location
    $env:CARGO_HOME = $taskOldCargoHome
    $env:TEMP = $taskOldTemp
    $env:TMP = $taskOldTmp
    $env:PATH = $taskOldPath
    $env:RUSTUP_HOME = $taskOldRustup
}
