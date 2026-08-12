#!/bin/bash
set -e

if [ -f ipv7-simbi.conf ]; then
    read -p "Ya existe ipv7-simbi.conf. Sobrescribir? (s/N): " CONFIRM
    if [ "$CONFIRM" != "s" ] && [ "$CONFIRM" != "S" ]; then
        echo Cancelado.
        exit 0
    fi
fi

echo "--- Configuracion de IPv7-SIMBI VPN ---"
read -p "ID de este nodo (ej A): " NODE_ID
read -p "Direccion UDP local (ej 0.0.0.0:9001): " BIND
read -p "Clave precompartida (todos los nodos deben usar la misma): " PSK
read -p "DID del otro nodo (ej B): " TUNNEL_DST
read -p "Direccion del otro nodo host:puerto (ej 192.168.1.50:9002): " PEER_ADDR
read -p "Nombre del adaptador TUN (ej ipv7): " TUNNEL_BIND
read -p "IP del tunel para ESTE nodo (ej 10.0.0.1): " TUN_ADDR
read -p "IP del tunel del OTRO nodo (ej 10.0.0.2): " TUN_DEST
read -p "Direccion del tracker (dejar vacio si no hay): " TRACKER_ADDR
read -p "Servidor STUN (dejar vacio si no hay): " STUN_SERVER

TUNNEL_BIND=${TUNNEL_BIND:-ipv7}
TUN_ADDR=${TUN_ADDR:-10.0.0.1}
TUN_DEST=${TUN_DEST:-10.0.0.2}

cat > ipv7-simbi.conf <<EOF
# IPv7-SIMBI configuracion
IPV7_NODE_ID = "$NODE_ID"
IPV7_BIND = "$BIND"
IPV7_PSK = "$PSK"
IPV7_TUNNEL_DST = "$TUNNEL_DST"
IPV7_TUNNEL_BIND = "$TUNNEL_BIND"
EOF

if [ -n "$PEER_ADDR" ]; then
    echo "IPV7_PEERS = \"2:$PEER_ADDR\"" >> ipv7-simbi.conf
    echo "IPV7_HEAT = \"$TUNNEL_DST:2\"" >> ipv7-simbi.conf
fi

if [ -n "$TRACKER_ADDR" ]; then
    echo "IPV7_TRACKER_ADDR = \"$TRACKER_ADDR\"" >> ipv7-simbi.conf
fi

if [ -n "$STUN_SERVER" ]; then
    echo "IPV7_STUN_SERVER = \"$STUN_SERVER\"" >> ipv7-simbi.conf
fi

cat >> ipv7-simbi.conf <<EOF
IPV7_TUN_DEVICE = "$TUNNEL_BIND"
IPV7_TUN_ADDR = "$TUN_ADDR"
IPV7_TUN_NETMASK = "255.255.255.0"
IPV7_TUN_DEST = "$TUN_DEST"
IPV7_TUN_MTU = "1400"
EOF

echo
echo "Configuracion guardada en ipv7-simbi.conf."
