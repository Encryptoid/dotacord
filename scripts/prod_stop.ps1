$ErrorActionPreference = "Stop"

$VPS = "root@178.104.97.192"

ssh $VPS "systemctl stop dotacord.service"
if ($LASTEXITCODE -ne 0) {
    throw "Failed to stop service"
}

$status = ssh $VPS "systemctl is-active dotacord.service"
if ($status -eq "inactive") {
    Write-Host "Service stopped successfully."
} else {
    throw "Service not stopped: $status"
}
