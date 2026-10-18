# MSI を作る（ADR 0025）。Windows のランナーで、make dist が作った exe を WiX 5 で包む。品質の判断は持たない。
#   pwsh packaging/windows/build-msi.ps1 -Exe dist/nene-ruler.exe -OutDir dist
# 版は Cargo.toml の [workspace.package] の version（FR-011）。
param(
    [Parameter(Mandatory = $true)] [string] $Exe,
    [Parameter(Mandatory = $true)] [string] $OutDir
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$root = Resolve-Path (Join-Path $PSScriptRoot '../..')
$version = (Select-String -Path (Join-Path $root 'Cargo.toml') -Pattern '^version = "(.+)"$' | Select-Object -First 1).Matches[0].Groups[1].Value
if (-not $version) { throw 'Cargo.toml に version が無い' }

$msi = Join-Path $OutDir 'NeNe-Ruler-Setup.msi'
wix build (Join-Path $PSScriptRoot 'nene-ruler.wxs') `
    -arch x64 `
    -d "Version=$version" `
    -d "Exe=$(Resolve-Path $Exe)" `
    -d "License=$(Join-Path $root 'LICENSE')" `
    -o $msi
if ($LASTEXITCODE -ne 0) { throw "wix build が失敗した（$LASTEXITCODE）" }

# sha256sum --check と同じ形（小文字の 16 進・空白 2 つ・名前・LF）。Set-Content は CRLF になり Linux で照合できない。
$hash = (Get-FileHash -Algorithm SHA256 $msi).Hash.ToLowerInvariant()
[System.IO.File]::WriteAllText("$msi.sha256", "$hash  NeNe-Ruler-Setup.msi`n")
Write-Output "msi: NeNe Ruler $version"
Get-ChildItem $OutDir
