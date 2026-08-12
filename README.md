# IPv7-SIMBI — Guía del programa

> **ESTADO: CONGELADO** — Esta versión del documento es estable e inmodificable. Cualquier modificación futura requiere aprobación explícita del responsable del proyecto.

Este documento explica en lenguaje natural qué hace el programa, para qué sirve cada parte y cómo se conectan entre sí. No es una especificación formal; es una guía práctica para entender el código y poder ejecutarlo.

---

## 0. Principios de diseño

IPv7-SIMBI busca que tanto el código como el transporte de datos sean elegantes, eficientes y fáciles de mantener. Los siguientes principios guían cada decisión de protocolo e implementación:

### 0.1 Belleza matemática

Prefiere expresiones compactas, simétricas y generales que reemplacen docenas de casos especiales. Cada regla del protocolo debería poder describirse con una ecuación o invariante simple. Si una decisión requiere demasiadas excepciones, es candidata a refactorización.

### 0.2 Economía algorítmica

Se prefieren algoritmos cortos, con pocos casos especiales y complejidad temporal baja. Un algoritmo fácil de entender y ejecutar es mejor que uno sofisticado que solo ahorre unos pocos ciclos. La complejidad también se mide en tiempo de lectura y mantenimiento.

### 0.3 Simplicidad funcional

A veces lo mejor es lo más sencillo. Mantener componentes pequeños, con una sola responsabilidad, produce código limpio y un sistema de transporte eficiente. Cada campo del encabezado y cada modo `z8` debe justificar su existencia.

### 0.4 Eficiencia en el transporte

El protocolo evita copiar, reordenar o inspeccionar datos innecesarios. El router solo toca los 8 bytes de control; el payload fluye sin ser modificado. Esto reduce latencia, consumo de energía y superficie de errores.

### 0.5 Robustez por degradación

Los sistemas simples se recuperan mejor. Si una ruta cristalizada falla, el nodo degrada a un modo más lento pero más resiliente sin perder el paquete. La corrección debe ser local, no global.

### 0.6 Primera forma útil: VPN ultra sencilla

El desarrollo se orienta inicialmente a ofrecer una VPN mínima. En lugar de construir una pila de red completa desde el primer día, el primer producto útil conecta dos o más computadoras como nodos IPv7-SIMBI y transporta paquetes entre ellas de forma transparente. Eso da una utilidad real inmediata, mientras se validan el encabezado de 8 bytes, el enrutamiento por DID y los modos Z8.

---

## 1. Qué es este programa

Es un **prototipo funcional de un router IPv7-SIMBI** escrito en Rust. En lugar de implementar el protocolo en el kernel del sistema operativo, simula el comportamiento de los nodos usando **sockets UDP reales**. Esto permite ver paquetes viajando entre procesos e incluso entre máquinas distintas, sin necesidad de drivers de red ni hardware especial.

El programa puede correr en tres modos:

1. **Demo de 4 nodos**: levanta automáticamente una red A→B→C→D en localhost y envía tres paquetes de ejemplo.
2. **Router individual**: escucha en una IP y puerto, y reenvía paquetes según las reglas que recibe por variables de entorno.
3. **Sender de prueba**: inyecta un único paquete hacia un router y termina.

---

## 2. Conceptos básicos

### 2.1 DID

Un **DID** (Entidad Lógica Descentralizada) es el identificador de destino. No es una dirección IP ni un nombre de dominio; es una cadena semántica, como `99X`. Los nodos aprenden por qué puerto se puede alcanzar cada DID gracias a un **mapa de calor** de prefijos: si un DID empieza con `99`, quizá se alcance por el puerto 5; si empieza con `9`, por el puerto 2.

### 2.2 Nodo

Un nodo es un proceso que:
- Escucha paquetes UDP.
- Lee los primeros 8 bytes del paquete (la cabecera IPv7-SIMBI).
- Valida el PoW (prueba de trabajo) y firma un PoS (prueba de servicio).
- Decide si reenviar el paquete, entregarlo localmente o descartarlo.

### 2.3 Puerto lógico

Dentro del programa, un **puerto** no es un puerto de red de TCP/UDP. Es un número de 0 a 15 que el nodo usa internamente para elegir por dónde reenviar. A cada puerto lógico se le asigna una dirección UDP real, como `127.0.0.1:9001`.

---

## 3. El encabezado de 8 bytes

Cada paquete comienza con 8 bytes que se empaquetan en un solo número de 64 bits. Están distribuidos así:

- **Bits 0-2**: `z8` — el modo del paquete (Tren Bala, Explorador, Multicast, etc.).
- **Bit 3**: `flex` — indica si el paquete tolera pérdida (por ejemplo video) o es estricto.
- **Bits 4-7**: `route_index` — índice de la ruta cristalizada, de 0 a 15.
- **Bits 8-15**: `pow_signature` — prueba de trabajo que el emisor debe resolver.
- **Bits 16-47**: `anchor` — hash del DID destino, usado para validación rápida.
- **Bits 48-55**: `length` — tamaño del payload.
- **Bits 56-63**: `sequence` — número de secuencia para reordenar fragmentos.

Este encabezado es lo único que el router necesita mirar. El resto del paquete (payload) viaja sin que el router lo toque, aunque en este prototipo en Rust sí se copia en memoria por simplicidad.

---

## 4. Modos Z8

El campo `z8` determina qué lógica se ejecuta en el nodo.

- **0 — Reposo**: no hace nada. Reservado para uso futuro.
- **1 — Tren Bala**: usa una ruta cristalizada. El nodo lee `route_index`, busca el puerto en su tabla de rutas y reenvía. Si falla, el nodo **degrada** el paquete a Autocuración y prueba otros caminos.
- **2 — Multicast**: el nodo envía copias del paquete a todos sus pares excepto por donde llegó, convirtiendo cada copia en Tren Bala.
- **3 — Explorador**: no usa una ruta predefinida. El nodo busca en su mapa de calor el mejor puerto según el prefijo del DID destino. Si no sabe, descarta el paquete.
- **4 — Bit-Flip**: invierte el bit `flex` y sigue explorando.
- **5 — Prueba de Servicio**: responde con un eco por el mismo puerto de entrada.
- **6 — Autocuración**: envía el paquete a todos los pares excepto el de entrada. Si el nodo no tiene pares, lo entrega localmente.
- **7 — Lockdown**: descarta el paquete inmediatamente.

---

## 5. Cómo se envía un paquete por la red

### 5.1 Serialización

Antes de enviar un paquete por UDP, el programa lo convierte en bytes:

1. Los 8 bytes del encabezado.
2. Un byte con la longitud del `DID_src` y luego esos bytes.
3. Un byte con la longitud del `DID_dst` y luego esos bytes.
4. Un byte con la longitud del `payload` y luego esos bytes.
5. Dos bytes de **CRC-16-CCITT** calculados sobre todo lo anterior.

Cuando un nodo recibe bytes UDP, hace el proceso inverso: parsea, verifica el CRC, desempaqueta el encabezado y extrae los DIDs y el payload. Si el CRC no coincide, el paquete se descarta como corrupto.

### 5.2 Reenvío real

El nodo usa `std::net::UdpSocket` para escuchar y para `send_to`. Cada `send_to` es un datagrama UDP real que viaja por la red (o por localhost). Eso significa que este prototipo no es una simulación matemática: los paquetes realmente saltan entre procesos y pueden probarse entre computadoras distintas.

---

## 6. PoW y PoS

### 6.1 PoW — Prueba de Trabajo

Cada nodo tiene un `nonce` y una máscara de dificultad. Cuando un paquete llega, el nodo verifica:

```
(pow_signature XOR nonce) AND máscara == 0
```

Si la máscara es `0xFF`, el emisor debe resolver un puzzle de 8 bits completo. Si la máscara es `0x00`, cualquier `pow_signature` es válida. Esto permite darle a dispositivos IoT verificados un pase fácil mientras se exige trabajo a nodos no verificados.

### 6.2 PoS — Prueba de Servicio

Cada nodo que pasa el paquete calcula una firma criptográfica ligera sobre su propio ID y el DID destino, usando un secreto local. La firma demuestra que el nodo realmente vio el paquete y está en condiciones de reenviarlo. Esto ayuda a detectar nodos maliciosos que intenten secuestrar rutas.

---

## 7. Enrutamiento

### 7.1 Modo Explorador

Es la primera fase. Un paquete explorador viaja de nodo en nodo buscando el destino. En cada salto:
- El nodo mira su mapa de calor de prefijos.
- Elige el puerto que mejor coincida con el DID destino.
- Evita devolver el paquete por el mismo puerto de entrada para no caer en bucles.
- Si llega al destino (el mapa de calor indica puerto 0), se entrega.

### 7.2 Modo Tren Bala

Una vez que se conoce una buena ruta, los siguientes paquetes usan el modo Tren Bala. Cada nodo tiene una **tabla de rutas cristalizadas** de 16 posiciones: el `route_index` del paquete dice qué entrada usar. El nodo reenvía y suma 1 al `route_index` para que el siguiente nodo use la siguiente entrada. Esto es O(1): no busca, no calcula, solo lee un índice.

### 7.3 Degradación elegante

Si un puerto cristalizado está caído, el nodo no descarta el paquete. Cambia el `z8` a `Z_AUTOCURACION` y reenvía el paquete por todos los otros pares. Esto permite seguir entregando mientras se encuentra o recristaliza una ruta nueva.

### 7.4 Decisiones probabilísticas

Algunas decisiones del router se pueden resolver con ecuaciones sencillas en lugar de tablas grandes:

- **Balanceo de carga por gradiente de calor.** Si un prefijo DID coincide con varios puertos, se elige el puerto `i` con probabilidad inversa a su latencia medida `lat_i`:

  ```
  P(puerto_i) = (1 / (lat_i + 1)) / Σ(1 / (lat_j + 1))
  ```

- **Explorador sin calor.** Cuando el nodo no conoce ninguna ruta, elige un par al azar entre los que no sean el de entrada:

  ```
  P(puerto_i) = 1 / (n_pares - 1)
  ```

- **Degradación por fallos.** La probabilidad de abandonar el modo Tren Bala y pasar a Autocuración puede crecer con el contador de fallos `f` y un umbral `U`:

  ```
  p_fallo = min(f / U, 1.0)
  ```

Estas fórmulas evitan mantener tablas de umbrales o prioridades y se computan en cada salto con los valores que ya maneja el nodo.

---

## 8. Multicast y Autocuración

- **Multicast**: un nodo recibe un paquete marcado como Multicast y lo copia a cada uno de sus pares. Es útil para streaming: un solo paquete entra al nodo y se multiplica localmente, sin saturar al emisor original.
- **Autocuración**: intenta enviar el paquete por todos los caminos disponibles. Se usa como mecanismo de emergencia cuando una ruta cristalizada falla o cuando se desconoce la mejor ruta.

---

## 9. Cómo ejecutar el programa

### 9.1 Demo automático

```powershell
cd C:\Users\Frondabrick\Desktop\dvd\7\ipv7_simbi
cargo run
```

Esto levanta A, B, C y D en localhost y envía un Explorador, un Tren Bala y un Multicast.

### 9.2 Router individual

```powershell
$env:IPV7_NODE_ID="A"
$env:IPV7_BIND="127.0.0.1:9010"
$env:IPV7_PEERS="2:127.0.0.1:9011;5:127.0.0.1:9012"
$env:IPV7_ROUTES="0:2;1:5"
$env:IPV7_HEAT="9:2;99X:0"
$env:IPV7_DOWN_PORTS="2"
$env:IPV7_NONCE="0xA5"
target\debug\ipv7_simbi.exe
```

### 9.3 Enviar un paquete de prueba

```powershell
$env:IPV7_SEND_TO="127.0.0.1:9010"
$env:IPV7_SEND_Z8="1"
$env:IPV7_SEND_ROUTE="0"
$env:IPV7_SEND_DST="99X"
$env:IPV7_SEND_POW="0xA5"
$env:IPV7_SEND_PAYLOAD="hola"
target\debug\ipv7_simbi.exe
```

---

## 10. Variables de entorno

### Para un nodo

- `IPV7_NODE_ID`: nombre del nodo.
- `IPV7_BIND`: dirección UDP donde escuchar, por ejemplo `127.0.0.1:9010`.
- `IPV7_PEERS`: lista de pares, con el formato `puerto_lógico:dirección;...`. Ejemplo: `2:127.0.0.1:9011;5:127.0.0.1:9012`.
- `IPV7_ROUTES`: tabla de ruta cristalizada, con el formato `índice:puerto_lógico;...`. Ejemplo: `0:2;1:5`.
- `IPV7_HEAT`: mapa de calor de prefijos, con el formato `prefijo:puerto_lógico;...`. Ejemplo: `9:2;99:5;99X:0`.
- `IPV7_DOWN_PORTS`: puertos lógicos que se consideran caídos. Ejemplo: `2;5`.
- `IPV7_NONCE`, `IPV7_IOT_MASK`, `IPV7_UNVERIFIED_MASK`: parámetros del PoW.
- `IPV7_WHITELIST`: DIDs que reciben trato de dispositivo IoT, separados por `;`.

### Para el sender

- `IPV7_SEND_TO`: dirección destino.
- `IPV7_SEND_DST`, `IPV7_SEND_SRC`, `IPV7_SEND_PAYLOAD`.
- `IPV7_SEND_Z8`: modo (1 = Tren Bala, 3 = Explorador, etc.).
- `IPV7_SEND_ROUTE`: `route_index` inicial.
- `IPV7_SEND_POW`: firma PoW en hexadecimal.
- `IPV7_SEND_SEQ`, `IPV7_SEND_FLEX`.

---

## 11. Limitaciones del prototipo

- Es un protocolo de aplicación sobre UDP, no un protocolo de red real en el kernel.
- No implementa aún ventanas deslizantes, control de congestión, ni cifrado de cebolla.
- El PoW es una demostración de 8 bits; no resiste ataques reales.
- La degradación a Autocuración es útil pero puede generar duplicados si varios caminos llegan al mismo destino.
- Los paquetes no se fragmentan; si se excede la MTU de la red subyacente, se pierden.

---

## 12. Para qué sirve este prototipo

Sirve para demostrar los conceptos centrales de IPv7-SIMBI en un entorno controlado: encabezado mínimo, enrutamiento por identidad, PoW, PoS, Tren Bala, Explorador, Multicast, Autocuración y degradación elegante. Es la base sobre la que se pueden construir las demás capas del `ROADMAP.md`.

---

## 13. Escenario de uso: el computador como hub del hogar

Los primeros dispositivos que implementarán IPv7-SIMBI serán computadoras. En ese escenario, la computadora actúa como **hub de conexión** o **gateway** para el resto de dispositivos del hogar.

- La computadora ejecuta un nodo IPv7 completo y se conecta a la red IPv7 pública o micelial.
- Los teléfonos, sensores, televisiones o consolas del hogar se conectan a la computadora por interfaces locales (Wi-Fi, Ethernet, USB, Bluetooth o WebSocket) sin necesidad de implementar IPv7 nativamente.
- El hub traduce entre IPv7 y las interfaces locales: recibe paquetes IPv7, los entrega al dispositivo correcto según su DID local, y retransmite lo que sale de la casa hacia la red IPv7.
- Es un componente **externo al núcleo**: mantiene tablas de presencia locales y políticas del hogar, pero no modifica el protocolo de 8 bytes ni los modos Z8.

Esto permite que el ecosistema crezca sin esperar a que cada dispositivo tenga un stack IPv7 completo; el computador hace de puente mientras el núcleo sigue siendo mínimo.
