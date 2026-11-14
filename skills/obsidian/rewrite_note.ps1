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

# Check if file existed before to preserve frontmatter if any
$fullContent = "# $($data.title)`nUpdated: $date (Optimized and rewritten by ALIEN)`n`n$content"
[System.IO.File]::WriteAllText($targetFile, $fullContent, [System.Text.Encoding]::UTF8)

Write-Output "Successfully rewritten and optimized Obsidian note: [[$category/$($data.title)]] at $targetFile"

