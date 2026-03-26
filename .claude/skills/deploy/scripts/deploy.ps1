param(
    [Parameter(Mandatory)][string]$Tag,
    [string]$Notes = ""
)

$ErrorActionPreference = "Stop"

$VPS = "root@178.104.97.192"
$RemoteBinary = "/opt/dotacord/dotacord"
$Repo = "Encryptoid/dotacord"
$LocalBinary = "target/release/dotacord"
$LocalContext = "context/dotacord.md"
$RemoteContext = "/opt/dotacord/context/dotacord.md"

# Build
Write-Host "Building release..."
cargo build --release
if ($LASTEXITCODE -ne 0) {
    throw "Build failed"
}

# Create GitHub release with binary attached
Write-Host "Creating release $Tag..."
if ($Notes) {
    gh release create $Tag $LocalBinary $LocalContext --title $Tag --notes $Notes
}
else {
    gh release create $Tag $LocalBinary $LocalContext --title $Tag --generate-notes
}
if ($LASTEXITCODE -ne 0) {
    throw "Release creation failed"
}

# Stop the running service
Write-Host "Stopping dotacord service..."
ssh $VPS "systemctl stop dotacord.service"
if ($LASTEXITCODE -ne 0) {
    throw "Failed to stop service"
}

# Download the release binary
$DownloadUrl = "https://github.com/$Repo/releases/download/$Tag/dotacord"
Write-Host "Downloading $Tag to VPS..."
ssh $VPS "curl -sfL '$DownloadUrl' -o $RemoteBinary"
if ($LASTEXITCODE -ne 0) {
    throw "Download failed"
}

ssh $VPS "chmod +x $RemoteBinary"

# Download the context file
$ContextUrl = "https://github.com/$Repo/releases/download/$Tag/dotacord.md"
Write-Host "Downloading context file to VPS..."
ssh $VPS "mkdir -p /opt/dotacord/context && curl -sfL '$ContextUrl' -o $RemoteContext"
if ($LASTEXITCODE -ne 0) {
    throw "Context file download failed"
}

# Start the service
Write-Host "Starting dotacord service..."
ssh $VPS "systemctl start dotacord.service"
if ($LASTEXITCODE -ne 0) {
    throw "Failed to start service"
}

# Verify service is running
$status = ssh $VPS "systemctl is-active dotacord.service"
if ($status -ne "active") {
    throw "Service not active after deploy: $status"
}

Write-Host "Deployed $Tag - service is active"
