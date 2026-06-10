@echo off
setlocal

set "ROOT_DIR=%~dp0"
set "RELEASE_DIR=%ROOT_DIR%src-tauri\target\release"
set "RELEASE_EXE=%RELEASE_DIR%\pgrn.exe"

if not exist "%RELEASE_EXE%" (
    echo [PGRN] release executable not found:
    echo %RELEASE_EXE%
    echo [PGRN] run build-release.bat first.
    pause
    exit /b 1
)

pushd "%RELEASE_DIR%"
"%RELEASE_EXE%"
popd

echo.
echo [PGRN] process exited.
pause
