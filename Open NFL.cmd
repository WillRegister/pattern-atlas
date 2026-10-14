@echo off
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\open-nfl.ps1"
if errorlevel 1 pause
