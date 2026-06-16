@echo off
cd /d "%~dp0"
set LOG=build.log
echo === Vayou build ===
echo Log file: %CD%\%LOG%
echo.
powershell -NoProfile -ExecutionPolicy Bypass -Command "& { pnpm tauri build 2>&1 | Tee-Object -FilePath '%LOG%'; exit $LASTEXITCODE }"
set ERR=%ERRORLEVEL%
echo.
echo ===================================
echo Exit code: %ERR%
echo Full log: %CD%\%LOG%
echo ===================================
pause
