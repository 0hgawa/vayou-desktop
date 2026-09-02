@echo off
cd /d "%~dp0"
set LOG=dev.log

rem Verbose app logging by default: the dev script exists to diagnose, and at
rem `info` the log shows startup and little else. `debug` adds every command
rem sent to mpv, which is what makes a stuck playback or translation readable.
rem Override for a quieter run:  set RUST_LOG=vayou=info  &&  dev.bat
if not defined RUST_LOG set RUST_LOG=vayou=debug

echo === Vayou dev ===
echo Log file:  %CD%\%LOG%
echo RUST_LOG:  %RUST_LOG%
echo.
powershell -NoProfile -ExecutionPolicy Bypass -Command "& { pnpm tauri dev 2>&1 | Tee-Object -FilePath '%LOG%'; exit $LASTEXITCODE }"
set ERR=%ERRORLEVEL%
echo.
echo ===================================
echo Exit code: %ERR%
echo Full log: %CD%\%LOG%
echo ===================================
pause
