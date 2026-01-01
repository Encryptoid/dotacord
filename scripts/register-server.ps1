param(
    [Parameter(Mandatory=$true, Position=0)]
    [long]$ServerId,

    [Parameter(Mandatory=$true, Position=1)]
    [string]$ServerName
)

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$dbPath = Join-Path $scriptDir ".." "data" "dotacord.db"
$dbPath = [System.IO.Path]::GetFullPath($dbPath)

if (-not (Test-Path $dbPath)) {
    Write-Error "Database not found at: $dbPath"
    exit 1
}

$sql = @"
INSERT INTO servers (server_id, server_name, is_sub_week, is_sub_month, is_sub_reload)
VALUES ($ServerId, '$($ServerName -replace "'", "''")', 0, 0, 0);
"@

Write-Host "Registering server: $ServerName ($ServerId)"
sqlite3 $dbPath $sql

if ($LASTEXITCODE -eq 0) {
    Write-Host "Server registered successfully."
} else {
    Write-Error "Failed to register server."
    exit 1
}
