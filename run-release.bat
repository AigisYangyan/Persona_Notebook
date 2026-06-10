@echo off
setlocal

set "ROOT_DIR=%~dp0"
set "RELEASE_DIR=%ROOT_DIR%src-tauri\target\release"
set "RELEASE_EXE=%RELEASE_DIR%\pgrn.exe"
set "SETUP_EXE=%ROOT_DIR%src-tauri\target\release\bundle\nsis\Personal Growth RPG Notebook_0.1.0_x64-setup.exe"

if exist "%RELEASE_EXE%" (
    start "" /D "%RELEASE_DIR%" "pgrn.exe"
    exit /b 0
)

echo [PGRN] release executable not found:
echo %RELEASE_EXE%
if exist "%SETUP_EXE%" (
    echo [PGRN] installer is available here:
    echo %SETUP_EXE%
) else (
    echo [PGRN] run build-release.bat first.
)
pause
