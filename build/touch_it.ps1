# Get the current user's home directory and construct the file path
$currentDir = Get-Location
$filePath = Join-Path -Path $currentDir -ChildPath "build.rs"

# Print debug information
Write-Host "File Path: $filePath"

# Update the timestamp of the file
(Get-Item $filePath).LastWriteTime = Get-Date
Write-Host "File timestamp updated: $filePath"

# Wait for 1 second
Start-Sleep -Seconds 1
Write-Host "Waited for 1 second"