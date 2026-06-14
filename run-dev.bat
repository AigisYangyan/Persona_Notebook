@echo off
setlocal
chcp 65001 >nul

set "ROOT_DIR=%~dp0"
cd /d "%ROOT_DIR%"

echo [PGRN] Starting desktop app in development mode...
call npm run tauri dev
if errorlevel 1 (
    echo.
    echo [PGRN] Development launch failed.
    pause
    exit /b 1
)

exit /b 0
