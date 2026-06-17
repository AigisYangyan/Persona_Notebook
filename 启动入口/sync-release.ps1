param()

$ErrorActionPreference = "Stop"

$entryDir = $PSScriptRoot
$rootDir = Split-Path -Parent $entryDir
$publishFolderName = ([string][char]0x53D1) + [char]0x5E03
$publishDir = Join-Path $entryDir $publishFolderName

if (-not (Test-Path -LiteralPath $publishDir)) {
    New-Item -ItemType Directory -Path $publishDir | Out-Null
}

$bundleRoots = @(
    (Join-Path $rootDir "src-tauri\target\release\bundle\msi"),
    (Join-Path $rootDir "src-tauri\target\release\bundle\nsis")
)

foreach ($bundleRoot in $bundleRoots) {
    if (-not (Test-Path -LiteralPath $bundleRoot)) {
        continue
    }

    Get-ChildItem -LiteralPath $bundleRoot -File |
        Where-Object { $_.Extension -in ".msi", ".exe" } |
        ForEach-Object {
            Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $publishDir $_.Name) -Force
        }
}

Write-Host "[PGRN] Release artifacts synced to $publishDir"
