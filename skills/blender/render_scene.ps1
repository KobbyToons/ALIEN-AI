param([string]$jsonArgs = "{}")

$ErrorActionPreference = "Stop"
$parsed = ConvertFrom-Json $jsonArgs -ErrorAction SilentlyContinue
if (-not $parsed) { $parsed = [pscustomobject]@{} }

# 1. Locate Blender executable
$blenderPath = "C:\Program Files\Blender Foundation\Blender 4.4\blender.exe"
if (-not (Test-Path $blenderPath)) {
    $found = Get-Command "blender.exe" -ErrorAction SilentlyContinue
    if ($found) { $blenderPath = $found.Source }
}
if (-not (Test-Path $blenderPath)) {
    $candidates = Get-ChildItem "C:\Program Files\Blender Foundation" -Filter "blender.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($candidates) { $blenderPath = $candidates.FullName }
}
if (-not (Test-Path $blenderPath)) {
    Write-Output "Error: Blender executable not found."
    exit 1
}

$outImg = $parsed.output_image_path
if ([string]::IsNullOrWhiteSpace($outImg)) {
    $rendersDir = Join-Path $HOME "Documents\BlenderScenes\Renders"
    if (-not (Test-Path $rendersDir)) { New-Item -ItemType Directory -Path $rendersDir -Force | Out-Null }
    $timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
    $outImg = Join-Path $rendersDir "Render_$timestamp.png"
}

# Ensure destination directory exists
$parentDir = Split-Path $outImg -Parent
if ($parentDir -and -not (Test-Path $parentDir)) {
    New-Item -ItemType Directory -Path $parentDir -Force | Out-Null
}

$samples = if ($parsed.samples) { [int]$parsed.samples } else { 64 }
$blendFile = $parsed.blend_file

# Set up render python snippet to configure samples and output path properly
$tempPy = [System.IO.Path]::GetTempFileName() + ".py"
$pyContent = @"
import bpy

scene = bpy.context.scene
if hasattr(scene, 'cycles'):
    scene.cycles.samples = $samples
scene.render.image_settings.file_format = 'PNG'
scene.render.filepath = r"$outImg"
bpy.ops.render.render(write_still=True)
"@

[System.IO.File]::WriteAllText($tempPy, $pyContent, [System.Text.Encoding]::UTF8)

try {
    if ($blendFile -and (Test-Path $blendFile)) {
        & "$blenderPath" -b "$blendFile" -P "$tempPy" | Out-Null
    } else {
        & "$blenderPath" -b -P "$tempPy" | Out-Null
    }

    if (Test-Path $outImg) {
        $result = @{
            success = $true
            message = "Blender render completed successfully."
            rendered_image = $outImg
            samples = $samples
        }
        Write-Output ($result | ConvertTo-Json -Compress)
    } else {
        # Check if Blender appended frame number e.g. 0001.png
        $candidateWithFrame = [System.IO.Path]::ChangeExtension($outImg, $null) + "0001.png"
        if (Test-Path $candidateWithFrame) {
            Move-Item -Path $candidateWithFrame -Destination $outImg -Force
            $result = @{
                success = $true
                message = "Blender render completed successfully."
                rendered_image = $outImg
                samples = $samples
            }
            Write-Output ($result | ConvertTo-Json -Compress)
        } else {
            Write-Output "Error: Render completed but output image was not found at $outImg."
            exit 1
        }
    }
} finally {
    if (Test-Path $tempPy) { Remove-Item $tempPy -Force }
}

