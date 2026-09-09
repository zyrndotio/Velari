[CmdletBinding()]
param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:USERPROFILE\.velari\bin"
)

$ErrorActionPreference = "Stop"
$repo = "zyrndotio/Velari"
$release = if ($Version -eq "latest") { "latest" } else { "tags/v$Version" }
$api = "https://api.github.com/repos/$repo/releases/$release"
$metadata = Invoke-RestMethod -Uri $api
$asset = $metadata.assets | Where-Object { $_.name -eq "velari-windows-x86_64.zip" } | Select-Object -First 1
if (-not $asset) { throw "No Windows x86_64 asset was found for release $Version." }

$temp = Join-Path ([IO.Path]::GetTempPath()) ("velari-" + [Guid]::NewGuid())
New-Item -ItemType Directory -Path $temp | Out-Null
$archive = Join-Path $temp "velari.zip"
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $archive
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Expand-Archive -Path $archive -DestinationPath $InstallDir -Force

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$entries = if ($userPath) { $userPath -split ';' } else { @() }
if ($entries -notcontains $InstallDir) {
    [Environment]::SetEnvironmentVariable("Path", (($entries + $InstallDir) -join ';'), "User")
}
& (Join-Path $InstallDir "velari.exe") version
Write-Host "VelaRi installed to $InstallDir. Open a new PowerShell window before using 'velari' globally."
Remove-Item -Recurse -Force $temp
