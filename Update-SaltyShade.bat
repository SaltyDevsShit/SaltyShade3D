@echo off
rem Update SaltyShade 3D now. Extra options:
rem   -Force            rebuild even if nothing changed
rem   -CheckOnly        only say whether an update is available
rem   -InstallRust      install the Rust toolchain first (needed once)
rem   -InstallSchedule  also check for updates every day (Task Scheduler)
rem   -RemoveSchedule   stop the daily check
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0tools\updater.ps1" %*
pause
