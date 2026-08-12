@echo off
setlocal

net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [AVISO] Este script requiere privilegios de administrador para crear el adaptador TUN.
    echo Clic derecho en run-first.bat ^> Ejecutar como administrador.
    pause
    exit /b 1
)

if not exist wintun.dll (
    echo [ERROR] No se encontro wintun.dll.
    echo Descargalo desde https://www.wintun.net y colocalo junto a este archivo.
    pause
    exit /b 1
)

if not exist ipv7-simbi.conf (
    echo No se encontro ipv7-simbi.conf. Ejecutando asistente de configuracion...
    call setup.bat
)

if not exist ipv7-simbi.conf (
    echo No se pudo crear ipv7-simbi.conf. Saliendo.
    pause
    exit /b 1
)

echo Iniciando IPv7-SIMBI VPN...
ipv7_simbi.exe
