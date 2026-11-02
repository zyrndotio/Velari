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
  $installedBinary = Join-Path $InstallDirectory "velari.exe"
  Copy-Item $binary.FullName $installedBinary -Force

  $uninstaller = Join-Path $InstallDirectory "uninstall.ps1"
  @'
$ErrorActionPreference = "SilentlyContinue"
$installDirectory = $PSScriptRoot
$uninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\VelaRi"
Remove-Item (Join-Path $installDirectory "velari.exe") -Force
Remove-Item (Join-Path $installDirectory "uninstall.ps1") -Force
Remove-Item $uninstallKey -Recurse -Force
if ((Get-ChildItem $installDirectory -Force | Measure-Object).Count -eq 0) {
  Remove-Item $installDirectory -Force
}
Write-Host "VelaRi was removed for the current Windows user."
'@ | Set-Content -Path $uninstaller -Encoding UTF8

  $installedVersion = (& $installedBinary version 2>$null).Trim()
  if ([string]::IsNullOrWhiteSpace($installedVersion)) { $installedVersion = "unknown" }
  $uninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\VelaRi"
  New-Item -Path $uninstallKey -Force | Out-Null
  New-ItemProperty -Path $uninstallKey -Name "DisplayName" -Value "VelaRi" -PropertyType String -Force | Out-Null
  New-ItemProperty -Path $uninstallKey -Name "DisplayVersion" -Value $installedVersion -PropertyType String -Force | Out-Null
  New-ItemProperty -Path $uninstallKey -Name "Publisher" -Value "zyrndotio" -PropertyType String -Force | Out-Null
  New-ItemProperty -Path $uninstallKey -Name "InstallLocation" -Value $InstallDirectory -PropertyType String -Force | Out-Null
  New-ItemProperty -Path $uninstallKey -Name "UninstallString" -Value "powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$uninstaller`"" -PropertyType String -Force | Out-Null
  New-ItemProperty -Path $uninstallKey -Name "DisplayIcon" -Value $installedBinary -PropertyType String -Force | Out-Null
  New-ItemProperty -Path $uninstallKey -Name "URLInfoAbout" -Value "https://github.com/zyrndotio/Velari" -PropertyType String -Force | Out-Null
  New-ItemProperty -Path $uninstallKey -Name "NoModify" -Value 1 -PropertyType DWord -Force | Out-Null
  New-ItemProperty -Path $uninstallKey -Name "NoRepair" -Value 1 -PropertyType DWord -Force | Out-Null
  Write-Host "Registered VelaRi in Windows Installed apps for the current user."

  $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
  $entries = @($userPath -split ';' | Where-Object { $_ -ne '' })
  if ($entries -notcontains $InstallDirectory) {
    [Environment]::SetEnvironmentVariable("Path", (($entries + $InstallDirectory) -join ';'), "User")
    Write-Host "Added $InstallDirectory to the user PATH. Open a new terminal to use it."
  }
  & $installedBinary version
} finally {
  if (Test-Path $tmp) { Remove-Item $tmp -Recurse -Force }
}
