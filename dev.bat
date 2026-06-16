@echo off
cd /d "%~dp0"
set LOG=dev.log
echo === Vayou dev ===
echo Log file: %CD%\%LOG%
echo.
powershell -NoProfile -ExecutionPolicy Bypass -Command "& { pnpm tauri dev 2>&1 | Tee-Object -FilePath '%LOG%'; exit $LASTEXITCODE }"
set ERR=%ERRORLEVEL%
echo.
echo ===================================
echo Exit code: %ERR%
echo Full log: %CD%\%LOG%
echo ===================================
pause
