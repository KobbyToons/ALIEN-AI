param([string]$jsonArgs = "{}")

$ErrorActionPreference = "Stop"
$parsed = ConvertFrom-Json $jsonArgs -ErrorAction SilentlyContinue
if (-not $parsed) { $parsed = [pscustomobject]@{} }

$content = $parsed.content
if ([string]::IsNullOrWhiteSpace($content)) {
    Write-Output "Error: Message content cannot be empty."
    exit 1
}

$userId = $parsed.user_id
$userName = $parsed.user_name
$botToken = $parsed.bot_token

# Check config file in ~/.alien/discord.json if not provided
$configPath = Join-Path $HOME ".alien\discord.json"
if (Test-Path $configPath) {
    try {
        $cfg = Get-Content $configPath -Raw | ConvertFrom-Json
        if (-not $botToken -and $cfg.bot_token) { $botToken = $cfg.bot_token }
    } catch {}
}
if (-not $botToken -and $env:DISCORD_BOT_TOKEN) { $botToken = $env:DISCORD_BOT_TOKEN }

if ($botToken -and $userId) {
    $headers = @{
        "Authorization" = "Bot $botToken"
        "User-Agent" = "AlienApp (https://alien.app, 1.0.0)"
    }
    # 1. Create DM channel
    try {
        $dmPayload = @{ recipient_id = $userId } | ConvertTo-Json
        $dmChannel = Invoke-RestMethod -Uri "https://discord.com/api/v10/users/@me/channels" -Headers $headers -Method Post -Body $dmPayload -ContentType "application/json"
        $dmChannelId = $dmChannel.id

        # 2. Send message
        $msgPayload = @{ content = $content } | ConvertTo-Json
        $resp = Invoke-RestMethod -Uri "https://discord.com/api/v10/channels/$dmChannelId/messages" -Headers $headers -Method Post -Body $msgPayload -ContentType "application/json"

        $result = @{
            success = $true
            method = "dm_api"
            message = "DM sent to user $userId."
            dm_channel_id = $dmChannelId
            message_id = $resp.id
        }
        Write-Output ($result | ConvertTo-Json -Compress)
        exit 0
    } catch {
        Write-Output "Error sending DM via bot: $($_.Exception.Message)"
        exit 1
    }
}

# Fallback: Copy to clipboard and launch deep link to user DM
Set-Clipboard -Value $content
if ($userId) {
    Start-Process "discord://discord.com/users/$userId"
    $result = @{
        success = $true
        method = "deep_link"
        message = "Opened Discord DM for user ID $userId. Message is copied to clipboard ready to paste."
    }
    Write-Output ($result | ConvertTo-Json -Compress)
} else {
    $discordPath = Join-Path $env:LOCALAPPDATA "Discord\Update.exe"
    $discordArgs = "--processStart Discord.exe"
    if (Test-Path $discordPath) {
        Start-Process -FilePath $discordPath -ArgumentList $discordArgs
    }
    $result = @{
        success = $true
        method = "app_launch"
        message = "Opened Discord. Message copied to clipboard for $userName."
    }
    Write-Output ($result | ConvertTo-Json -Compress)
}

