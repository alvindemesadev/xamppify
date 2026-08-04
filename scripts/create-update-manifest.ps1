# Creates the `latest.json` update manifest that tauri-plugin-updater queries
# from the GitHub Release. Upload it as a release asset named `latest.json`
# (alongside the installer and its `.sig` file).
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts/create-update-manifest.ps1
#   powershell ... -Repo "you/yourrepo" -Tag "v0.3.0" -Notes "Highlights of this release"
#
# Requires: a signed release build (TAURI_SIGNING_PRIVATE_KEY env var set) so
# the .sig file exists next to the installer in target\release\bundle\nsis.
# NOTE: keep this file ASCII-only (Windows PowerShell 5.1 parses UTF-8 without
# a BOM as ANSI, which corrupts non-ASCII characters).

param(
  [string]$Repo = "alvindemesadev/xamppify",
  [string]$Tag = "",
  [string]$Version = "",
  [string]$BundleDir = "src-tauri/target/release/bundle/nsis",
  [string]$Notes = "",
  [string]$OutFile = "latest.json"
)

$ErrorActionPreference = "Stop"

if (-not $Version) {
  $Version = (Get-Content package.json -Raw | ConvertFrom-Json).version
}
if (-not $Tag) {
  $Tag = "v$Version"
}

$installer = Get-ChildItem $BundleDir -Filter "*$Version*x64-setup.exe" -ErrorAction SilentlyContinue |
  Where-Object { $_.Name -notlike "*.sig" } |
  Sort-Object LastWriteTime -Descending |
  Select-Object -First 1

if (-not $installer) {
  $installer = Get-ChildItem $BundleDir -Filter "*.exe" -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -notlike "*.sig" } |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
}
if (-not $installer) {
  throw "No installer found in $BundleDir"
}

$sigFile = "$($installer.FullName).sig"
if (-not (Test-Path $sigFile)) {
  throw "Signature file not found: $sigFile - rebuild with TAURI_SIGNING_PRIVATE_KEY set"
}

$manifest = [ordered]@{
  version  = $Version
  notes    = $Notes
  pub_date = [DateTime]::UtcNow.ToString("o")
  platforms = [ordered]@{
    "windows-x86_64" = [ordered]@{
      signature = (Get-Content $sigFile -Raw).Trim()
      url       = "https://github.com/$Repo/releases/download/$Tag/$($installer.Name)"
    }
  }
}

$manifest | ConvertTo-Json -Depth 5 | Set-Content -Path $OutFile -Encoding utf8
Write-Host "Wrote $OutFile (version $Version, installer $($installer.Name))"
Write-Host "Upload these to the GitHub release:"
Write-Host "  - $($installer.Name)"
Write-Host "  - $($installer.Name).sig"
Write-Host "  - latest.json"
