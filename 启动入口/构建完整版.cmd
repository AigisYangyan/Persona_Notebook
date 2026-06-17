@echo off
setlocal
chcp 65001 >nul

set "ROOT_DIR=%~dp0.."
set "ENTRY_DIR=%~dp0"
cd /d "%ROOT_DIR%"

echo [PGRN] Building latest desktop release...
call npm run tauri build
if errorlevel 1 (
    echo.
    echo [PGRN] Build failed.
    if /I "%~1"=="--no-pause" exit /b 1
    pause
    exit /b 1
)

echo [PGRN] Syncing installers to release folder...
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%ENTRY_DIR%sync-release.ps1"
if errorlevel 1 (
    echo.
    echo [PGRN] Build finished, but installer sync failed.
    if /I "%~1"=="--no-pause" exit /b 1
    pause
    exit /b 1
)

echo.
echo [PGRN] Build complete.
echo [PGRN] Release exe: src-tauri\target\release\pgrn.exe
echo [PGRN] Installers synced to launch-entry release folder.
if /I "%~1"=="--no-pause" exit /b 0
pause
exit /b 0
