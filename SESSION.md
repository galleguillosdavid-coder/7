# Sesión de continuidad — IPv7-SIMBI VPN

## Contexto

Fecha: 2026-08-12
Objetivo: dejar la VPN funcionando de verdad — tráfico IP real entre dos nodos y
navegación a internet a través del túnel.

## Estado actual

- **Túnel verificado extremo a extremo.** `sudo ./test-two-nodes.sh` levanta dos nodos
  en network namespaces separados y hace ping entre `10.0.0.1` y `10.0.0.2` con 0% de
  pérdida.
- **Navegación verificada.** `sudo ./test-gateway.sh` enruta todo el tráfico del nodo A
  por el túnel; el nodo B hace NAT hacia internet. `ping 8.8.8.8` responde y
  `curl http://example.com` devuelve `HTTP 200`.
- Build limpio y sin warnings en Linux y en Windows (`--target x86_64-pc-windows-gnu`).
  El `ipv7_simbi.exe` del repositorio es el binario nuevo.

## Bugs corregidos en esta sesión

1. **El TUN nunca enviaba nada con `IPV7_BIND = "0.0.0.0:puerto"`.** `tun.rs` descartaba
   la dirección del router cuando era no especificada. Ahora usa `127.0.0.1:<puerto>`.
2. **El anti-replay descartaba tráfico legítimo.** El identificador se derivaba solo del
   encabezado, así que dos paquetes IP distintos del mismo tamaño dentro de 60 s se
   consideraban un replay: con eso una VPN real no puede funcionar. Ahora el
   identificador es el nonce único de cada paquete (o un hash de encabezado + payload
   cuando no hay cifrado), y la caché se poda al superar 8192 entradas.
3. **Nonce de cifrado reutilizado.** `build_nonce` derivaba el nonce del encabezado y de
   un timestamp truncado, así que se repetía en todos los paquetes (fatal en
   ChaCha20-Poly1305) y además cambiaba cada ~4,6 h rompiendo el descifrado. Ahora cada
   paquete lleva un nonce propio de 12 bytes (semilla de proceso + contador) delante del
   texto cifrado.
4. **Sin entrega local.** Un paquete cuyo `did_dst` era el propio nodo solo se entregaba
   si el mapa de calor tenía una entrada artificial hacia el puerto 0. Ahora se entrega
   siempre que el DID coincide.
5. **Nombres DNS en la configuración.** `IPV7_STUN_SERVER = "stun.l.google.com:19302"` se
   ignoraba en silencio porque se parseaba como `SocketAddr`. `IPV7_PEERS`,
   `IPV7_TRACKER_ADDR`, `IPV7_TUNNEL_PEER` y el STUN ahora resuelven nombres.
6. `src/config.rs`: eliminada la función `var` sin uso (el warning de la sesión anterior).
7. `src/tun.rs`: arreglado el cierre de `platform_config` que ya no compilaba con la
   versión actual del crate `tun`.

## Novedades

- `gateway.sh` — activa forwarding + NAT en Linux para que el otro extremo navegue.
- `gateway.ps1` — equivalente en Windows vía ICS (COM `HNetCfg.HNetShare`), con
  `-Off` para revertir.
- `test-two-nodes.sh` y `test-gateway.sh` — pruebas reproducibles con netns.
- `setup.sh` / `setup.bat` ahora preguntan por la dirección del otro nodo y las IPs del
  túnel, y escriben `IPV7_PEERS` e `IPV7_HEAT` (antes había que editarlos a mano).

## Cómo continuar

1. Probar entre las dos máquinas Windows reales (PC A y notebook B) con el `.exe` nuevo.
2. En el nodo que hace de gateway, ejecutar `gateway.ps1` como administrador y en el otro
   `route add 0.0.0.0 mask 0.0.0.0 10.0.0.1 metric 1`.
3. Roadmap pendiente: relay/TURN para NAT simétrico, handshake Noise sin PSK, DNS interno.

## Notas

- `wintun.dll` debe estar junto a `ipv7_simbi.exe` en Windows.
- En Windows, `run-first.bat` requiere **Ejecutar como administrador**.
- Los dos extremos deben usar el mismo `IPV7_PSK` y el mismo `IPV7_NONCE`.
