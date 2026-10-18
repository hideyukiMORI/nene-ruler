# Microsoft Store に出す MSIX を作る（ADR 0026）。Windows のランナーで、make dist が作った exe とロゴをそのまま包む。
#   pwsh packaging/windows/build-msix.ps1 -Exe dist/nene-ruler.exe -Assets dist/msix-assets -Portable dist/NeNe-Ruler-windows-portable.zip -OutDir dist
# 🔴 署名しない（認定のあと Store が署名し直す）。鍵と証明書はリポジトリに置かない。
# 身元は packaging/windows/store-identity.json（Partner Center の「製品 ID」の値）の 1 か所だけ。
# 版は Cargo.toml の [workspace.package] の version に .0 を足す（FR-011）。先頭が 0 の版は Store が受け付けない。
param(
    [Parameter(Mandatory = $true)] [string] $Exe,
    [Parameter(Mandatory = $true)] [string] $Assets,
    [Parameter(Mandatory = $true)] [string] $Portable,
    [Parameter(Mandatory = $true)] [string] $OutDir
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

# Windows SDK の道具は PATH に無いので、版のフォルダを名指しする（NeNe Clock と同じ版）。
$sdk = 'C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64'
$makeappx = Join-Path $sdk 'makeappx.exe'
$makepri = Join-Path $sdk 'makepri.exe'
foreach ($tool in @($makeappx, $makepri)) {
    if (-not (Test-Path -LiteralPath $tool -PathType Leaf)) { throw "$tool is not where it is pinned" }
}

$root = Resolve-Path (Join-Path $PSScriptRoot '../..')
$version = (Select-String -Path (Join-Path $root 'Cargo.toml') -Pattern '^version = "(.+)"$' | Select-Object -First 1).Matches[0].Groups[1].Value
if ($version -notmatch '^\d+\.\d+\.\d+$') { throw "Cargo.toml の version が読めない: $version" }
$packageVersion = "$version.0"
$identity = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'store-identity.json') -Raw | ConvertFrom-Json

# MSIX に入れる exe は、ポータブル zip で配る exe と同じでなければならない。
function Get-ZipEntryHash([string] $archive, [string] $entryName) {
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zip = [IO.Compression.ZipFile]::OpenRead((Resolve-Path $archive))
    try {
        $entry = $zip.GetEntry($entryName)
        if ($null -eq $entry) { throw "$archive に $entryName が無い" }
        $stream = $entry.Open()
        try { return [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($stream)) }
        finally { $stream.Dispose() }
    }
    finally { $zip.Dispose() }
}
$exeHash = (Get-FileHash -LiteralPath $Exe -Algorithm SHA256).Hash
if ((Get-ZipEntryHash $Portable 'NeNe-Ruler-windows-portable/nene-ruler.exe') -ne $exeHash) {
    throw 'exe がポータブル zip の中の exe と違う'
}

$work = Join-Path $OutDir 'msix-work'
$layout = Join-Path $work 'layout'
if (Test-Path -LiteralPath $work) { Remove-Item -LiteralPath $work -Recurse -Force }
New-Item -ItemType Directory -Force -Path (Join-Path $layout 'Assets') | Out-Null
Copy-Item -LiteralPath $Exe -Destination (Join-Path $layout 'nene-ruler.exe')
$logos = @(Get-ChildItem -LiteralPath $Assets -Filter '*.png' -File)
if ($logos.Count -ne 25) { throw "MSIX のロゴが 25 枚でない: $($logos.Count)" }
foreach ($logo in $logos) { Copy-Item -LiteralPath $logo.FullName -Destination (Join-Path $layout 'Assets') }

$escape = { param($text) [Security.SecurityElement]::Escape($text) }
# MinVersion は Windows 10 version 2004（NeNe Loupe と同じ）。runFullTrust は Win32 のデスクトップアプリとして動くため。
$manifest = @"
<?xml version="1.0" encoding="utf-8"?>
<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
         xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
         xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
         IgnorableNamespaces="uap rescap">
  <Identity Name="$(& $escape $identity.identityName)"
            Publisher="$(& $escape $identity.publisher)"
            Version="$packageVersion"
            ProcessorArchitecture="x64" />
  <Properties>
    <DisplayName>NeNe Ruler</DisplayName>
    <PublisherDisplayName>$(& $escape $identity.publisherDisplayName)</PublisherDisplayName>
    <Logo>Assets\StoreLogo.png</Logo>
  </Properties>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.19041.0" MaxVersionTested="10.0.26100.0" />
  </Dependencies>
  <Resources>
    <Resource Language="en-us" />
  </Resources>
  <Applications>
    <Application Id="NeNeRuler" Executable="nene-ruler.exe" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements DisplayName="NeNe Ruler"
                          Description="A tiny transparent screen ruler that measures in physical pixels."
                          BackgroundColor="transparent"
                          Square150x150Logo="Assets\Square150x150Logo.png"
                          Square44x44Logo="Assets\Square44x44Logo.png" />
    </Application>
  </Applications>
  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
  </Capabilities>
</Package>
"@
[IO.File]::WriteAllText((Join-Path $layout 'AppxManifest.xml'), $manifest, [Text.UTF8Encoding]::new($false))

# 倍率別・targetsize 別のロゴを Windows が選べるよう、資源の索引（resources.pri）を作る。
# 既定の設定は言語・倍率ごとに別の資源パッケージへ分けるので、その指定を外して 1 つの索引にする（Loupe と同じ）。
$priConfig = Join-Path $work 'priconfig.xml'
& $makepri createconfig /cf $priConfig /dq en-US /pv 10.0.0 /o | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'makepri createconfig が失敗した' }
$config = [regex]::Replace([IO.File]::ReadAllText($priConfig), '(?s)\s*<packaging>.*?</packaging>', '')
[IO.File]::WriteAllText($priConfig, $config, [Text.UTF8Encoding]::new($false))
& $makepri new /pr $layout /cf $priConfig /mn (Join-Path $layout 'AppxManifest.xml') /of (Join-Path $layout 'resources.pri') /o | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'makepri new が失敗した' }

# 先頭が 0 の版は Store が受け付けない。作りはするが、提出できない名前にして警告する（NeNe Clock と同じ）。
$name = if ($version.StartsWith('0.')) { 'NeNe-Ruler-store-NOT-SUBMITTABLE.msix' } else { 'NeNe-Ruler-store.msix' }
if ($version.StartsWith('0.')) { Write-Warning "MSIX の版 $packageVersion は先頭が 0 なので Store に出せない" }
$msix = Join-Path $OutDir $name
& $makeappx pack /o /d $layout /p $msix | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'makeappx pack が失敗した' }

# 包んだ物を開き直し、exe が同じことを確かめる。
$verify = Join-Path $work 'verify'
& $makeappx unpack /o /p $msix /d $verify | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'makeappx unpack が失敗した' }
if ((Get-FileHash -LiteralPath (Join-Path $verify 'nene-ruler.exe') -Algorithm SHA256).Hash -ne $exeHash) {
    throw 'MSIX の中の exe が配る exe と違う'
}
Remove-Item -LiteralPath $work -Recurse -Force

$hash = (Get-FileHash -Algorithm SHA256 $msix).Hash.ToLowerInvariant()
[System.IO.File]::WriteAllText("$msix.sha256", "$hash  $name`n")
Write-Output "msix: $name $packageVersion $($identity.identityName) / $($identity.publisher) / $($identity.publisherDisplayName)"
Write-Output "exe SHA256 (portable zip = MSIX): $exeHash"
