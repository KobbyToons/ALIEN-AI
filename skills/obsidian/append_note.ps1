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
$category = if ($data.category) { $data.category } else { "Daily" }
$title = $data.title.Replace('/', '_').Replace('\', '_').Replace(':', '_')
if (-not $title.EndsWith('.md')) { $title = "$title.md" }
$content = $data.content

$targetDir = Join-Path $vault $category
if (-not (Test-Path $targetDir)) {
    New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
}

$targetFile = Join-Path $targetDir $title
$date = (Get-Date).ToUniversalTime().ToString("yyyy-MM-dd HH:mm UTC")

if (Test-Path $targetFile) {
    $append = "`n`n---`n### Update: $date`n$content"
    Add-Content -Path $targetFile -Value $append -Encoding utf8
} else {
    $full = "# $($data.title)`nCreated: $date`n`n$content"
    Set-Content -Path $targetFile -Value $full -Encoding utf8
}

Write-Output "Appended update to note: [[$category/$($data.title)]]"

