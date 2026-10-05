# Majeh's PDF Viewer - Windows Dependency Installer
# Requires PowerShell run as Administrator

Write-Host "Checking for Chocolatey..."
if (!(Get-Command choco -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Chocolatey..."
    Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
}

Write-Host "Installing Build Tools (MinGW, Git, FFmpeg)..."
choco install -y mingw git ffmpeg
Write-Host "Dependencies installed. Please restart your terminal."
