param([string]$ArgJson)

function Get-VaultPath {
    $candidates = @(
        "$env:APPDATA\com.alien.desktop\vault_path.txt",
        "$env:APPDATA\com.buddy.app\vault_path.txt"
    )
    foreach ($c in $candidates) {
        if (Test-Path $c) {
            $p = (Get-Content $c -Raw).Trim().Trim('"').Trim("'")
            if (Test-Path $p) { return $p }
        }
    }
    $default = "$env:USERPROFILE\Documents\Future Heroes\ALIEN's Mind"
    if (Test-Path $default) { return $default }
    return $null
}

$vault = Get-VaultPath
if (-not $vault) {
    [PSCustomObject]@{
        connected = $false
        path = $null
        noteCount = 0
    } | ConvertTo-Json
    exit 0
}

$notes = (Get-ChildItem -Path $vault -Recurse -Filter "*.md" -File -ErrorAction SilentlyContinue | Where-Object { $_.FullName -notmatch '\\\.obsidian\\' }).Count

[PSCustomObject]@{
    connected = $true
    path = $vault
    noteCount = $notes
} | ConvertTo-Json

