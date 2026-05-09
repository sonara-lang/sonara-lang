@echo off
setlocal EnableDelayedExpansion
chcp 65001 >nul 2>&1

echo.
echo  Sonara installer (Windows)
echo  ────────────────────────────────────────
echo.

:: ── download binary ────────────────────────────────────────────────────────────
set "BINARY_URL=https://github.com/sonara-lang/sonara-lang/raw/refs/heads/main/bin/sonara.exe"
set "BINARY_DIR=%USERPROFILE%\.sonara\bin"
set "BINARY=!BINARY_DIR!\sonara.exe"

if not exist "!BINARY_DIR!" mkdir "!BINARY_DIR!"

echo  [->] Downloading Sonara binary...
powershell -NoProfile -Command "Invoke-WebRequest -Uri '!BINARY_URL!' -OutFile '!BINARY!'" >nul 2>&1
if not exist "!BINARY!" (
    echo  [X] Failed to download binary.
    pause
    exit /b 1
)
echo  [OK] Binary downloaded

:: ── check for admin ───────────────────────────────────────────────────────────
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo  [!] Not running as Administrator.
    echo     Some installs may require elevation.
    echo     Re-run as Admin if installs fail.
    echo.
)


:: ── detect package manager ────────────────────────────────────────────────────
set "PKG_MGR="

where winget >nul 2>&1
if %errorlevel% == 0 ( set "PKG_MGR=winget" & goto :pkg_detected )

where choco >nul 2>&1
if %errorlevel% == 0 ( set "PKG_MGR=choco" & goto :pkg_detected )

where scoop >nul 2>&1
if %errorlevel% == 0 ( set "PKG_MGR=scoop" & goto :pkg_detected )

echo  [!] No package manager found. Audio dependencies must be installed manually.
set "PKG_MGR=none"

:pkg_detected

:: ── install audio engine ──────────────────────────────────────────────────────
where sclang >nul 2>&1
if %errorlevel% == 0 (
    echo  [OK] Audio engine ready
    goto :engine_done
)

echo  [->] Installing audio engine...

if "!PKG_MGR!" == "winget" (
    winget install --id SuperCollider.SuperCollider -e --silent --accept-package-agreements --accept-source-agreements >nul 2>&1
    if !errorlevel! == 0 ( echo  [OK] Audio engine installed & goto :engine_done )
)
if "!PKG_MGR!" == "choco" (
    choco install supercollider -y >nul 2>&1
    if !errorlevel! == 0 ( echo  [OK] Audio engine installed & goto :engine_done )
)
if "!PKG_MGR!" == "scoop" (
    scoop bucket add extras >nul 2>&1
    scoop install supercollider >nul 2>&1
    if !errorlevel! == 0 ( echo  [OK] Audio engine installed & goto :engine_done )
)

echo  [!] Audio engine could not be installed automatically.
echo     Please reinstall Sonara from the official package.

:engine_done

:: ── install ffmpeg ────────────────────────────────────────────────────────────
where ffmpeg >nul 2>&1
if %errorlevel% == 0 (
    echo  [OK] Audio converter ready
    goto :ffmpeg_done
)

echo  [->] Installing audio converter...

if "!PKG_MGR!" == "winget" (
    winget install --id Gyan.FFmpeg -e --silent --accept-package-agreements --accept-source-agreements >nul 2>&1
    if !errorlevel! == 0 ( echo  [OK] Audio converter installed & goto :ffmpeg_done )
)
if "!PKG_MGR!" == "choco" (
    choco install ffmpeg -y >nul 2>&1
    if !errorlevel! == 0 ( echo  [OK] Audio converter installed & goto :ffmpeg_done )
)
if "!PKG_MGR!" == "scoop" (
    scoop install ffmpeg >nul 2>&1
    if !errorlevel! == 0 ( echo  [OK] Audio converter installed & goto :ffmpeg_done )
)

echo  [!] Audio converter could not be installed automatically.
echo     Please reinstall Sonara from the official package.

:ffmpeg_done

:: ── install sonara binary ─────────────────────────────────────────────────────
echo.
set "INSTALL_DIR=%USERPROFILE%\.local\bin"
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

copy /y "!BINARY!" "%INSTALL_DIR%\sonara.exe" >nul
echo  [OK] Installed: %INSTALL_DIR%\sonara.exe

:: ── add to user PATH (persistent) ────────────────────────────────────────────
echo %PATH% | find /i "%INSTALL_DIR%" >nul 2>&1
if %errorlevel% neq 0 (
    powershell -NoProfile -Command ^
        "$cur = [System.Environment]::GetEnvironmentVariable('PATH','User'); ^
         if ($cur -notlike '*\.local\bin*') { ^
             [System.Environment]::SetEnvironmentVariable('PATH', $cur + ';%INSTALL_DIR%', 'User') ^
         }"
    set "PATH=%PATH%;%INSTALL_DIR%"
    echo  [OK] Added to PATH
) else (
    echo  [OK] PATH already configured
)

:: ── reload PATH from registry ─────────────────────────────────────────────────
for /f "tokens=2*" %%a in ('reg query "HKCU\Environment" /v PATH 2^>nul') do set "USER_PATH=%%b"
for /f "tokens=2*" %%a in ('reg query "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment" /v Path 2^>nul') do set "SYS_PATH=%%b"
set "PATH=%SYS_PATH%;%USER_PATH%"

:: ── done ──────────────────────────────────────────────────────────────────────
echo.
echo  Installation complete!
echo  ────────────────────────────────────────

where sonara >nul 2>&1
if %errorlevel% == 0 (
    echo  [OK] sonara is ready
) else (
    echo  [!] Open a new terminal to use sonara
    echo      Or run: %INSTALL_DIR%\sonara.exe
)

echo.
echo  Usage:
echo    sonara build examples\christmas.son --to=mp3
echo    sonara build examples\christmas.son --to=wav
echo.
pause
