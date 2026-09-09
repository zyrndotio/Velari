[CmdletBinding()]
param([string]$Velari = "velari")

$ErrorActionPreference = "Stop"
& $Velari version
if ($LASTEXITCODE -ne 0) { throw "VelaRi version command failed." }

$temp = Join-Path ([IO.Path]::GetTempPath()) ("velari-smoke-" + [Guid]::NewGuid())
New-Item -ItemType Directory -Path $temp | Out-Null
$source = Join-Path $temp "smoke.vr"
@"
begin
    let values be [1, 2, 3]
    assert length(values) == 3
    print "VelaRi Windows smoke test passed"
end
"@ | Set-Content -Path $source -Encoding utf8
& $Velari check $source
if ($LASTEXITCODE -ne 0) { throw "VelaRi check command failed." }
& $Velari run $source
if ($LASTEXITCODE -ne 0) { throw "VelaRi run command failed." }
Remove-Item -Recurse -Force $temp
