param([string]$jsonArgs = "{}")

$ErrorActionPreference = "Stop"
$parsed = ConvertFrom-Json $jsonArgs -ErrorAction SilentlyContinue
if (-not $parsed) { $parsed = [pscustomobject]@{} }

$filePath = $parsed.file_path
if ([string]::IsNullOrWhiteSpace($filePath)) {
    Write-Output "Error: No file_path provided."
    exit 1
}

$resolved = Resolve-Path $filePath -ErrorAction SilentlyContinue
if (-not $resolved -or -not (Test-Path $resolved.Path)) {
    Write-Output "Error: File '$filePath' does not exist."
    exit 1
}

$targetPath = $resolved.Path

$paintPath = "C:\Program Files\CELSYS\CLIP STUDIO 1.5\CLIP STUDIO PAINT\CLIPStudioPaint.exe"
if (-not (Test-Path $paintPath)) {
    $cand = Get-ChildItem "C:\Program Files\CELSYS" -Filter "CLIPStudioPaint.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($cand) { $paintPath = $cand.FullName }
}

if (-not (Test-Path $paintPath)) {
    # If executable not found directly, open with default shell association
    Start-Process $targetPath
    $result = @{
        success = $true
        opened_file = $targetPath
        method = "system_association"
        message = "Opened file with default program."
    }
    Write-Output ($result | ConvertTo-Json -Compress)
    exit 0
}

Start-Process -FilePath $paintPath -ArgumentList "`"$targetPath`""
$result = @{
    success = $true
    opened_file = $targetPath
    method = "clip_studio_paint"
    message = "Successfully opened '$targetPath' in Clip Studio Paint."
}
Write-Output ($result | ConvertTo-Json -Compress)

