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

## Panel de estado

Con el nodo corriendo, abre <http://127.0.0.1:7777> para ver pares, RTT,
contadores de paquetes y las últimas líneas del log. Sólo escucha en loopback.
Se apaga con `IPV7_UI = "0"` y se cambia de puerto con `IPV7_UI = "8080"`.

Para depurar, `IPV7_VERBOSE = "1"` registra cada paquete. Cuesta rendimiento:
déjalo apagado en uso normal.

## Navegar por internet a través del túnel

Un nodo actúa de gateway: reenvía el tráfico del túnel a su conexión a internet con NAT.

En el nodo gateway:

```bash
sudo ./gateway.sh          # Linux (iptables + ip_forward);  --off para revertir
```

```powershell
.\gateway.ps1              # Windows, como administrador (ICS);  -Off para revertir
```

En el otro nodo, manda todo el tráfico por el túnel:

```bash
sudo ip route replace default dev ipv7                        # Linux
route add 0.0.0.0 mask 0.0.0.0 10.0.0.1 metric 1              # Windows (como admin)
```

## Pruebas

Con root en Linux, sin tocar la red del equipo (usa network namespaces):

```bash
sudo ./test-two-nodes.sh   # ping entre los dos extremos del túnel
sudo ./test-gateway.sh     # navegación real a internet a través del túnel
```

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
