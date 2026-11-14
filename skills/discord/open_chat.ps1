param([string]$jsonArgs = "{}")

$ErrorActionPreference = "Stop"
$parsed = ConvertFrom-Json $jsonArgs -ErrorAction SilentlyContinue
if (-not $parsed) { $parsed = [pscustomobject]@{} }

$target = $parsed.target
$guildId = $parsed.guild_id

if ($target) {
    if ($guildId) {
        $uri = "discord://discord.com/channels/$guildId/$target"
    } else {
        $uri = "discord://discord.com/channels/@me/$target"
    }
    Start-Process $uri
    $result = @{
        success = $true
        uri = $uri
        message = "Opened Discord destination: $uri"
    }
    Write-Output ($result | ConvertTo-Json -Compress)
} else {
    $discordPath = Join-Path $env:LOCALAPPDATA "Discord\Update.exe"
    $discordArgs = "--processStart Discord.exe"
    if (Test-Path $discordPath) {
        Start-Process -FilePath $discordPath -ArgumentList $discordArgs
    } else {
        Start-Process "discord://"
    }
    $result = @{
        success = $true
        message = "Launched Discord app."
    }
    Write-Output ($result | ConvertTo-Json -Compress)
}

