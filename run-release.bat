@echo off
cd /d "%~dp0"
set EXE=src-tauri\target\release\vayou-desktop.exe
set LOG=release.log
if not exist "%EXE%" (
  echo Release build not found at: %EXE%
  echo Run build.bat first.
  pause
  exit /b 1
)
echo === Running: %EXE% ===
echo Log file: %CD%\%LOG%
echo.
"%EXE%" > "%LOG%" 2>&1
set ERR=%ERRORLEVEL%
echo.
echo ===================================
echo Exit code: %ERR%
echo Full log: %CD%\%LOG%
echo ===================================
pause
