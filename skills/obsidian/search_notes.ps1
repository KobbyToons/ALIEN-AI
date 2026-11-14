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
$query = $data.query

$matches = Get-ChildItem -Path $vault -Recurse -Filter "*.md" -File -ErrorAction SilentlyContinue | Where-Object {
    $_.FullName -notmatch '\\\.obsidian\\' -and ($_.Name -like "*$query*" -or (Select-String -Path $_.FullName -Pattern $query -SimpleMatch -Quiet))
} | Select-Object -First 15 | ForEach-Object {
    [PSCustomObject]@{
        name = $_.BaseName
        path = $_.FullName.Substring($vault.Length).TrimStart('\', '/')
        updated = $_.LastWriteTime.ToString("yyyy-MM-dd HH:mm")
    }
}

$matches | ConvertTo-Json

