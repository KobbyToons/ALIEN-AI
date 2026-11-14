param([string]$jsonArgs = "{}")

$ErrorActionPreference = "Stop"
$parsed = ConvertFrom-Json $jsonArgs -ErrorAction SilentlyContinue
if (-not $parsed) { $parsed = [pscustomobject]@{} }

$projectName = if ($parsed.project_name) { $parsed.project_name.Trim() } else { "Artwork_" + (Get-Date -Format "yyyyMMdd_HHmmss") }
# Sanitize project name
$cleanName = ($projectName -replace '[\\/*?:"<>|]', '_')

$preset = if ($parsed.preset) { $parsed.preset.ToLower() } else { "illustration" }

$width = 1920
$height = 1080
$dpi = 300

switch ($preset) {
    "webtoon" {
        $width = 800
        $height = 1280
    }
    "4k_canvas" {
        $width = 3840
        $height = 2160
    }
    "comic_a4" {
        $width = 2480
        $height = 3508
    }
    "square" {
        $width = 2048
        $height = 2048
    }
    default { # illustration
        $width = 1920
        $height = 1080
    }
}

$saveDir = $parsed.save_dir
if ([string]::IsNullOrWhiteSpace($saveDir)) {
    $saveDir = Join-Path $HOME "Documents\ClipStudioProjects\$cleanName"
}
if (-not (Test-Path $saveDir)) {
    New-Item -ItemType Directory -Path $saveDir -Force | Out-Null
}

$targetFile = Join-Path $saveDir "$cleanName.png"

# Generate blank white canvas at target resolution and DPI
Add-Type -AssemblyName System.Drawing
$bmp = New-Object System.Drawing.Bitmap $width, $height
$bmp.SetResolution($dpi, $dpi)
$graphics = [System.Drawing.Graphics]::FromImage($bmp)
$graphics.Clear([System.Drawing.Color]::White)
$graphics.Dispose()
$bmp.Save($targetFile, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()

$paintPath = "C:\Program Files\CELSYS\CLIP STUDIO 1.5\CLIP STUDIO PAINT\CLIPStudioPaint.exe"
if (-not (Test-Path $paintPath)) {
    $cand = Get-ChildItem "C:\Program Files\CELSYS" -Filter "CLIPStudioPaint.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($cand) { $paintPath = $cand.FullName }
}

if (Test-Path $paintPath) {
    Start-Process -FilePath $paintPath -ArgumentList "`"$targetFile`""
} else {
    Start-Process $targetFile
}

$result = @{
    success = $true
    project_name = $cleanName
    preset = $preset
    dimensions = "${width}x${height} @ ${dpi} DPI"
    canvas_file = $targetFile
    message = "Created $preset canvas ($width x $height) and opened in Clip Studio Paint."
}
Write-Output ($result | ConvertTo-Json -Compress)

