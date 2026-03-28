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
$LocalConfig = "dotacord.release.toml"
$RemoteConfig = "/opt/dotacord/dotacord.toml"

# Diff config: compare local release config with VPS config
Write-Host "Checking config diff..."
$TempRemote = [System.IO.Path]::GetTempFileName()
scp "${VPS}:${RemoteConfig}" $TempRemote 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "  No remote config found - will upload local config."
    $ConfigAction = "upload"
}
else {
    $diff = diff $LocalConfig $TempRemote
    if ($diff) {
        Write-Host "  Config differences detected:"
        Write-Host ($diff | Out-String)
        $answer = Read-Host "Upload local config to VPS? (y/n)"
        $ConfigAction = if ($answer -eq "y") { "upload" } else { "skip" }
    }
    else {
        Write-Host "  Config is up to date."
        $ConfigAction = "skip"
    }
}
Remove-Item $TempRemote -ErrorAction SilentlyContinue

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

# Upload config if needed
if ($ConfigAction -eq "upload") {
    Write-Host "Uploading config to VPS..."
    scp $LocalConfig "${VPS}:${RemoteConfig}"
    if ($LASTEXITCODE -ne 0) {
        throw "Config upload failed"
    }
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
