[CmdletBinding()]
param(
  [string]$Version = 'latest',
  [switch]$Pre,
  [string]$Target = 'x86_64-pc-windows-msvc',
  [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\Aureline\bin'),
  [string]$Repo = 'pixelscortex/aureline-orm',
  [string]$DownloadRoot
)

$ErrorActionPreference = 'Stop'
$BinaryName = 'aureline.exe'

function Resolve-AurelineTag {
  param([string]$Repo, [string]$Version, [bool]$Pre)

  $api = "https://api.github.com/repos/$Repo/releases"
  if ($Pre) {
    $page = 1
    while ($true) {
      $releases = Invoke-RestMethod -Uri "$api?per_page=100&page=$page" -Headers @{ 'User-Agent' = 'aureline-installer' }
      if (-not $releases -or $releases.Count -eq 0) { break }
      $release = @($releases | Where-Object { -not $_.draft -and $_.prerelease } | Select-Object -First 1)
      if ($release.Count -gt 0) { return $release[0].tag_name }
      $page += 1
    }
    throw "no GitHub prerelease found for $Repo"
  }

  if ($Version -eq 'latest') {
    $latest = Invoke-RestMethod -Uri "$api/latest" -Headers @{ 'User-Agent' = 'aureline-installer' }
    if (-not $latest.tag_name) { throw "failed to resolve latest release for $Repo" }
    return $latest.tag_name
  }

  if ($Version.StartsWith('v')) { return $Version }
  return "v$Version"
}

function Copy-Or-DownloadAsset {
  param([string]$SourceRoot, [string]$Tag, [string]$Asset, [string]$Destination, [string]$Repo)

  if ([string]::IsNullOrWhiteSpace($SourceRoot)) {
    $uri = "https://github.com/$Repo/releases/download/$Tag/$Asset"
    Invoke-WebRequest -Uri $uri -OutFile $Destination -Headers @{ 'User-Agent' = 'aureline-installer' }
    return
  }

  $root = $SourceRoot.TrimEnd('/', '\')
  if ($root -match '^https?://') {
    Invoke-WebRequest -Uri "$root/$Tag/$Asset" -OutFile $Destination -Headers @{ 'User-Agent' = 'aureline-installer' }
    return
  }

  if ($root -match '^file://') {
    $localRoot = ([Uri]$root).LocalPath
  } else {
    $localRoot = $root
  }

  $source = Join-Path (Join-Path $localRoot $Tag) $Asset
  Copy-Item $source $Destination
}

$tag = Resolve-AurelineTag -Repo $Repo -Version $Version -Pre ([bool]$Pre)
$resolvedVersion = $tag.TrimStart('v')
$asset = "aureline-$resolvedVersion-$Target.zip"

$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Force $tmp | Out-Null
try {
  $archive = Join-Path $tmp $asset
  Write-Host "Downloading $asset"
  Copy-Or-DownloadAsset -SourceRoot $DownloadRoot -Tag $tag -Asset $asset -Destination $archive -Repo $Repo

  $extractDir = Join-Path $tmp 'extract'
  Expand-Archive -Path $archive -DestinationPath $extractDir

  $entries = @(Get-ChildItem -LiteralPath $extractDir -Force)
  if ($entries.Count -ne 1 -or $entries[0].Name -ne $BinaryName -or $entries[0].PSIsContainer) {
    $names = ($entries | ForEach-Object { $_.Name }) -join ', '
    throw "release archive must contain exactly one root $BinaryName; found: $names"
  }

  New-Item -ItemType Directory -Force $InstallDir | Out-Null
  $destination = Join-Path $InstallDir $BinaryName
  Copy-Item -LiteralPath $entries[0].FullName -Destination $destination -Force

  Write-Host "Installed $BinaryName $resolvedVersion to $destination"
  $pathEntries = ($env:PATH -split [IO.Path]::PathSeparator) | Where-Object { $_ }
  if ($pathEntries -notcontains $InstallDir) {
    Write-Host "Add $InstallDir to PATH to run aureline from anywhere."
  }
} finally {
  Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
