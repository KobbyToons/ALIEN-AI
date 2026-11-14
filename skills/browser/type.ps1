param([string]$jsonArgs = "{}")
. "$PSScriptRoot\browser_bridge.ps1"
Invoke-AlienBrowserAction -Action "type" -JsonArgs $jsonArgs
