<#
.SYNOPSIS
  SaltyShade 3D auto-updater: pulls the latest TextureCraft from GitHub, re-applies your
  SaltyShade design (overlay/) on top, builds it, and swaps the new build in.

.DESCRIPTION
  upstream/ is a pristine clone of https://github.com/storytold/texturecraft. Every update resets
  it to the newest commit, then tools/apply_overlay.py lays overlay/ over it. Your design lives
  only in overlay/, which the updater never touches, so it survives every update.

  If upstream changed so much that a required part of the design no longer fits, or the build
  fails, the update stops and the previous SaltyShade 3D build keeps working.

.PARAMETER Launch       Start SaltyShade 3D, then update in the background (used by SaltyShade3D.bat).
.PARAMETER Force        Rebuild even if nothing changed.
.PARAMETER CheckOnly    Only report whether an update is available.
.PARAMETER Quiet        No pop-up messages (scheduled runs).
.PARAMETER InstallSchedule  Also check for updates daily via Windows Task Scheduler.
.PARAMETER RemoveSchedule   Remove that scheduled task.
.PARAMETER InstallRust  Install the Rust toolchain (rustup) if it is missing.
.PARAMETER NoBuild      Fetch upstream and apply the overlay only (to inspect upstream/), no build.
#>
param(
    [switch]$Launch,
    [switch]$Force,
    [switch]$CheckOnly,
    [switch]$Quiet,
    [switch]$InstallSchedule,
    [switch]$RemoveSchedule,
    [switch]$InstallRust,
    [switch]$NoBuild
)

$ErrorActionPreference = 'Stop'
$Root      = Split-Path -Parent $PSScriptRoot
$Repo      = 'https://github.com/storytold/texturecraft'
$Branch    = 'main'
$Upstream  = Join-Path $Root 'upstream'
$Overlay   = Join-Path $Root 'overlay'
$Dist      = Join-Path $Root 'dist'
$State     = Join-Path $Root 'state'
$Exe       = Join-Path $Dist 'SaltyShade3D.exe'
$NewExe    = Join-Path $Dist 'SaltyShade3D.new.exe'
$TaskName  = 'SaltyShade 3D auto-update'
New-Item -ItemType Directory -Force $Dist, $State | Out-Null

function Say($msg, $color = 'Gray') {
    Write-Host "[SaltyShade] $msg" -ForegroundColor $color
    Add-Content -Path (Join-Path $State 'update.log') -Value ("{0:u}  {1}" -f (Get-Date), $msg) -Encoding utf8
}

function Popup($msg, $title = 'SaltyShade 3D') {
    if ($Quiet) { return }
    Add-Type -AssemblyName System.Windows.Forms
    [System.Windows.Forms.MessageBox]::Show($msg, $title) | Out-Null
}

function Invoke-Git { & git -c core.longpaths=true @args; if ($LASTEXITCODE -ne 0) { throw "git $($args -join ' ') failed" } }

function Find-Cargo {
    $c = Get-Command cargo -ErrorAction SilentlyContinue
    if ($c) { return $c.Source }
    $p = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
    if (Test-Path $p) { return $p }
    return $null
}

function Install-Rust {
    $msvc = Test-Path "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    # Without Visual Studio's C++ build tools, the GNU toolchain links on its own.
    $hostTriple = if ($msvc) { 'x86_64-pc-windows-msvc' } else { 'x86_64-pc-windows-gnu' }
    $init = Join-Path $env:TEMP 'rustup-init.exe'
    Say "Installing Rust (stable, $hostTriple) from rustup.rs ..." Cyan
    Invoke-WebRequest -UseBasicParsing 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe' -OutFile $init
    & $init -y --default-toolchain stable --default-host $hostTriple --profile minimal
    if ($LASTEXITCODE -ne 0) { throw 'rustup-init failed' }
    if (-not $msvc) {
        Say 'Installing MinGW (WinLibs) for the GNU toolchain ...' Cyan
        & winget install --id BrechtSanders.WinLibs.POSIX.MSVCRT -e --scope user --silent --accept-package-agreements --accept-source-agreements --disable-interactivity
    }
}

function Overlay-Hash {
    # Any edit to your design (or to the script that applies it) triggers a rebuild.
    $files = @(Get-ChildItem -Recurse -File $Overlay) + (Get-Item (Join-Path $PSScriptRoot 'apply_overlay.py'))
    $sha = [System.Security.Cryptography.SHA256]::Create()
    $buf = New-Object System.IO.MemoryStream
    foreach ($f in ($files | Sort-Object FullName)) {
        $name = [Text.Encoding]::UTF8.GetBytes($f.FullName.Substring($Root.Length))
        $buf.Write($name, 0, $name.Length)
        $bytes = [IO.File]::ReadAllBytes($f.FullName)
        $buf.Write($bytes, 0, $bytes.Length)
    }
    return ([BitConverter]::ToString($sha.ComputeHash($buf.ToArray())) -replace '-', '').ToLower()
}

function Read-State($name) { $p = Join-Path $State $name; if (Test-Path $p) { (Get-Content $p -Raw).Trim() } else { '' } }
function Write-State($name, $value) { Set-Content -Path (Join-Path $State $name) -Value $value -Encoding ascii }

function Remote-Sha {
    $line = & git ls-remote $Repo "refs/heads/$Branch"
    if ($LASTEXITCODE -ne 0 -or -not $line) { throw "cannot reach $Repo" }
    return ($line -split '\s+')[0]
}

function Swap-PendingBuild {
    if (Test-Path $NewExe) {
        try { Move-Item -Force $NewExe $Exe; Say 'Installed the build downloaded last time.' Green } catch { }
    }
}

# --- Scheduling --------------------------------------------------------------------------------
if ($RemoveSchedule) {
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
    Say 'Daily update check removed.' Green
    return
}
if ($InstallSchedule) {
    $action  = New-ScheduledTaskAction -Execute 'powershell.exe' -Argument "-NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$PSCommandPath`" -Quiet"
    $trigger = New-ScheduledTaskTrigger -Daily -At 12:00
    $settings = New-ScheduledTaskSettingsSet -StartWhenAvailable -RunOnlyIfNetworkAvailable
    Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Settings $settings -Force | Out-Null
    Say "Daily update check installed (Task Scheduler: '$TaskName')." Green
    return
}

# --- Launch first, update after ----------------------------------------------------------------
Swap-PendingBuild
if ($Launch -and (Test-Path $Exe)) {
    Start-Process -FilePath $Exe -WorkingDirectory $Dist
}

$lock = Join-Path $State 'update.lock'
if ((Test-Path $lock) -and ((Get-Date) - (Get-Item $lock).LastWriteTime).TotalHours -lt 2) {
    Say 'Another update is already running.' Yellow
    return
}

try {
    $remote = Remote-Sha
    $overlayHash = Overlay-Hash
    $changed = $Force -or -not (Test-Path $Exe) -or ($remote -ne (Read-State 'upstream.sha')) -or ($overlayHash -ne (Read-State 'overlay.sha'))
    if (-not $changed) { Say "Up to date (TextureCraft $($remote.Substring(0,7)))." Green; return }
    if ($CheckOnly) { Say "Update available: TextureCraft $($remote.Substring(0,7))." Yellow; return }

    $python = Get-Command python -ErrorAction SilentlyContinue
    if (-not $python) { throw 'Python 3 is needed to apply the SaltyShade overlay (https://www.python.org/downloads/).' }
    $cargo = Find-Cargo
    if (-not $cargo -and $InstallRust) { Install-Rust; $cargo = Find-Cargo }
    if (-not $cargo -and -not $NoBuild) {
        Say 'Rust is not installed. Run Update-SaltyShade.bat -InstallRust once (or install it from https://rustup.rs).' Red
        Popup "SaltyShade 3D needs the Rust toolchain to build updates.`n`nRun once:  Update-SaltyShade.bat -InstallRust"
        exit 2
    }
    Set-Content -Path $lock -Value $PID

    # 1. A pristine copy of the newest upstream. target/ (the build cache) is ignored by git, so it
    #    survives the reset and rebuilds stay incremental.
    if (-not (Test-Path (Join-Path $Upstream '.git'))) {
        Say "Downloading TextureCraft from $Repo ..." Cyan
        Invoke-Git clone -c core.longpaths=true --branch $Branch $Repo $Upstream
    }
    Push-Location $Upstream
    try {
        Invoke-Git config core.longpaths true
        Invoke-Git fetch --quiet origin $Branch
        Invoke-Git reset --quiet --hard "origin/$Branch"
        Invoke-Git clean --quiet -fd
        $sha = (Invoke-Git rev-parse HEAD).Trim()
        $subject = (Invoke-Git log -1 --format=%s).Trim()
    } finally { Pop-Location }
    Say "TextureCraft $($sha.Substring(0,7)): $subject" Cyan

    # 2. Your design on top.
    Say 'Applying the SaltyShade 3D overlay ...' Cyan
    & $python.Source -I (Join-Path $PSScriptRoot 'apply_overlay.py') $Upstream
    if ($LASTEXITCODE -ne 0) {
        throw 'This TextureCraft update changed code your design hooks into, so the overlay no longer fits (see the FAIL lines above). Your current build is unchanged.'
    }

    if ($NoBuild) { Say "Overlay applied in $Upstream (no build)." Green; return }

    # 3. Build.
    Say 'Building (the first build takes a while; later ones are incremental) ...' Cyan
    # A short build folder: Windows cannot run build scripts nested past 260 characters.
    $env:CARGO_TARGET_DIR = Join-Path $env:LOCALAPPDATA 'SaltyShade3D\target'
    # Without Visual Studio's C++ tools, the GNU toolchain needs MinGW's dlltool, as and gcc
    # (winget install BrechtSanders.WinLibs.POSIX.MSVCRT); Rust's own copies are the fallback.
    $mingw = Get-ChildItem -Directory -ErrorAction SilentlyContinue (Join-Path $env:LOCALAPPDATA 'Microsoft\WinGet\Packages\BrechtSanders.WinLibs*\mingw64\bin') | Select-Object -First 1
    $selfContained = Get-ChildItem -Directory -ErrorAction SilentlyContinue (Join-Path $env:USERPROFILE '.rustup\toolchains\*\lib\rustlib\x86_64-pc-windows-gnu\bin\self-contained') | Select-Object -First 1
    foreach ($d in @($selfContained, $mingw)) { if ($d) { $env:PATH = "$($d.FullName);$env:PATH" } }
    Push-Location $Upstream
    try {
        & $cargo build --release --locked -p texturecraft
        $built = $LASTEXITCODE -eq 0
    } finally { Pop-Location }
    if (-not $built) { throw 'The build failed (see the compiler output above). Your current build is unchanged.' }

    # 4. Swap it in; if SaltyShade 3D is open, the new build waits for the next start.
    $out = Join-Path $env:CARGO_TARGET_DIR 'release\texturecraft.exe'
    try { Copy-Item -Force $out $Exe; Say 'SaltyShade 3D updated.' Green }
    catch { Copy-Item -Force $out $NewExe; Say 'SaltyShade 3D updated; the new build starts next time you open it.' Green }
    Write-State 'upstream.sha' $sha
    Write-State 'overlay.sha' $overlayHash
} catch {
    Say $_.Exception.Message Red
    Popup ("SaltyShade 3D update failed:`n`n" + $_.Exception.Message + "`n`nLog: " + (Join-Path $State 'update.log'))
    exit 1
} finally {
    Remove-Item -ErrorAction SilentlyContinue $lock
}

if ($Launch -and -not (Get-Process -Name 'SaltyShade3D' -ErrorAction SilentlyContinue)) {
    Start-Process -FilePath $Exe -WorkingDirectory $Dist
}
