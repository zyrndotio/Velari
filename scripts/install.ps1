[CmdletBinding()]
param(
  [string]$Version = "latest",
  [string]$Repository = "zyrndotio/Velari",
  [string]$InstallDirectory = "$env:LOCALAPPDATA\VelaRi\bin"
)

$ErrorActionPreference = "Stop"
$asset = "velari-windows-amd64.zip"
if ($Version -eq "latest") {
  $base = "https://github.com/$Repository/releases/latest/download"
} else {
  $base = "https://github.com/$Repository/releases/download/$Version"
}
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("velari-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
  $archive = Join-Path $tmp $asset
  $checksum = Join-Path $tmp "$asset.sha256"
  Write-Host "Downloading VelaRi ($asset) ..."
  Invoke-WebRequest -Uri "$base/$asset" -OutFile $archive
  Invoke-WebRequest -Uri "$base/$asset.sha256" -OutFile $checksum
  $expected = (Get-Content $checksum -Raw).Trim().Split(' ')[0].ToLowerInvariant()
  $actual = (Get-FileHash -Algorithm SHA256 -Path $archive).Hash.ToLowerInvariant()
  if ($expected -ne $actual) { throw "Checksum verification failed." }

  $unpack = Join-Path $tmp "unpack"
  Expand-Archive -Path $archive -DestinationPath $unpack -Force
  $binary = Get-ChildItem -Path $unpack -Filter "velari-*.exe" -Recurse | Select-Object -First 1
  if ($null -eq $binary) {
    $binary = Get-ChildItem -Path $unpack -Filter "velari.exe" -Recurse | Select-Object -First 1
  }
  if ($null -eq $binary) { throw "Release archive did not contain a VelaRi Windows executable." }
  New-Item -ItemType Directory -Path $InstallDirectory -Force | Out-Null
  Copy-Item $binary.FullName (Join-Path $InstallDirectory "velari.exe") -Force

  $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
  $entries = @($userPath -split ';' | Where-Object { $_ -ne '' })
  if ($entries -notcontains $InstallDirectory) {
    [Environment]::SetEnvironmentVariable("Path", (($entries + $InstallDirectory) -join ';'), "User")
    Write-Host "Added $InstallDirectory to the user PATH. Open a new terminal to use it."
  }
  & (Join-Path $InstallDirectory "velari.exe") version
} finally {
  if (Test-Path $tmp) { Remove-Item $tmp -Recurse -Force }
}
