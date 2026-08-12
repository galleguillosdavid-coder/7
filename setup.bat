@echo off
setlocal EnableDelayedExpansion

if exist ipv7-simbi.conf (
    echo Ya existe ipv7-simbi.conf. Sobrescribir? (s/N)
    set /p CONFIRM=
    if /I not "!CONFIRM!"=="s" (
        echo Cancelado.
        exit /b 0
    )
)

echo --- Configuracion de IPv7-SIMBI VPN ---
set /p NODE_ID=ID de este nodo (ej A): 
set /p BIND=Direccion UDP local (ej 0.0.0.0:9001): 
set /p PSK=Clave precompartida (todos los nodos deben usar la misma): 
set /p TUNNEL_DST=DID del otro nodo (ej B): 
set /p TUNNEL_BIND=Nombre del adaptador TUN (ej ipv7): 
set /p TRACKER_ADDR=Direccion del tracker (dejar vacio si no hay): 
set /p STUN_SERVER=Servidor STUN (dejar vacio si no hay): 

(
echo # IPv7-SIMBI configuracion
echo IPV7_NODE_ID = "%NODE_ID%"
echo IPV7_BIND = "%BIND%"
echo IPV7_PSK = "%PSK%"
echo IPV7_TUNNEL_DST = "%TUNNEL_DST%"
echo IPV7_TUNNEL_BIND = "%TUNNEL_BIND%"
) > ipv7-simbi.conf

if not "%TRACKER_ADDR%"=="" (
    echo IPV7_TRACKER_ADDR = "%TRACKER_ADDR%" >> ipv7-simbi.conf
)

if not "%STUN_SERVER%"=="" (
    echo IPV7_STUN_SERVER = "%STUN_SERVER%" >> ipv7-simbi.conf
)

(
echo IPV7_TUN_DEVICE = "ipv7"
echo IPV7_TUN_ADDR = "10.0.0.2"
echo IPV7_TUN_NETMASK = "255.255.255.0"
echo IPV7_TUN_DEST = "10.0.0.1"
echo IPV7_TUN_MTU = "1400"
) >> ipv7-simbi.conf

echo.
echo Configuracion guardada en ipv7-simbi.conf.
pause
