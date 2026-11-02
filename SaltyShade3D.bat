@echo off
rem Starts SaltyShade 3D, then checks GitHub for a newer TextureCraft and rebuilds it with your
rem SaltyShade design in the background (the new build is used the next time you start).
start "SaltyShade 3D updater" /min powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0tools\updater.ps1" -Launch
