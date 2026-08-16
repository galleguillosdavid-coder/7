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
set /p PEER_ADDR=Direccion del otro nodo host:puerto (ej 192.168.1.50:9002): 
set /p TUNNEL_BIND=Nombre del adaptador TUN (ej ipv7): 
set /p TUN_ADDR=IP del tunel para ESTE nodo (ej 10.0.0.1): 
set /p TUN_DEST=IP del tunel del OTRO nodo (ej 10.0.0.2): 
set /p TRACKER_ADDR=Direccion del tracker (dejar vacio si no hay): 
set /p STUN_SERVER=Servidor STUN (dejar vacio si no hay): 

if "%TUNNEL_BIND%"=="" set TUNNEL_BIND=ipv7
if "%TUN_ADDR%"=="" set TUN_ADDR=10.0.0.1
if "%TUN_DEST%"=="" set TUN_DEST=10.0.0.2

(
echo # IPv7-SIMBI configuracion
echo IPV7_NODE_ID = "%NODE_ID%"
echo IPV7_BIND = "%BIND%"
echo IPV7_PSK = "%PSK%"
echo IPV7_TUNNEL_DST = "%TUNNEL_DST%"
echo IPV7_TUNNEL_BIND = "%TUNNEL_BIND%"
) > ipv7-simbi.conf

if not "%PEER_ADDR%"=="" (
    echo IPV7_PEERS = "2:%PEER_ADDR%" >> ipv7-simbi.conf
    echo IPV7_HEAT = "%TUNNEL_DST%:2" >> ipv7-simbi.conf
)

if not "%TRACKER_ADDR%"=="" (
    echo IPV7_TRACKER_ADDR = "%TRACKER_ADDR%" >> ipv7-simbi.conf
)

if not "%STUN_SERVER%"=="" (
    echo IPV7_STUN_SERVER = "%STUN_SERVER%" >> ipv7-simbi.conf
)

(
echo IPV7_TUN_DEVICE = "%TUNNEL_BIND%"
echo IPV7_TUN_ADDR = "%TUN_ADDR%"
echo IPV7_TUN_NETMASK = "255.255.255.0"
echo IPV7_TUN_DEST = "%TUN_DEST%"
echo IPV7_TUN_MTU = "1400"
) >> ipv7-simbi.conf

echo.
echo Configuracion guardada en ipv7-simbi.conf.
pause
