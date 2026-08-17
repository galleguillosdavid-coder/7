# Diseño IPv7-SIMBI v2 — Especificación para reimplementación desde cero

## Contexto para la nueva IA

El usuario está reconstruyendo el proyecto desde cero porque la versión anterior
se volvió compleja y no logró cerrar el túnel P2P en una red con NAT.
**No mires el código anterior como referencia.** Este documento es el único
contexto que necesitás.

## Filosofía

- **Menos es más.** Un solo archivo de código, no módulos separados.
- **Reutilizar librerías.** No inventar criptografía, serialización ni drivers.
- **Conectividad antes que velocidad.** Que el túnel funcione; la optimización
  viene después.
- **Administración mínima.** Un archivo de configuración simple y un solo
  comando para correr.

## Objetivo del programa

Crear una VPN P2P simple entre dos nodos (Windows y/o Linux) que:

1. Cree un adaptador TUN virtual (`ipv7`) con IP 10.0.0.x.
2. Escuche en un puerto UDP local.
3. Envíe paquetes del TUN a un peer remoto cifrados con ChaCha20-Poly1305.
4. Descifre paquetes del peer y los escriba en el TUN.
5. Soporte NAT traversal mediante un tracker/relay público pequeño.
6. Permita a un nodo actuar como gateway de internet para el otro (opcional).

## Requisitos técnicos

- Lenguaje: **Rust** (portable, binario estático, puede usar crates `tun` y
  `chacha20poly1305`).
- Plataformas: Windows 10+ (Wintun) y Linux (tun/tap).
- Conexión: UDP IPv4.
- Cifrado: ChaCha20-Poly1305 con clave precompartida (PSK).
- Cada paquete UDP contiene un nonce único (12 bytes), payload cifrado y tag
  Poly1305 (16 bytes).

## Arquitectura mínima (un solo archivo)

```rust
use std::net::UdpSocket;
use std::sync::mpsc;
use tun::Configuration;
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use chacha20poly1305::aead::{Aead, NewAead};

fn main() {
    // 1. Leer config del entorno o de un archivo.
    // 2. Crear TUN con la IP local.
    // 3. Abrir socket UDP en el puerto local.
    // 4. Configurar peer estático o conectarse al tracker.
    // 5. Spawnear 2 hilos:
    //    a) Leer del TUN -> cifrar -> enviar a peer.
    //    b) Recibir del socket -> descifrar -> escribir en TUN.
    // 6. Loop principal infinito.
}
```

## Configuración (un solo archivo `ipv7.conf`)

```
NODE_ID=A
BIND=0.0.0.0:9001
PEER=140.232.64.2:53120
PSK=mi_clave_secreta
TUN_NAME=ipv7
TUN_ADDR=10.0.0.2
TUN_DST=10.0.0.1
```

## Protocolo de paquete UDP

| Campo          | Tamaño     | Descripción                                   |
|----------------|------------|-----------------------------------------------|
| magic          | 4 bytes    | `0x49503637` ("IPv7" en letras codificadas)   |
| version        | 1 byte     | 1                                             |
| did_src_len    | 1 byte     | Longitud del DID origen                       |
| did_dst_len    | 1 byte     | Longitud del DID destino                      |
| nonce          | 12 bytes   | Nonce de ChaCha20-Poly1305                    |
| did_src        | variable   | Identificador del nodo origen                 |
| did_dst        | variable   | Identificador del nodo destino                |
| ciphertext     | variable   | Payload original (IP) cifrado + tag 16 bytes  |

El header **no se cifra** excepto el payload. El nonce evita replay.

## Reglas básicas del router

1. Si recibo un paquete de una IP:puerto que no conozco, la guardo como peer
   temporal y respondo a esa IP:puerto.
2. Si el destino del paquete es mi DID, lo escribo en el TUN.
3. Si el destino es otro DID y conozco su dirección, lo reenvío.
4. Si no conozco la dirección, lo descarto.

## NAT traversal — el problema real

- Ambos nodos domésticos están detrás de NAT.
- Los NAT simétricos cambian el puerto externo por cada destino, por lo que el
  hole punching clásico falla.
- **Solución mínima viable:** un tracker/relay pequeño en una VPS con IP pública.
  - Cada nodo se registra en el tracker con su DID y su IP:puerto externo.
  - Cada nodo consulta el tracker para obtener la IP:puerto del otro.
  - Si el hole punching falla, el tracker hace de relay UDP (peor latencia, pero
    funciona).

## Advertencias importantes

- **No inventar un protocolo de routing ad-hoc.** Usar el kernel del SO para
  enrutar el tráfico dentro del TUN. El programa solo transporta paquetes IP
  entre los dos TUN.
- **No hardcodear IP/puerto del peer en el binario.** Usar config o tracker.
- **El cifrado requiere PSK compartida.** Sin handshake dinámico para el MVP.
- **Windows requiere ejecutar como administrador** para crear el TUN.
- **Un gateway requiere que el nodo gateway tenga forwarding activado** (Linux)
  o use ICS (Windows).

## Roadmap mínimo

1. MVP: dos nodos con TUN, PSK y peer estático. Solo funciona en LAN o con un
   nodo con IP pública.
2. STUN: descubrir IP:puerto externo propio.
3. Tracker: registrar y resolver peers.
4. Relay fallback: si el túnel directo no cierra, pasar por el tracker.
5. Gateway: un nodo comparte su internet con el otro.

## Recursos recomendados para la nueva IA

- Crate `tun` para crear TUN: https://docs.rs/tun
- Crate `chacha20poly1305`: https://docs.rs/chacha20poly1305
- Wintun: https://www.wintun.net
- Documentación Rust std::net::UdpSocket

## Límites del MVP

- Sin DHT, sin mDNS, sin IPv6.
- Sin autenticación por DID, solo PSK.
- Sin MTU discovery, usar 1400 bytes de MTU.
- Sin relay integrado: requiere VPS.

## Mensaje final para el usuario

La versión anterior fracasó principalmente porque intentó demasiadas cosas a la
vez y no resolvió el NAT traversal. Esta especificación propone un programa
más pequeño, medible y con un camino claro para conectar dos PCs reales.
