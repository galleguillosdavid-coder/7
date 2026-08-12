#!/bin/bash
set -e

if [ ! -f ipv7-simbi.conf ]; then
    echo "[ERROR] No se encontro ipv7-simbi.conf"
    echo "Copia ipv7-simbi.conf.example a ipv7-simbi.conf y editalo."
    exit 1
fi

echo "Iniciando IPv7-SIMBI (se requieren privilegios para TUN)..."
if [ "$EUID" -ne 0 ]; then
    sudo ./ipv7_simbi
else
    ./ipv7_simbi
fi
