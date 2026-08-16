#!/bin/bash
# Convierte este nodo en la salida a internet del tunel IPv7-SIMBI.
# El otro extremo enruta todo su trafico por el tunel y sale con la IP
# publica de este equipo.
#
#   sudo ./gateway.sh [interfaz-uplink] [subred-tunel] [interfaz-tun]
#
# Para deshacerlo:  sudo ./gateway.sh --off
set -e

UPLINK="${1:-$(ip route show default | awk '{print $5; exit}')}"
SUBNET="${2:-10.0.0.0/24}"
DEV="${3:-ipv7}"

if [ "$1" = "--off" ]; then
    UPLINK="$(ip route show default | awk '{print $5; exit}')"
    iptables -t nat -D POSTROUTING -s "$SUBNET" -o "$UPLINK" -j MASQUERADE 2>/dev/null || true
    iptables -D FORWARD -i "$DEV" -o "$UPLINK" -j ACCEPT 2>/dev/null || true
    iptables -D FORWARD -i "$UPLINK" -o "$DEV" -m state --state RELATED,ESTABLISHED -j ACCEPT 2>/dev/null || true
    echo "Gateway IPv7-SIMBI desactivado."
    exit 0
fi

if [ "$(id -u)" -ne 0 ]; then
    echo "Ejecutar como root: sudo ./gateway.sh"
    exit 1
fi

echo "Uplink: $UPLINK   Tunel: $DEV   Subred: $SUBNET"
sysctl -w net.ipv4.ip_forward=1
iptables -t nat -C POSTROUTING -s "$SUBNET" -o "$UPLINK" -j MASQUERADE 2>/dev/null ||
    iptables -t nat -A POSTROUTING -s "$SUBNET" -o "$UPLINK" -j MASQUERADE
iptables -C FORWARD -i "$DEV" -o "$UPLINK" -j ACCEPT 2>/dev/null ||
    iptables -A FORWARD -i "$DEV" -o "$UPLINK" -j ACCEPT
iptables -C FORWARD -i "$UPLINK" -o "$DEV" -m state --state RELATED,ESTABLISHED -j ACCEPT 2>/dev/null ||
    iptables -A FORWARD -i "$UPLINK" -o "$DEV" -m state --state RELATED,ESTABLISHED -j ACCEPT

echo "Listo. En el otro nodo:  sudo ip route replace default dev $DEV"
