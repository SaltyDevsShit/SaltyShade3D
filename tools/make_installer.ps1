<#
.SYNOPSIS
  Package the built SaltyShade 3D into dist\SaltyShade3D-<version>-Setup.exe (the file you send
  people). Used by Make-Installer.bat locally and by the GitHub release workflow.

.PARAMETER AppExe   The built app (default: dist\SaltyShade3D.exe from the updater).
.PARAMETER Version  Release version (default: the current UTC time, yyyy.MM.dd.HHmm).
.PARAMETER Repo     owner/name the installed copies update from (default: release.json).
#>
param(
    [string]$AppExe,
    [string]$Version,
    [string]$Repo
)

$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
$Dist = Join-Path $Root 'dist'
New-Item -ItemType Directory -Force $Dist | Out-Null

if (-not $AppExe) { $AppExe = Join-Path $Dist 'SaltyShade3D.exe' }
if (-not (Test-Path $AppExe)) { throw "No built app at $AppExe. Run Update-SaltyShade.bat first." }
if (-not $Version) { $Version = (Get-Date).ToUniversalTime().ToString('yyyy.MM.dd.HHmm') }
if (-not $Repo) { $Repo = (Get-Content (Join-Path $Root 'release.json') -Raw | ConvertFrom-Json).repo }

# The launcher (what the desktop icon opens).
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
$cargo = if ($cargo) { $cargo.Source } else { Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe' }
$mingw = Get-ChildItem -Directory -ErrorAction SilentlyContinue (Join-Path $env:LOCALAPPDATA 'Microsoft\WinGet\Packages\BrechtSanders.WinLibs*\mingw64\bin') | Select-Object -First 1
if ($mingw) { $env:PATH = "$($mingw.FullName);$env:PATH" }
$launcherTarget = if ($env:RUNNER_TEMP) { Join-Path $env:RUNNER_TEMP 'launcher-target' } else { Join-Path $env:LOCALAPPDATA 'SaltyShade3D\launcher-target' }
Push-Location (Join-Path $Root 'launcher')
try {
    $env:CARGO_TARGET_DIR = $launcherTarget
    & $cargo build --release
    if ($LASTEXITCODE -ne 0) { throw 'Building the launcher failed.' }
} finally { Pop-Location; Remove-Item Env:\CARGO_TARGET_DIR -ErrorAction SilentlyContinue }
$launcher = Join-Path $launcherTarget 'release\SaltyShade3D-Launcher.exe'

# version.txt and update.json, installed next to the app.
$stage = Join-Path ([IO.Path]::GetTempPath()) "SaltyShade3D-stage-$Version"
New-Item -ItemType Directory -Force $stage | Out-Null
[IO.File]::WriteAllText((Join-Path $stage 'version.txt'), $Version)
[IO.File]::WriteAllText((Join-Path $stage 'update.json'), (@{ repo = $Repo } | ConvertTo-Json))

$iscc = @(
    (Get-Command ISCC.exe -ErrorAction SilentlyContinue | ForEach-Object Source),
    (Join-Path $env:LOCALAPPDATA 'Programs\Inno Setup 6\ISCC.exe'),
    (Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe'),
    (Join-Path $env:ProgramFiles 'Inno Setup 6\ISCC.exe')
) | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
if (-not $iscc) { throw 'Inno Setup is not installed (winget install JRSoftware.InnoSetup).' }

& $iscc /Qp "/DAppVersion=$Version" "/DAppExe=$((Resolve-Path $AppExe).Path)" "/DLauncherExe=$launcher" `
    "/DUpstreamDir=$(Join-Path $Root 'upstream')" "/DRepo=$Repo" "/DOutDir=$Dist" "/DStageDir=$stage" `
    (Join-Path $Root 'installer\SaltyShade3D.iss')
if ($LASTEXITCODE -ne 0) { throw 'Inno Setup failed.' }
Remove-Item -Recurse -Force $stage
$setup = Join-Path $Dist "SaltyShade3D-$Version-Setup.exe"
Write-Host "[SaltyShade] Installer: $setup" -ForegroundColor Green
if ($env:GITHUB_OUTPUT) {
    Add-Content $env:GITHUB_OUTPUT "setup=$setup"
    Add-Content $env:GITHUB_OUTPUT "version=$Version"
}
