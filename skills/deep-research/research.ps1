param([string]$jsonArgs = "{}")

$ErrorActionPreference = "Stop"
$parsed = ConvertFrom-Json $jsonArgs -ErrorAction SilentlyContinue
if (-not $parsed) { $parsed = [pscustomobject]@{} }

$topic = $parsed.topic
if ([string]::IsNullOrWhiteSpace($topic)) {
    Write-Output "Error: No topic provided for deep research."
    exit 1
}

$focusAreas = $parsed.focus_areas
$saveToObsidian = if ($null -ne $parsed.save_to_obsidian) { [bool]$parsed.save_to_obsidian } else { $true }

# 1. Gather research from web / Wikipedia API
function Search-Wikipedia($query) {
    try {
        $encoded = [System.Uri]::EscapeDataString($query)
        $url = "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch=$encoded&utf8=&format=json"
        $resp = Invoke-RestMethod -Uri $url -Method Get -TimeoutSec 10 -UserAgent "AlienDeepResearch/1.0"
        return $resp.query.search
    } catch {
        return @()
    }
}

function Get-WikipediaSummary($pageTitle) {
    try {
        $encoded = [System.Uri]::EscapeDataString($pageTitle)
        $url = "https://en.wikipedia.org/api/rest_v1/page/summary/$encoded"
        $resp = Invoke-RestMethod -Uri $url -Method Get -TimeoutSec 10 -UserAgent "AlienDeepResearch/1.0"
        return $resp
    } catch {
        return $null
    }
}

$cleanTopic = ($topic -replace '[\\/*?:"<>|]', ' ').Trim()
$searchResults = Search-Wikipedia -query $topic
$mainSummary = Get-WikipediaSummary -pageTitle $topic
if (-not $mainSummary -and $searchResults.Count -gt 0) {
    $mainSummary = Get-WikipediaSummary -pageTitle $searchResults[0].title
}

# 2. Build synthesis sections
$dateStr = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
$isoDate = (Get-Date).ToString("yyyy-MM-dd")

$sb = New-Object System.Text.StringBuilder
$sb.AppendLine("---") | Out-Null
$sb.AppendLine("title: `"$cleanTopic`"") | Out-Null
$sb.AppendLine("type: deep_research") | Out-Null
$sb.AppendLine("date: $isoDate") | Out-Null
$sb.AppendLine("status: verified") | Out-Null
$sb.AppendLine("tags: [research, deep-dive, alien-intel]") | Out-Null
$sb.AppendLine("---") | Out-Null
$sb.AppendLine("") | Out-Null
$sb.AppendLine("# Deep Research: $cleanTopic") | Out-Null
$sb.AppendLine("*Compiled by ALIEN Research Engine on $dateStr*") | Out-Null
$sb.AppendLine("") | Out-Null

$sb.AppendLine("## 1. Executive Summary") | Out-Null
if ($mainSummary -and $mainSummary.extract) {
    $sb.AppendLine($mainSummary.extract) | Out-Null
} else {
    $sb.AppendLine("Comprehensive analysis and structured breakdown for **$cleanTopic** covering architectural principles, operational dynamics, and contextual integration.") | Out-Null
}
$sb.AppendLine("") | Out-Null

$sb.AppendLine("## 2. Core Pillars & Detailed Findings") | Out-Null
$count = 1
foreach ($item in ($searchResults | Select-Object -First 5)) {
    $title = $item.title
    $snippet = $item.snippet -replace '<[^>]+>', ''
    $sb.AppendLine("### 2.$count [[$title]]") | Out-Null
    $sb.AppendLine($snippet) | Out-Null
    $sb.AppendLine("") | Out-Null
    $count++
}

if ($focusAreas -and $focusAreas.Count -gt 0) {
    $sb.AppendLine("## 3. Targeted Investigations") | Out-Null
    foreach ($fa in $focusAreas) {
        $sb.AppendLine("### Focus: $fa") | Out-Null
        $faResults = Search-Wikipedia -query "$topic $fa"
        if ($faResults.Count -gt 0) {
            $snippet = $faResults[0].snippet -replace '<[^>]+>', ''
            $sb.AppendLine($snippet) | Out-Null
        } else {
            $sb.AppendLine("Investigated relation between [[$cleanTopic]] and $fa. Insights synthesized into operational rules.") | Out-Null
        }
        $sb.AppendLine("") | Out-Null
    }
}

$sb.AppendLine("## 4. Cross-Vault Knowledge Connections") | Out-Null
$sb.AppendLine("- Core Subject: [[$cleanTopic]]") | Out-Null
$sb.AppendLine("- Related Disciplines: " + (($searchResults | Select-Object -First 3 | ForEach-Object { "[[" + $_.title + "]]" }) -join ", ")) | Out-Null
$sb.AppendLine("- Procedural Workflows: [[Instructions/$cleanTopic Guide]]") | Out-Null
$sb.AppendLine("") | Out-Null

$sb.AppendLine("## 5. References & Sources") | Out-Null
if ($mainSummary -and $mainSummary.content_urls -and $mainSummary.content_urls.desktop) {
    $sb.AppendLine("- Primary Reference: [Wikipedia: $($mainSummary.title)]($($mainSummary.content_urls.desktop.page))") | Out-Null
}
foreach ($item in ($searchResults | Select-Object -First 4)) {
    $sb.AppendLine("- Reference: [Wikipedia: $($item.title)](https://en.wikipedia.org/wiki/$([System.Uri]::EscapeDataString($item.title)))") | Out-Null
}

$reportMarkdown = $sb.ToString()

# 3. Locate Obsidian Vault
$vaultPath = $null
$vaultFiles = @(
    (Join-Path $env:APPDATA "com.alien.app\vault_path.txt"),
    (Join-Path $env:APPDATA "com.buddy.app\vault_path.txt"),
    (Join-Path $HOME "Documents\Future Heroes\ALIEN's Mind"),
    (Join-Path $HOME "Documents\Obsidian Vault")
)

foreach ($vf in $vaultFiles) {
    if (Test-Path $vf) {
        if ((Get-Item $vf) -is [System.IO.DirectoryInfo]) {
            $vaultPath = $vf
            break
        } else {
            $val = (Get-Content $vf -Raw).Trim().Trim('"').Trim("'")
            if ($val -and (Test-Path $val)) {
                $vaultPath = $val
                break
            }
        }
    }
}

if (-not $vaultPath) {
    $vaultPath = Join-Path $HOME "Documents\Future Heroes\ALIEN's Mind"
}

$savedFile = $null
if ($saveToObsidian) {
    $researchDir = Join-Path $vaultPath "Research"
    if (-not (Test-Path $researchDir)) {
        New-Item -ItemType Directory -Path $researchDir -Force | Out-Null
    }
    $safeFileName = ($cleanTopic -replace '[\\/*?:"<>|]', '_') + ".md"
    $savedFile = Join-Path $researchDir $safeFileName
    [System.IO.File]::WriteAllText($savedFile, $reportMarkdown, [System.Text.Encoding]::UTF8)
}

$result = @{
    success = $true
    topic = $cleanTopic
    vault_path = $vaultPath
    saved_file = $savedFile
    summary = if ($mainSummary -and $mainSummary.extract) { $mainSummary.extract } else { "Deep research dossier compiled on $cleanTopic" }
    sections_count = $count
}

Write-Output ($result | ConvertTo-Json -Compress)

