@echo off
setlocal
chcp 65001 >nul

set "ROOT_DIR=%~dp0.."
cd /d "%ROOT_DIR%"

echo [PGRN] Building latest desktop release...
call npm run tauri build
if errorlevel 1 (
    echo.
    echo [PGRN] Build failed.
    pause
    exit /b 1
)

echo.
echo [PGRN] Build complete.
echo [PGRN] Release exe: src-tauri\target\release\pgrn.exe
echo [PGRN] Installers: src-tauri\target\release\bundle\
pause
exit /b 0
