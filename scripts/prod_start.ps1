$ErrorActionPreference = "Stop"

$VPS = "root@178.104.97.192"

ssh $VPS "systemctl start dotacord.service"
if ($LASTEXITCODE -ne 0) {
    throw "Failed to start service"
}

$status = ssh $VPS "systemctl is-active dotacord.service"
if ($status -eq "active") {
    Write-Host "Service started successfully."
} else {
    throw "Service not started: $status"
}
