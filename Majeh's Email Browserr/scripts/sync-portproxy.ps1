# Sync Windows portproxy + firewall so the WSL dev server (trunk, port 8080)
# is reachable from other devices on the LAN at http://<Windows-IP>:8080.
#
# Usage: run in an ELEVATED PowerShell whenever the WSL IP changes
# (Windows reboot / wsl --shutdown), or register as a boot task (see README):
#   powershell -ExecutionPolicy Bypass -File sync-portproxy.ps1

# 1. Detect current WSL IP
$wslIp = (wsl.exe -e hostname -I).Trim().Split(" ")[0]
if (-not $wslIp) { Write-Host "ERROR: could not detect WSL IP (is WSL running?)"; exit 1 }
Write-Host "WSL IP: $wslIp"

# 2. Replace the portproxy entry (delete is best-effort; add may fail if not elevated)
cmd /c "netsh interface portproxy delete v4tov4 listenport=8080 listenaddress=0.0.0.0 >nul 2>&1"
cmd /c "netsh interface portproxy add v4tov4 listenport=8080 listenaddress=0.0.0.0 connectport=8080 connectaddress=$wslIp"
if ($LASTEXITCODE -ne 0) { Write-Host "ERROR: netsh add failed - run this from an Administrator PowerShell."; exit 1 }
Write-Host "Portproxy: 0.0.0.0:8080 -> ${wslIp}:8080"

# 3. Ensure the firewall rule exists (idempotent)
if (-not (Get-NetFirewallRule -DisplayName "WSL trunk 8080" -ErrorAction SilentlyContinue)) {
    New-NetFirewallRule -DisplayName "WSL trunk 8080" -Direction Inbound -LocalPort 8080 -Protocol TCP -Action Allow | Out-Null
    Write-Host "Firewall rule 'WSL trunk 8080' created."
} else {
    Write-Host "Firewall rule 'WSL trunk 8080' already exists."
}

# 4. Show final state
cmd /c "netsh interface portproxy show all"
