param([string]$jsonArgs = "{}")
. "$PSScriptRoot\browser_bridge.ps1"
Invoke-AlienBrowserAction -Action "list_tabs" -JsonArgs $jsonArgs
