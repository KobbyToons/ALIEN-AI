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
    Write-Error "No Obsidian vault connected."
    exit 1
}

$data = $ArgJson | ConvertFrom-Json
$category = $data.category
$title = $data.title.Replace('/', '_').Replace('\', '_').Replace(':', '_')
if (-not $title.EndsWith('.md')) { $title = "$title.md" }

$found = $null

if ($category) {
    $f = Join-Path (Join-Path $vault $category) $title
    if (Test-Path $f) { $found = $f }
} else {
    $candidates = Get-ChildItem -Path $vault -Recurse -Filter $title -File -ErrorAction SilentlyContinue
    if ($candidates.Count -gt 0) { $found = $candidates[0].FullName }
}

if (-not $found) {
    Write-Error "Note not found in Obsidian vault: $($data.title)"
    exit 1
}

Get-Content -Path $found -Raw

