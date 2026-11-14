param([string]$ArgJson)

function Get-VaultPath {
    $candidates = @(
        "$env:APPDATA\com.alien.app\vault_path.txt",
        "$env:APPDATA\com.alien.desktop\vault_path.txt",
        "$env:APPDATA\com.buddy.app\vault_path.txt",
        "$env:USERPROFILE\Documents\Future Heroes\ALIEN's Mind",
        "$env:USERPROFILE\Documents\Obsidian Vault"
    )
    foreach ($c in $candidates) {
        if (Test-Path $c) {
            if ((Get-Item $c) -is [System.IO.DirectoryInfo]) {
                return $c
            }
            $p = (Get-Content $c -Raw).Trim().Trim('"').Trim("'")
            if (Test-Path $p) { return $p }
        }
    }
    return $null
}

$vault = Get-VaultPath
if (-not $vault) {
    Write-Error "No Obsidian vault is currently connected. Please configure your vault path in ALIEN Settings."
    exit 1
}

$data = $ArgJson | ConvertFrom-Json
$title = $data.title.Replace('/', '_').Replace('\', '_').Replace(':', '_')
if (-not $title.EndsWith('.md')) { $title = "$title.md" }
$instructions = $data.instructions

$targetDir = Join-Path $vault "Instructions"
if (-not (Test-Path $targetDir)) {
    New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
}

$targetFile = Join-Path $targetDir $title
$date = (Get-Date).ToUniversalTime().ToString("yyyy-MM-dd HH:mm UTC")

$fullContent = @"
---
type: instruction
title: "$($data.title)"
taught_on: $date
status: active
---

# Procedural Instruction: $($data.title)
*Taught directly by the user to ALIEN on $date*

## Core Rules & Execution Procedure
$instructions

## Contextual Triggers & Guidance
Whenever the user asks about or tasks ALIEN with `$($data.title)` or related actions, ALIEN MUST consult and adhere to this exact procedure.
"@

[System.IO.File]::WriteAllText($targetFile, $fullContent, [System.Text.Encoding]::UTF8)

Write-Output "Successfully learned and recorded procedural instruction [[Instructions/$($data.title)]] into Obsidian."

