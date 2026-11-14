<#
.SYNOPSIS
  Shared PowerShell bridge for ALIEN Brave Browser automation tools.
#>

function Ensure-AlienBridgeServer {
    $port = 4114
    $healthUrl = "http://127.0.0.1:$port/health"

    # Check if already running
    try {
        $resp = Invoke-RestMethod -Uri $healthUrl -Method Get -TimeoutSec 2 -ErrorAction Stop
        if ($resp.ok) {
            return $true
        }
    } catch {
        # Not responding yet
    }

    # Start bridge server in background
    $serverScript = "d:\KOBbyTOOns\ARt\AL\My Music\ANTIGRAVITY\buddy\extensions\alien-browser-extension\bridge-server.js"
    if (-not (Test-Path $serverScript)) {
        Write-Error "Bridge server script not found: $serverScript"
        return $false
    }

    try {
        Start-Process -FilePath "node" -ArgumentList "`"$serverScript`"" -WindowStyle Hidden
    } catch {
        Write-Error "Failed to launch Node.js bridge server: $_"
        return $false
    }

    # Wait up to 3 seconds for server to respond
    for ($i = 0; $i -lt 15; $i++) {
        Start-Sleep -Milliseconds 200
        try {
            $resp = Invoke-RestMethod -Uri $healthUrl -Method Get -TimeoutSec 1 -ErrorAction Stop
            if ($resp.ok) {
                return $true
            }
        } catch { }
    }

    Write-Error "Bridge server failed to initialize on port $port."
    return $false
}

function Invoke-AlienBrowserAction {
    param(
        [Parameter(Mandatory = $true)][string]$Action,
        [Parameter(Mandatory = $false)][string]$JsonArgs = "{}"
    )

    $ErrorActionPreference = "Stop"

    # 1. Ensure local bridge server is active
    $isReady = Ensure-AlienBridgeServer
    if (-not $isReady) {
        Write-Output "Error: ALIEN Browser Bridge could not be reached. Ensure Node.js is installed."
        return
    }

    # 2. Parse arguments
    $params = [pscustomobject]@{}
    if (-not [string]::IsNullOrWhiteSpace($JsonArgs)) {
        try {
            $params = ConvertFrom-Json $JsonArgs -ErrorAction Stop
        } catch {
            $params = [pscustomobject]@{}
        }
    }

    $payloadObj = @{
        action = $Action
        params = $params
        timeoutMs = 25000
    }
    $body = $payloadObj | ConvertTo-Json -Depth 10

    # 3. Dispatch command to bridge server
    try {
        $resp = Invoke-RestMethod -Uri "http://127.0.0.1:4114/command" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 28
        
        if ($resp.success) {
            if ($resp.data) {
                return ($resp.data | ConvertTo-Json -Depth 10)
            } else {
                return "Action '$Action' succeeded."
            }
        } else {
            $err = if ($resp.error) { $resp.error } else { "Unknown browser extension failure." }
            return "Browser Action Error: $err"
        }
    } catch {
        $msg = $_.Exception.Message
        if ($msg -like "*timed out*") {
            return "Error: Browser action '$Action' timed out. Please ensure Brave Browser is open with the ALIEN Extension installed and enabled."
        }
        return "Error contacting browser bridge: $msg"
    }
}
