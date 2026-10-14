param(
    [Parameter(Mandatory=$true)][string]$InputPath,
    [Parameter(Mandatory=$true)][string]$OutputPath,
    [string]$DatasetName = 'Word dictionary',
    [Parameter(Mandatory=$true)][string]$BatchId,
    [string]$Source = '',
    [string[]]$Tags = @('dictionary')
)
$ErrorActionPreference = 'Stop'
# One word/phrase per UTF-8 line; blank lines ignored. Preserve spelling and order.
$taskInput = (Resolve-Path -LiteralPath $InputPath).Path
$taskOutput = [System.IO.Path]::GetFullPath($OutputPath)
if (Test-Path -LiteralPath $taskOutput) { throw "Output already exists: $taskOutput" }
if ([string]::IsNullOrWhiteSpace($BatchId)) { throw 'BatchId cannot be blank' }
if ([string]::IsNullOrWhiteSpace($DatasetName)) { throw 'DatasetName cannot be blank' }
$taskSeen = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
$taskEntities = [System.Collections.Generic.List[object]]::new()
$taskNumber = 0
foreach ($taskLine in [System.IO.File]::ReadLines($taskInput, [System.Text.Encoding]::UTF8)) {
    $taskWord = $taskLine.Trim().TrimStart([char]0xFEFF)
    if ($taskWord.Length -eq 0 -or -not $taskSeen.Add($taskWord)) { continue }
    $taskNumber++
    $taskEntities.Add([ordered]@{
        key = "word-$taskNumber"; name = $taskWord; kind = 'word'; aliases = @(); tags = @($Tags)
        context = ''; source = $Source; attributes = @{}
    })
}
if ($taskEntities.Count -eq 0) { throw 'No words found in input' }
$taskImport = [ordered]@{
    format = 'pattern-atlas-data'; version = 1; batch_id = $BatchId
    dataset = [ordered]@{
        name = $DatasetName; context = 'UTF-8 word list; one word or phrase per line; case-insensitive duplicates removed.'
        entities = @($taskEntities.ToArray()); places = @(); observations = @()
    }
}
$taskJson = ConvertTo-Json -InputObject $taskImport -Depth 20
$taskBytes = [System.Text.UTF8Encoding]::new($false).GetBytes($taskJson)
# CreateNew prevents accidental overwriting even if another process creates the file.
$taskFile = [System.IO.File]::Open($taskOutput, [System.IO.FileMode]::CreateNew)
try { $taskFile.Write($taskBytes, 0, $taskBytes.Length); $taskFile.Flush() } finally { $taskFile.Dispose() }
Write-Output "Wrote $($taskEntities.Count) words to $taskOutput. Preview in Import data or validate with --validate-data."
