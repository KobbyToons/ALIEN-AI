param([string]$jsonArgs = "{}")

$ErrorActionPreference = "Stop"
$parsed = ConvertFrom-Json $jsonArgs -ErrorAction SilentlyContinue
if (-not $parsed) { $parsed = [pscustomobject]@{} }

$paintPath = "C:\Program Files\CELSYS\CLIP STUDIO 1.5\CLIP STUDIO PAINT\CLIPStudioPaint.exe"
$hubPath = "C:\Program Files\CELSYS\CLIP STUDIO 1.5\CLIP STUDIO\CLIPStudio.exe"

$targetExe = if ($parsed.mode -eq "hub") { $hubPath } else { $paintPath }

if (-not (Test-Path $targetExe)) {
    # Try searching Program Files
    $cand = Get-ChildItem "C:\Program Files\CELSYS" -Filter "CLIPStudioPaint.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($cand) { $targetExe = $cand.FullName }
}

if (-not (Test-Path $targetExe)) {
    Write-Output "Error: Clip Studio Paint executable not found at '$targetExe'."
    exit 1
}

Start-Process -FilePath $targetExe
$result = @{
    success = $true
    launched = $targetExe
    message = "Clip Studio Paint launched successfully."
}
Write-Output ($result | ConvertTo-Json -Compress)

