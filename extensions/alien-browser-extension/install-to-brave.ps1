<#
.SYNOPSIS
  Automated Installer for ALIEN Assistant Brave Browser Extension
#>

$extDir = "d:\KOBbyTOOns\ARt\AL\My Music\ANTIGRAVITY\buddy\extensions\alien-browser-extension"
$crxPath = "d:\KOBbyTOOns\ARt\AL\My Music\ANTIGRAVITY\buddy\extensions\alien-browser-extension.crx"
$extId = "afblndojgjjnemhbljhfbnobgnigbpoi"
$braveExe = "C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe"

Write-Host "==========================================================" -ForegroundColor Red
Write-Host "      ALIEN ASSISTANT — BRAVE BROWSER INSTALLER           " -ForegroundColor White
Write-Host "==========================================================" -ForegroundColor Red

# 1. Start Local Bridge Server if not running
$healthUrl = "http://127.0.0.1:4114/health"
$bridgeRunning = $false
try {
    $resp = Invoke-RestMethod -Uri $healthUrl -Method Get -TimeoutSec 1 -ErrorAction SilentlyContinue
    if ($resp.ok) { $bridgeRunning = $true }
} catch { }

if (-not $bridgeRunning) {
    Write-Host "[1/4] Starting ALIEN Local Bridge Server on port 4114..." -ForegroundColor Yellow
    $serverScript = Join-Path $extDir "bridge-server.js"
    Start-Process -FilePath "node" -ArgumentList "`"$serverScript`"" -WindowStyle Hidden
    Start-Sleep -Seconds 1
} else {
    Write-Host "[1/4] ALIEN Local Bridge Server is already online." -ForegroundColor Green
}

# 2. Register in Windows Registry
Write-Host "[2/4] Registering external extension in Windows Registry..." -ForegroundColor Yellow
$braveKey = "HKCU:\Software\BraveSoftware\Brave-Browser\Extensions\$extId"
New-Item -Path $braveKey -Force | Out-Null
Set-ItemProperty -Path $braveKey -Name "path" -Value $crxPath
Set-ItemProperty -Path $braveKey -Name "version" -Value "1.0.0"
Write-Host "      Registry key: $braveKey" -ForegroundColor Gray

# 3. Copy unpacked extension path to clipboard
Write-Host "[3/4] Copying extension path to Windows clipboard..." -ForegroundColor Yellow
Set-Clipboard -Value $extDir
Write-Host "      Path copied: $extDir" -ForegroundColor Green

# 4. Open Brave Extensions page
Write-Host "[4/4] Opening brave://extensions in Brave Browser..." -ForegroundColor Yellow
if (Test-Path $braveExe) {
    Start-Process -FilePath $braveExe -ArgumentList "brave://extensions"
} else {
    Start-Process "brave://extensions"
}

Write-Host ""
Write-Host "----------------------------------------------------------" -ForegroundColor Cyan
Write-Host "  FINAL 5-SECOND STEP IN BRAVE:" -ForegroundColor White
Write-Host "  1. Look at the top right of the Brave Extensions page." -ForegroundColor Gray
Write-Host "  2. Toggle 'Developer mode' ON (if not already on)." -ForegroundColor Gray
Write-Host "  3. Click 'Load unpacked' button on top-left." -ForegroundColor Gray
Write-Host "  4. Press Ctrl + V (path is already on your clipboard!)" -ForegroundColor Yellow
Write-Host "  5. Click 'Select Folder'." -ForegroundColor Yellow
Write-Host "----------------------------------------------------------" -ForegroundColor Cyan
Write-Host "ALIEN Browser Extension is now installed and active!" -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Red
