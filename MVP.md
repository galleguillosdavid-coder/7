# IPv7-SIMBI — VPN lista

Este documento declara que el **Producto Mínimo Viable (MVP)** de IPv7-SIMBI **VPN** está terminado.

## Qué es

IPv7-SIMBI es una **VPN P2P privada y cifrada** con interfaz TUN. Permite conectar dos o más nodos Windows/Linux como si estuvieran en la misma red local, usando un protocolo propio que supera limitaciones de IPv4/IPv6 tradicionales. Ver `VPN.md` para el análisis completo.

## Funcionalidades del MVP

- **VPN real** con adaptador TUN en Windows y Linux.
- **Protocolo IPv7-SIMBI** propio con encabezado compacto y 11 modos.
- **Enrutamiento** por calor de prefijos, rutas cristalizadas, exploración y degradación.
- **Balanceo de carga** probabilístico por latencia medida.
- **Cifrado** de payloads con ChaCha20-Poly1305 mediante PSK.
- **Anti-replay** con cache de nonces.
- **Descubrimiento de peers** vía tracker y STUN.
- **NAT hole punching** con saludos automáticos.
- **Heartbeats** y limpieza de peers muertos.
- **Configuración** por archivo (`ipv7-simbi.conf`) con asistente interactivo (`setup.bat`/`setup.sh`).
- **Logs** simultáneos a consola y archivo.
- **CLI básica** (`--help`, `--config`, `--send`, `--demo`).
- **Inicio con doble click** en Windows.
- **Scripts** de inicio y rutas del SO para Windows y Linux.
- **Servicio systemd** listo.

## Instalación rápida

### Windows

1. Descomprime el paquete.
2. Doble click en `setup.bat`.
3. Doble click en `run-first.bat`.

### Linux

1. Compila: `cargo build --release`.
2. `./setup.sh`
3. `./run-first.sh`

Ver `INSTALL.md` y `VPN.md` para más detalles.

## Distribución

Ver `DISTRIBUTE.md`.

## Estado

**VPN MVP terminada.** Lista para pruebas entre dos PCs con IP pública o en LAN con tracker.
