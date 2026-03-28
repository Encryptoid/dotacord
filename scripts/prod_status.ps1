$ErrorActionPreference = "Stop"

$VPS = "root@178.104.97.192"

$status = ssh $VPS "systemctl is-active dotacord.service"
Write-Output "Service status: $status"
