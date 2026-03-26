param(
    [Parameter(Mandatory)][string]$Tag
)

$ErrorActionPreference = "Stop"

$VPS = "root@178.104.97.192"
$RemoteBinary = "/opt/dotacord/dotacord"
$Repo = "Encryptoid/dotacord"
$DownloadUrl = "https://github.com/$Repo/releases/download/$Tag/dotacord"

# Download release binary to VPS and restart
Write-Host "Rolling back to $Tag..."

ssh $VPS "curl -sfL '$DownloadUrl' -o $RemoteBinary && chmod +x $RemoteBinary && systemctl restart dotacord.service"
if ($LASTEXITCODE -ne 0) {
    throw "Rollback failed"
}

# Verify service is running
$status = ssh $VPS "systemctl is-active dotacord.service"
if ($status -ne "active") {
    throw "Service not active after rollback: $status"
}

Write-Host "Rolled back to $Tag - service is active"
