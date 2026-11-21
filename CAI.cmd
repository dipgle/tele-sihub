@echo off
rem Bam doi file nay (hoac go CAI.cmd trong thu muc huba-windows-*) la cai xong.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0cai.ps1" %*
echo.
echo Ma thoat: %ERRORLEVEL%
pause
