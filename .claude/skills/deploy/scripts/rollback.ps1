param(
    [Parameter(Mandatory)][string]$Tag
)

$ErrorActionPreference = "Stop"

$VPS = "root@178.104.97.192"
$RemoteBinary = "/opt/dotacord/dotacord"
$Repo = "Encryptoid/dotacord"
$DownloadUrl = "https://github.com/$Repo/releases/download/$Tag/dotacord"

# Stop the running service
Write-Host "Stopping dotacord service..."
ssh $VPS "systemctl stop dotacord.service"
if ($LASTEXITCODE -ne 0) {
    throw "Failed to stop service"
}

# Download the release binary
Write-Host "Downloading $Tag to VPS..."
ssh $VPS "curl -sfL '$DownloadUrl' -o $RemoteBinary"
if ($LASTEXITCODE -ne 0) {
    throw "Rollback download failed"
}

ssh $VPS "chmod +x $RemoteBinary"

# Start the service
Write-Host "Starting dotacord service..."
ssh $VPS "systemctl start dotacord.service"
if ($LASTEXITCODE -ne 0) {
    throw "Failed to start service"
}

# Verify service is running
$status = ssh $VPS "systemctl is-active dotacord.service"
if ($status -ne "active") {
    throw "Service not active after rollback: $status"
}

Write-Host "Rolled back to $Tag - service is active"
