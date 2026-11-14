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

$data = if ($ArgJson) { $ArgJson | ConvertFrom-Json } else { $null }
$categoryFilter = if ($data -and $data.category) { $data.category } else { $null }

$searchPath = if ($categoryFilter) { Join-Path $vault $categoryFilter } else { $vault }
if (-not (Test-Path $searchPath)) {
    Write-Output "[]"
    exit 0
}

$files = Get-ChildItem -Path $searchPath -Filter "*.md" -Recurse | Where-Object { -not $_.PSIsContainer }

$notes = @()
foreach ($f in $files) {
    $rel = $f.FullName.Substring($vault.Length).TrimStart('\', '/')
    $cat = (Split-Path $rel -Parent)
    if ([string]::IsNullOrWhiteSpace($cat)) { $cat = "Root" }

    $notes += @{
        title = $f.BaseName
        category = $cat
        path = $f.FullName
        size = $f.Length
        modified = $f.LastWriteTime.ToString("yyyy-MM-dd HH:mm:ss")
    }
}

$result = @{
    vault = $vault
    count = $notes.Count
    notes = $notes
}

Write-Output ($result | ConvertTo-Json -Depth 3 -Compress)

