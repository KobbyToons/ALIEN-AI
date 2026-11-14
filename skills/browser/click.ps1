param([string]$jsonArgs = "{}")
. "$PSScriptRoot\browser_bridge.ps1"
Invoke-AlienBrowserAction -Action "click" -JsonArgs $jsonArgs
