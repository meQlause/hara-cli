# HARA CLI - PowerShell 7 (Core) Installer
$ErrorActionPreference = "Stop"

$Repo = "meQlause/hara-cli"
$BinaryName = "hara"
$Asset = "hara-windows-x86_64.zip"
$InstallDir = Join-Path $HOME ".local\bin"

Write-Host "`n>>> HARA Installer for PowerShell 7+ <<<`n" -ForegroundColor Cyan

# 1. API Check
$ApiUrl = "https://api.github.com/repos/$Repo/releases/latest"
$Release = Invoke-RestMethod -Uri $ApiUrl -Headers @{"Accept"="application/vnd.github.v3+json"}
$Version = $Release.tag_name
Write-Host "[+] Found latest version: $Version"

# 2. Prep
if (-not (Test-Path $InstallDir)) { New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null }
$TempDir = Join-Path $env:TEMP "hara-pwsh-$(Get-Random)"
New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

# 3. Download & Extract
$DownloadUrl = "https://github.com/$Repo/releases/download/$Version/$Asset"
$ZipFile = Join-Path $TempDir $Asset
Write-Host "[+] Downloading $Asset..."
Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipFile
Expand-Archive -Path $ZipFile -DestinationPath $TempDir -Force

# 4. Install
$ExeDest = Join-Path $InstallDir "${BinaryName}.exe"
Move-Item -Path (Join-Path $TempDir "${BinaryName}.exe") -Destination $ExeDest -Force
Write-Host "[+] Binary installed to $ExeDest"

# 5. Path
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$currentPath", "User")
    $env:Path = "$InstallDir;$env:Path"
    Write-Host "[+] Path updated."
}

Remove-Item $TempDir -Recurse -Force
Write-Host "`n✨ Success! Restart your terminal and run 'hara --version'`n" -ForegroundColor Green
