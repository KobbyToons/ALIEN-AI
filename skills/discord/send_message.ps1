param([string]$jsonArgs = "{}")

$ErrorActionPreference = "Stop"
$parsed = ConvertFrom-Json $jsonArgs -ErrorAction SilentlyContinue
if (-not $parsed) { $parsed = [pscustomobject]@{} }

$content = $parsed.content
if ([string]::IsNullOrWhiteSpace($content)) {
    Write-Output "Error: Message content cannot be empty."
    exit 1
}

$recipient = $parsed.recipient
$webhook = $parsed.webhook_url
$botToken = $parsed.bot_token
$channelId = $parsed.channel_id

# Check config file in ~/.alien/discord.json if not provided
$configPath = Join-Path $HOME ".alien\discord.json"
if (Test-Path $configPath) {
    try {
        $cfg = Get-Content $configPath -Raw | ConvertFrom-Json
        if (-not $webhook -and $cfg.webhook_url) { $webhook = $cfg.webhook_url }
        if (-not $botToken -and $cfg.bot_token) { $botToken = $cfg.bot_token }
        if (-not $channelId -and $cfg.default_channel_id) { $channelId = $cfg.default_channel_id }
    } catch {}
}

if (-not $webhook -and $env:DISCORD_WEBHOOK_URL) { $webhook = $env:DISCORD_WEBHOOK_URL }
if (-not $botToken -and $env:DISCORD_BOT_TOKEN) { $botToken = $env:DISCORD_BOT_TOKEN }

# 1. Webhook delivery
if ($webhook) {
    $payload = @{
        content = $content
        username = "ALIEN"
    } | ConvertTo-Json -Depth 3

    try {
        $resp = Invoke-RestMethod -Uri $webhook -Method Post -Body $payload -ContentType "application/json"
        $result = @{
            success = $true
            method = "webhook"
            message = "Message successfully sent to Discord via webhook."
            recipient = $recipient
        }
        Write-Output ($result | ConvertTo-Json -Compress)
        exit 0
    } catch {
        Write-Output "Error sending via webhook: $($_.Exception.Message)"
        exit 1
    }
}

# 2. Bot API delivery
if ($botToken -and $channelId) {
    $headers = @{
        "Authorization" = "Bot $botToken"
        "User-Agent" = "AlienApp (https://alien.app, 1.0.0)"
    }
    $payload = @{
        content = $content
    } | ConvertTo-Json

    try {
        $resp = Invoke-RestMethod -Uri "https://discord.com/api/v10/channels/$channelId/messages" -Headers $headers -Method Post -Body $payload -ContentType "application/json"
        $result = @{
            success = $true
            method = "bot_api"
            message = "Message sent to Discord channel $channelId."
            message_id = $resp.id
        }
        Write-Output ($result | ConvertTo-Json -Compress)
        exit 0
    } catch {
        Write-Output "Error sending via bot API: $($_.Exception.Message)"
        exit 1
    }
}

# 3. Fallback: Copy to clipboard and launch Discord app
$discordPath = Join-Path $env:LOCALAPPDATA "Discord\Update.exe"
$discordArgs = "--processStart Discord.exe"
if (-not (Test-Path $discordPath)) {
    $found = Get-Command "Discord.exe" -ErrorAction SilentlyContinue
    if ($found) { $discordPath = $found.Source; $discordArgs = "" }
}

Set-Clipboard -Value $content
if (Test-Path $discordPath) {
    if ($discordArgs) {
        Start-Process -FilePath $discordPath -ArgumentList $discordArgs
    } else {
        Start-Process -FilePath $discordPath
    }
    $result = @{
        success = $true
        method = "app_launch"
        message = "Discord opened. Message copied to clipboard for recipient '$recipient'. (Configure webhook or bot_token in ~/.alien/discord.json for full background automation)."
    }
    Write-Output ($result | ConvertTo-Json -Compress)
} else {
    Write-Output "Error: No Discord webhook or bot token configured, and Discord app executable not found. Place webhook in ~/.alien/discord.json."
    exit 1
}

