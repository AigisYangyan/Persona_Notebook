@echo off
setlocal

echo [PGRN] building release bundles...
call npm run tauri build

if exist "%~dp0pgrn.exe" (
    del /f /q "%~dp0pgrn.exe"
)

echo.
echo [PGRN] build complete.
echo [PGRN] release exe: src-tauri\target\release\pgrn.exe
echo [PGRN] installers: src-tauri\target\release\bundle\
pause
