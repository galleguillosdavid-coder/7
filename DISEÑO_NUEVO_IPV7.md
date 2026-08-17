# IPv7-SIMBI v2 — Visión, principios y especificación

## Hacia dónde queremos llegar

Construir una **red de transporte alternativa** que mejore a IPv4/IPv6 para
comunicación directa entre personas y dispositivos. No queremos depender de
IPs centrales, DNS tradicional, ni de grandes backbones monolíticos.

El objetivo final es una **VPN P2P privada, cifrada y sin intermediarios**
que funcione entre Windows y Linux, conecte dos o más nodos aunque estén
detrás de NAT, y permita navegar por internet compartiendo la conexión de
uno de ellos.

## ¿Por qué "IPv7"?

IPv4 se agotó. IPv6 es técnicamente bueno pero no se adoptó masivamente por
ser demasiado disruptivo y mantener la lógica de direccionamiento jerárquico.
IPv7 propone:

- Direcciones más cortas y legibles.
- NAT traversal nativo desde el diseño.
- Cifrado obligatorio en capa de red.
- Identidad de nodo separada de la IP física.
- Conectividad punto a punto sin depender de servidores centrales.

## Principios de diseño

1. **Simplicidad absoluta.** Un solo concepto por fase. Nada de exploradores,
   trenes bala, ni mapas de calor en el MVP.
2. **Cifrado siempre.** Todo payload va cifrado con una PSK (después con Noise).
3. **Identidad = DID.** Un nodo se identifica por un nombre corto (A, B, gateway),
   no por su IP. La IP es solo un punto de encuentro.
4. **No confiar en el camino.** El paquete cifrado viaja por UDP puro; el
   remitente y destinatario son los únicos que lo entienden.
5. **Resolver NAT, no ignorarlo.** El diseño parte de la premisa de que ambos
   nodos están detrás de NAT. La solución primaria es un tracker/relay ligero.
6. **Un solo archivo.** Todo el motor en un `main.rs` de menos de 600 líneas.

## Sistema de direccionamiento de 12 símbolos

Para diferenciar a IPv7 de IPv4/IPv6, se propone un espacio de direcciones
compacto de **12 caracteres alfanuméricos en base-12**:

- Símbolos permitidos: `0 1 2 3 4 5 6 7 8 9 a b`
- Longitud fija: 12 símbolos.
- Espacio aproximado: 12^12 ≈ 9 billones de direcciones.
- Ejemplo: `a1b2c3d4e5f6`

En el MVP esta dirección es solo el **DID del nodo** (identificador). Más
adelante puede extenderse a direcciones de servicio, zonas y subredes.

```
Ejemplo de DID de 12 chars:
a1b2c3d4e5f6

Se puede acortar visualmente usando prefijos para el MVP:
A     -> a1b2c3d4e5f6
B     -> b1a2c3d4e5f6
GW    -> c1a2c3d4e5f6
```

## Fases de desarrollo

### Fase 1 — Túnel punto a punto en LAN

- Crear TUN en ambos nodos.
- Enviar paquetes IP crudos por UDP a un peer configurado a mano.
- Cifrar con ChaCha20-Poly1305 y PSK.
- Sin tracker, sin STUN, sin gateway.

### Fase 2 — NAT traversal con tracker

- Pequeño tracker UDP en una VPS.
- Cada nodo se registra con su DID + IP:puerto externo.
- Cada nodo pregunta por el otro.
- Si el túnel directo falla, el tracker retransmite (relay).

### Fase 3 — Gateway a internet

- Un nodo (el que tiene internet buena) comparte su conexión.
- El otro nodo enruta todo el tráfico por el TUN.
- En Windows: ICS. En Linux: `sysctl net.ipv4.ip_forward=1` + `iptables -t nat`.

### Fase 4 — DID y base-12

- Los DID se generan como 12 caracteres base-12.
- El tracker resuelve DID a dirección IP:puerto actual.
- Apariencia de "internet propia" dentro del TUN.

### Fase 5 — Producción

- Handshake Noise en lugar de PSK.
- Autenticación por DID.
- Posible integración con eBPF/XDP o io_uring.

## Especificación técnica mínima (MVP fase 1-2)

### Lenguaje y dependencias

- Rust, un solo binario.
- Crates: `tun`, `chacha20poly1305`.

### Archivo de configuración

```
NODE_ID=a1b2c3d4e5f6
BIND=0.0.0.0:9001
PEER=140.232.64.2:53120
PSK=clave_secreta_compartida
TUN_NAME=ipv7
TUN_ADDR=10.0.0.2
TUN_DST=10.0.0.1
```

### Formato de paquete UDP

| Campo          | Tamaño     | Descripción                                   |
|----------------|------------|-----------------------------------------------|
| magic          | 4 bytes    | `0x49503637` ("IPv7" en bytes)                |
| version        | 1 byte     | 1                                             |
| did_src_len    | 1 byte     | Longitud del DID origen                       |
| did_dst_len    | 1 byte     | Longitud del DID destino                      |
| nonce          | 12 bytes   | Nonce ChaCha20-Poly1305                       |
| did_src        | variable   | DID origen (hasta 12 chars)                   |
| did_dst        | variable   | DID destino (hasta 12 chars)                  |
| payload        | variable   | Paquete IP cifrado + tag 16 bytes             |

### Lógica del nodo

1. Lee del TUN, cifra el paquete IP, lo envía por UDP al peer.
2. Recibe por UDP, descifra, si el DID destino coincide con el propio lo
   escribe en el TUN.
3. Si el DID destino es otro y se conoce su dirección, lo reenvía.
4. Si recibe paquete de una IP:puerto desconocida y el DID origen es
   reconocible, guarda esa IP:puerto para responderle.

## Soluciones a los problemas reales

| Problema                         | Solución                                   |
|----------------------------------|--------------------------------------------|
| NAT simétrico                    | Tracker/relay en VPS                       |
| Cifrado                          | ChaCha20-Poly1305 con PSK                  |
| Identidad                        | DID de 12 chars base-12                    |
| Adaptador TUN en Windows         | Wintun + ejecución como admin              |
| Gateway a internet               | IP forwarding + NAT (Linux) o ICS (Windows)|
| Descubrimiento de peer           | Tracker central mínimo en VPS              |

## Advertencias

- IPv7 no es un estándar oficial. Es una propuesta experimental.
- El DID base-12 es un concepto novedoso: en el MVP puede ser solo un string.
- Wintun requiere privilegios de administrador.
- Sin tracker en una VPS, no hay manera confiable de atravesar NAT simétrico.
- No construir funciones avanzadas hasta que el túnel básico funcione.

## Mensaje final

La versión anterior falló por acumular demasiadas ideas (modos, heat maps,
explotadores) antes de resolver lo esencial: **túnel + NAT traversal + cifrado**.
Este documento reconstruye el proyecto desde lo mínimo, con una visión clara
hacia una red alternativa simple, privada y cifrada.
