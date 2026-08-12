#!/bin/bash
# Configura la ruta hacia la subred IPv7-SIMBI en Linux
# Ejecutar como root o con sudo

SUBNET="${1:-10.0.0.0/24}"
GATEWAY="${2:-10.0.0.1}"
DEV="${3:-ipv7}"

echo "Agregando ruta $SUBNET via $GATEWAY dev $DEV"
ip route add "$SUBNET" via "$GATEWAY" dev "$DEV" || true
