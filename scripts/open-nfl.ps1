param([string]$DataDir = '', [switch]$NoLaunch)
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($DataDir)) { $DataDir = Join-Path $taskRoot 'dist\data' }
$taskExe = Join-Path $taskRoot 'dist\pattern-atlas-0.9.0.exe'
$taskImport = Join-Path $taskRoot 'datasets\nfl-teams-2026-10-13.json'
$taskRunning = @(Get-CimInstance Win32_Process | Where-Object { $_.Name -like 'pattern-atlas*.exe' })
if ($taskRunning.Count -gt 0) { throw 'Save and close running Pattern Atlas windows before installing/opening the NFL session. This protects unsaved edits.' }
New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
$taskBackup = Join-Path $DataDir ('workspace-before-nfl-' + [guid]::NewGuid().ToString('N') + '.json')
if (Test-Path -LiteralPath (Join-Path $DataDir 'atlas.sqlite')) {
    & $taskExe --data-dir $DataDir --export $taskBackup
    if ($LASTEXITCODE -ne 0) { throw 'Workspace backup failed; import canceled' }
}
& $taskExe --data-dir $DataDir --ensure-data $taskImport --session-name 'NFL' --observer-place 'Chicago, Illinois'
if ($LASTEXITCODE -ne 0) { throw 'NFL import failed' }
if (-not $NoLaunch) { & $taskExe --data-dir $DataDir --open-session 'NFL' }
