@echo off
setlocal
chcp 65001 >nul

set "ENTRY_DIR=%~dp0"
set "ROOT_DIR=%ENTRY_DIR%.."
cd /d "%ROOT_DIR%"

"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%ENTRY_DIR%launch-full.ps1"
if errorlevel 1 (
    echo.
    echo [PGRN] Launch failed. The window will stay open so you can read the error.
    pause
    exit /b 1
)

exit /b 0
