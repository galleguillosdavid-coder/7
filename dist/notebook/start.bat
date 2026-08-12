@echo off
setlocal

if not exist ipv7-simbi.conf (
    echo [ERROR] No se encontro ipv7-simbi.conf
    echo Copia ipv7-simbi.conf.example a ipv7-simbi.conf y editalo.
    pause
    exit /b 1
)

if not exist wintun.dll (
    echo [ADVERTENCIA] No se encontro wintun.dll. El modo TUN no funcionara.
    echo Descargalo desde https://www.wintun.net y colocalo junto a este .bat
    pause
    exit /b 1
)

echo Iniciando IPv7-SIMBI...
echo Se requieren privilegios de administrador para crear el adaptador TUN.
ipv7_simbi.exe
