@echo off
setlocal enabledelayedexpansion

echo ========================================================
echo   CS2 Optimizer (Rust) - One-Click Build Script
echo ========================================================
echo.

where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo [!] Cargo / Rust is not found in PATH!
    echo [!] Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

echo [*] Compiling standalone high-performance release binary...
cargo build --release

if %ERRORLEVEL% equ 0 (
    echo.
    echo ========================================================
    echo   BUILD SUCCESSFUL!
    echo   Artifact: target\release\cs2-optimizer.exe
    echo   (Single lightweight executable, zero DLLs needed)
    echo ========================================================
) else (
    echo.
    echo [!] Cargo release build failed!
)

pause
