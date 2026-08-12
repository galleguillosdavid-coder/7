# IPv7-SIMBI VPN

IPv7-SIMBI es una **VPN P2P moderna** que no imita a IPv4 ni IPv6: redefine el concepto de red para la era del borde descentralizado.

## Qué la hace una VPN

- **Túnel virtual (TUN/TAP)**: crea una interfaz de red en el sistema operativo. Todo el tráfico IP que pasa por ella se encapsula, cifra y enruta a través de la overlay.
- **Cifrado por defecto**: cada payload viaja con ChaCha20-Poly1305. Sin la PSK correcta no se puede leer ni inyectar tráfico.
- **Punto a punto sin IP fija**: los nodos se encuentran por DID a través de trackers, STUN y hole punching.
- **Tráfico IP real**: cualquier aplicación (navegador, SSH, juego) funciona sin saber que usa IPv7-SIMBI.

## Mejoras sobre IPv4/IPv6 tradicional

| Aspecto | IPv4/IPv6 tradicional | IPv7-SIMBI |
|---|---|---|
| **Direccionamiento** | IP numérica, difícil de recordar, limitada por NAT | DID alfanumérico (ej. `A`, `B`, `Casa`) con resolución por tracker |
| **NAT** | Complejo, requiere routers, port forwarding, CGNAT | Hole punching y overlay por UDP; el propio protocolo atraviesa NAT |
| **Enrutamiento** | Tablas estáticas, BGP, jerarquías | Mapa de calor por prefijos, rutas cristalizadas (O(1)), exploración gravitacional y degradación automática |
| **Movilidad** | IP cambia al cambiar de red; rompe conexiones | DID se mantiene; el nodo se redescubre por tracker o STUN |
| **Cifrado** | Opcional (IPsec, WireGuard por separado) | Integrado en el protocolo con ChaCha20-Poly1305 |
| **Resiliencia** | Corte de enlace = corte de servicio | Modo Autocuración, heartbeats, balanceo por latencia y remolino evitado |
| **Escalabilidad inicial** | Requiere direcciones, ASN, coordinación central | Overlay auto-configurable; funciona en LAN, Internet con tracker o DHT |
| **Overhead** | Encabezados grandes (20-40 bytes IPv4/IPv6) | Encabezado compacto de 8 bytes + trailer CRC-16 |

## Arquitectura en una frase

> IPv7-SIMBI es una **red de identidades (DIDs) sobre UDP**, donde cada paquete sabe cómo navegar por el espacio de nombres y autocurarse si un camino falla.

## Uso inmediato

### Windows

1. Descarga y descomprime el paquete.
2. Asegúrate de que `wintun.dll` esté junto a `ipv7_simbi.exe`.
3. Haz doble click en `setup.bat` para crear `ipv7-simbi.conf`.
4. Haz doble click en `run-first.bat`.

### Linux

1. Descarga y descomprime el paquete.
2. `./setup.sh`
3. `./run-first.sh`

## Consejos

- Usa la misma PSK en todos los nodos.
- Si estás en LAN, usa direcciones locales como `0.0.0.0:9001`.
- Si estás en Internet, define un `IPV7_TRACKER_ADDR` y `IPV7_STUN_SERVER`.
- Para enrutar todo el tráfico por el túnel, ejecuta `routes.ps1` (Windows admin) o `routes.sh` (Linux root).
