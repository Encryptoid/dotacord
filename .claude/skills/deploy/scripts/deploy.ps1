param(
    [Parameter(Mandatory)][string]$Tag,
    [string]$Notes = ""
)

$ErrorActionPreference = "Stop"

$VPS = "root@178.104.97.192"
$RemoteBinary = "/opt/dotacord/dotacord"
$Repo = "Encryptoid/dotacord"
$LocalBinary = "target/release/dotacord"

# Build
Write-Host "Building release..."
cargo build --release
if ($LASTEXITCODE -ne 0) {
    throw "Build failed"
}

# Create GitHub release with binary attached
Write-Host "Creating release $Tag..."
if ($Notes) {
    gh release create $Tag $LocalBinary --title $Tag --notes $Notes
}
else {
    gh release create $Tag $LocalBinary --title $Tag --generate-notes
}
if ($LASTEXITCODE -ne 0) {
    throw "Release creation failed"
}

# Deploy to VPS
$DownloadUrl = "https://github.com/$Repo/releases/download/$Tag/dotacord"
Write-Host "Deploying $Tag to VPS..."

ssh $VPS "curl -sfL '$DownloadUrl' -o $RemoteBinary && chmod +x $RemoteBinary && systemctl restart dotacord.service"
if ($LASTEXITCODE -ne 0) {
    throw "Deploy failed"
}

# Verify service is running
$status = ssh $VPS "systemctl is-active dotacord.service"
if ($status -ne "active") {
    throw "Service not active after deploy: $status"
}

Write-Host "Deployed $Tag - service is active"
