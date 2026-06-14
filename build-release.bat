@echo off
setlocal
chcp 65001 >nul

set "ROOT_DIR=%~dp0"
cd /d "%ROOT_DIR%"

echo [PGRN] Building desktop release...
call npm run tauri build
if errorlevel 1 (
    echo.
    echo [PGRN] Release build failed.
    pause
    exit /b 1
)

exit /b 0
