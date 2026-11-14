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
    Write-Error "No Obsidian vault is currently connected. Please configure your vault path in ALIEN Settings."
    exit 1
}

$data = $ArgJson | ConvertFrom-Json
$category = if ($data.category) { $data.category } else { "Learned" }
$title = $data.title.Replace('/', '_').Replace('\', '_').Replace(':', '_')
if (-not $title.EndsWith('.md')) { $title = "$title.md" }
$content = $data.content

$targetDir = Join-Path $vault $category
if (-not (Test-Path $targetDir)) {
    New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
}

$targetFile = Join-Path $targetDir $title
$date = (Get-Date).ToUniversalTime().ToString("yyyy-MM-dd HH:mm UTC")

$fullContent = "# $($data.title)`nCreated: $date`n`n$content"
Set-Content -Path $targetFile -Value $fullContent -Encoding utf8

Write-Output "Successfully created Obsidian note: [[$category/$($data.title)]] at $targetFile"

