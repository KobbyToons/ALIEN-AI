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

$pyCode = $parsed.python_code
if ([string]::IsNullOrWhiteSpace($pyCode)) {
    Write-Output "Error: No python_code provided."
    exit 1
}

$openGui = if ($null -ne $parsed.open_gui) { [bool]$parsed.open_gui } else { $false }

$tempPy = [System.IO.Path]::GetTempFileName() + ".py"
[System.IO.File]::WriteAllText($tempPy, $pyCode, [System.Text.Encoding]::UTF8)

try {
    if ($openGui) {
        Start-Process -FilePath "$blenderPath" -ArgumentList @("-P", "$tempPy")
        $result = @{
            success = $true
            message = "Blender launched with script running in GUI mode."
            script_path = $tempPy
        }
        Write-Output ($result | ConvertTo-Json -Compress)
    } else {
        $output = & "$blenderPath" -b -P "$tempPy" 2>&1 | Out-String
        $result = @{
            success = $true
            message = "Blender python script executed in headless mode."
            output = $output.Trim()
        }
        Write-Output ($result | ConvertTo-Json -Compress)
    }
} finally {
    if (-not $openGui -and (Test-Path $tempPy)) {
        Remove-Item $tempPy -Force
    }
}

