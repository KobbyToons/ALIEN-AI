param([string]$jsonArgs = "{}")
. "$PSScriptRoot\browser_bridge.ps1"
Invoke-AlienBrowserAction -Action "get_content" -JsonArgs $jsonArgs
