# Instalación de IPv7-SIMBI VPN

IPv7-SIMBI es una VPN P2P cifrada. Consulta `VPN.md` para entender por qué es diferente a IPv4/IPv6.

## Windows — doble click

1. Descarga el paquete con `ipv7_simbi.exe`, `wintun.dll`, `setup.bat` y `run-first.bat`.
2. Haz **doble click en `setup.bat`** y responde las preguntas. Esto crea `ipv7-simbi.conf`.
3. Haz **doble click en `run-first.bat`** para iniciar la VPN.
4. Para enrutar tráfico por el túnel, abre PowerShell como administrador y corre `.\routes.ps1`.

> Nota: `wintun.dll` se descarga desde <https://www.wintun.net> si no viene en el paquete.

## Linux

1. Compila o descarga el binario `ipv7_simbi`.
2. Ejecuta `./setup.sh` y responde las preguntas. Esto crea `ipv7-simbi.conf`.
3. Ejecuta `./run-first.sh` (pide sudo para el dispositivo TUN).
4. Para enrutar tráfico por el túnel, corre `sudo ./routes.sh`.

## systemd

Copia `ipv7-simbi.service` a `/etc/systemd/system/`, ajusta `WorkingDirectory` y `ExecStart`, y corre:

```bash
systemctl enable --now ipv7-simbi
```

## CLI

```
ipv7_simbi --help
ipv7_simbi --config otra.conf
ipv7_simbi --demo
```
