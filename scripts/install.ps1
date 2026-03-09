#Requires -Version 5.1
<#
.SYNOPSIS
    Installs the Axon compiler (axonc) on Windows.
.DESCRIPTION
    Downloads the latest release of axonc from GitHub and installs it
    to a directory in the user's PATH.
.PARAMETER Version
    Specific version tag to install. Defaults to latest release.
.PARAMETER InstallDir
    Installation directory. Defaults to $env:LOCALAPPDATA\axon\bin.
.EXAMPLE
    .\install.ps1
    .\install.ps1 -Version v0.1.0
#>
[CmdletBinding()]
param(
    [string]$Version,
    [string]$InstallDir = "$env:LOCALAPPDATA\axon\bin"
)

$ErrorActionPreference = 'Stop'

$Repo = "axon-lang/axon"
$Binary = "axonc.exe"
$Target = "x86_64-pc-windows-msvc"

function Write-Info  { param([string]$Msg) Write-Host "[info] $Msg" -ForegroundColor Green }
function Write-Warn  { param([string]$Msg) Write-Host "[warn] $Msg" -ForegroundColor Yellow }
function Write-Err   { param([string]$Msg) Write-Host "[error] $Msg" -ForegroundColor Red; exit 1 }

# Get latest release tag
function Get-LatestVersion {
    $url = "https://api.github.com/repos/$Repo/releases/latest"
    try {
        $response = Invoke-RestMethod -Uri $url -UseBasicParsing
        return $response.tag_name
    }
    catch {
        Write-Err "Failed to fetch latest version: $_"
    }
}

function Main {
    Write-Info "Axon Compiler Installer for Windows"
    Write-Host ""

    # Determine version
    if (-not $Version) {
        Write-Info "Fetching latest release..."
        $Version = Get-LatestVersion
        if (-not $Version) {
            Write-Err "Could not determine latest version."
        }
    }
    Write-Info "Version: $Version"
    Write-Info "Target:  $Target"

    # Prepare install directory
    if (-not (Test-Path $InstallDir)) {
        Write-Info "Creating install directory: $InstallDir"
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    # Download
    $artifactName = "axonc-$Target"
    $downloadUrl = "https://github.com/$Repo/releases/download/$Version/$artifactName"
    $destPath = Join-Path $InstallDir $Binary

    Write-Info "Downloading from $downloadUrl"
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $destPath -UseBasicParsing
    }
    catch {
        Write-Err "Download failed: $_"
    }

    # Add to PATH if not already present
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$InstallDir*") {
        Write-Info "Adding $InstallDir to user PATH"
        [Environment]::SetEnvironmentVariable(
            "Path",
            "$userPath;$InstallDir",
            "User"
        )
        $env:Path = "$env:Path;$InstallDir"
    }

    Write-Host ""
    Write-Info "Installed $Binary $Version to $destPath"
    Write-Info "Restart your terminal, then run 'axonc --help' to get started."
}

Main
