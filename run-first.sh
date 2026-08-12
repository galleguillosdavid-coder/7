#!/bin/bash
set -e

if [ ! -f ipv7-simbi.conf ]; then
    echo "No se encontro ipv7-simbi.conf. Ejecutando asistente de configuracion..."
    ./setup.sh
fi

if [ ! -f ipv7-simbi.conf ]; then
    echo "No se pudo crear ipv7-simbi.conf. Saliendo."
    exit 1
fi

echo "Iniciando IPv7-SIMBI VPN..."
if [ "$EUID" -ne 0 ]; then
    sudo ./ipv7_simbi
else
    ./ipv7_simbi
fi
