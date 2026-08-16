# IPv7-SIMBI — Progreso y checklist

Este archivo resume lo que ya está implementado y lo que falta para tener una aplicación funcional en Windows y Linux. Está organizado por fases, de menor a mayor complejidad.

---

## Fase 0 — Núcleo del protocolo (COMPLETADO)

- [x] Encabezado compacto de 8 bytes (`z8`, `flex`, `route_index`, `pow_signature`, `anchor`, `length`, `sequence`).
- [x] Serialización/deserialización con trailer CRC-16-CCITT.
- [x] Prueba de trabajo (PoW) con `pow_signature XOR nonce AND máscara == 0`.
- [x] Prueba de servicio (PoS) con firma ligera `fnv1a_32(node + did + salt)`.
- [x] Router UDP real con `std::net::UdpSocket`.
- [x] Módulos separados: `packet.rs`, `router.rs`, `tun.rs`, `main.rs`.

## Fase 1 — Enrutamiento y robustez (COMPLETADO)

- [x] Modo Explorador con mapa de calor de prefijos.
- [x] Modo Tren Bala con rutas cristalizadas O(1).
- [x] Degradación elegante a Autocuración cuando falla un puerto.
- [x] Multicast con duplicación local.
- [x] Balanceo de carga probabilístico por latencia medida.
- [x] Exploración aleatoria cuando no hay calor.
- [x] Tabla de rutas dinámicas (`peers` estáticos + `dynamic` descubiertos).

## Fase 2 — Interactividad, medición y túneles (COMPLETADO)

- [x] Modo chat por stdin entre dos nodos.
- [x] Ping automático y medición de RTT por puerto.
- [x] Túnel UDP local (`IPV7_TUNNEL_BIND` / `IPV7_TUNNEL_PEER`).
- [x] Interfaz TUN real multiplataforma (`tun` crate).
- [x] Tracker/lookup de peers (`Z_REGISTER`, `Z_LOOKUP`, `Z_RESOLVE`).
- [x] Agregado automático de peers descubiertos al mapa de calor.
- [x] Heartbeats y limpieza de peers dinámicos inactivos.

## Fase 3 — Conectividad real entre PCs (COMPLETADO PARA MVP)

- [x] Cliente STUN mínimo para descubrir IP/puerto público propio.
- [x] Registro de dirección pública en el tracker (`Z_REGISTER` con payload).
- [x] Canal de control para envíos dirigidos desde `main.rs`.
- [x] NAT hole punching con `Z_HELLO` al recibir `Z_RESOLVE`.
- [ ] Relay/TURN fallback para NAT simétrico — *futuro*.
- [ ] Descubrimiento sin tracker central (DHT o mDNS en LAN) — *futuro*.

## Fase 4 — Seguridad (COMPLETADO PARA MVP)

- [x] Cifrado de payloads entre pares con ChaCha20-Poly1305 (clave precompartida `IPV7_PSK`).
- [x] Nonce implícito derivado del encabezado, DID destino y timestamp.
- [x] Anti-replay básico con cache de nonces (60s).
- [ ] Handshake Noise sin PSK — *futuro*.
- [ ] Autenticación de nodos por clave derivada del DID — *futuro*.
- [ ] Cifrado de cebolla opcional — *futuro*.

## Fase 5 — Usabilidad en Windows y Linux (COMPLETADO)

- [x] Archivo de configuración simple (`ipv7-simbi.conf`) en lugar de solo variables de entorno.
- [x] Asistente interactivo (`setup.bat` / `setup.sh`).
- [x] Inicio con doble click (`run-first.bat` / `run-first.sh`).
- [x] Verificación de privilegios de administrador (`run-first.bat`).
- [x] Logs a archivo (`IPV7_LOG`) con salida simultánea a consola.
- [x] Archivo de ejemplo (`ipv7-simbi.conf.example`).
- [x] Scripts de inicio para Windows (`start.bat`) y Linux (`start.sh`).
- [x] Instrucciones de instalación (`INSTALL.md`).
- [x] CLI básico (`--help`, `--config`, `--send`, `--demo`).
- [x] Archivo de servicio `systemd` (`ipv7-simbi.service`).

## Fase 6 — Integración con el sistema operativo (CERRADO PARA MVP)

- [x] Scripts de rutas para Windows (`routes.ps1`) y Linux (`routes.sh`).
- [x] Adaptador TUN creado y verificado en Windows con Wintun.
- [x] Tráfico IP real extremo a extremo verificado entre dos nodos (`test-two-nodes.sh`).
- [x] Salida a internet a través del túnel con un nodo gateway (`gateway.sh`, `gateway.ps1`, verificado con `test-gateway.sh`).
- [ ] Configuración automática de rutas dentro del binario — *futuro*.
- [ ] DNS interno para resolver nombres a DIDs — *futuro*.
- [ ] MTU discovery real por ruta — *futuro*.
- [ ] IPv6 sobre IPv7-SIMBI — *futuro*.

## Fase 7 — Escalabilidad y producción (FUTURO)

- [ ] Forwarding zero-copy con eBPF/XDP o io_uring.
- [ ] Co-procesador en Verilog/FPGA.
- [ ] Hub gateway para dispositivos del hogar sin IPv7.
- [ ] Redes miceliales locales sin internet.

---

## Estado resumido

**VPN terminada y probada con doble click.**

IPv7-SIMBI ya es una **VPN P2P privada y cifrada** entre nodos Windows y Linux. El usuario la descarga, configura con doble click en `setup.bat`/`setup.sh`, e inicia con `run-first.bat`/`run-first.sh` (requiere administrador en Windows).

Pruebas realizadas:
- Build release limpio y sin warnings (`cargo build --release`), Linux y Windows (`x86_64-pc-windows-gnu`).
- `ipv7_simbi.exe --help` y `--demo` responden.
- `run-first.bat` con privilegios de administrador crea el adaptador `ipv7` (status `Up`).
- `wintun.dll` extraída de `wintun-0.14.1.zip` y desplegada junto al ejecutable.
- `sudo ./test-two-nodes.sh`: ping ICMP entre `10.0.0.1` y `10.0.0.2` sobre el túnel, 0% de pérdida.
- `sudo ./test-gateway.sh`: `ping 8.8.8.8` y `HTTP 200` de example.com desde el nodo A saliendo por el NAT del nodo B.

Documentación:
- `VPN.md` — definición de la VPN y mejoras sobre IPv4/IPv6.
- `MVP.md` — resumen final del MVP.
- `INSTALL.md` — instalación rápida.
- `DISTRIBUTE.md` — empaquetado para usuarios finales.
- `SESSION.md` — resumen de continuidad para la siguiente sesión.

Las funciones restantes (relay, Noise, autenticación DID, DNS, eBPF/FPGA) quedan como roadmap futuro, fuera del MVP.
