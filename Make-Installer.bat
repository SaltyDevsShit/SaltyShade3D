@echo off
rem Packages the built SaltyShade 3D into dist\SaltyShade3D-<version>-Setup.exe, the file to send people.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0tools\make_installer.ps1" %*
pause
