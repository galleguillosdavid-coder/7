# Seis grados de separación como base de la red

> Propuesta de diseño. Nada de esto está implementado todavía; el objetivo es
> decidir si adoptamos este modelo antes de tocar el enrutamiento.

## 1. La idea

El experimento de Milgram (1967) mostró que dos personas cualesquiera están
unidas por una cadena de unos seis conocidos. Pero el resultado interesante para
una red no es que la cadena **exista**: es que las personas **la encontraron**,
sin conocer el grafo completo, decidiendo en cada paso a qué conocido reenviar la
carta. Eso es exactamente lo que hace un router.

Kleinberg (2000) explicó cuándo eso es posible. Un nodo tiene dos clases de
enlaces:

- **contactos locales**: sus vecinos inmediatos;
- **contactos lejanos**: unos pocos atajos elegidos al azar, con probabilidad
  proporcional a `1 / d(u,v)^r`, donde `d` es la distancia en el espacio
  subyacente.

Y el resultado clave: el enrutamiento voraz (reenviar siempre al contacto más
cercano al destino) es rápido —`O(log² n)` saltos— **sólo** cuando `r` iguala la
dimensión del espacio. Si los atajos son demasiado aleatorios (`r` chico) o
demasiado locales (`r` grande), las rutas cortas existen pero nadie las
encuentra. Los seis grados no son una propiedad del grafo: son una propiedad del
grafo **más** la regla de decisión local.

Esto encaja con los principios del proyecto: la decisión sigue siendo local y
O(1), no hay coordinador, y el estado por nodo es logarítmico en vez de global.

## 2. Qué falta hoy en IPv7-SIMBI

El modo Explorador ya es enrutamiento voraz, pero le falta la métrica:

```rust
// hoy: coincidencia de prefijo textual del DID
if did.starts_with(pref) { ...elige el prefijo mas largo... }
// si ningun prefijo coincide -> puerto al azar entre los pares
```

Un DID es una cadena semántica, así que `"99X"` y `"99Y"` se consideran vecinos y
`"99X"` y `"AB1"` son incomparables. No hay noción de "más cerca del destino", y
sin ella no hay enrutamiento voraz posible: por eso hoy la red sólo funciona con
un mapa de calor escrito a mano y con todos los nodos conectados entre sí.

## 3. El diseño propuesto

### 3.1 Un anillo de identidades

Cada DID recibe una coordenada de 64 bits:

```
coord(did) = H(did)                       # H = hash de 64 bits
d(a, b)    = min( (a-b) mod 2^64, (b-a) mod 2^64 )    # distancia en el anillo
```

El anillo es unidimensional, así que la `r` óptima de Kleinberg es **1**: los
atajos se eligen con probabilidad `∝ 1/d`. Esta es la construcción de Symphony,
la variante de mundo pequeño de Chord.

### 3.2 Los contactos de cada nodo

| Clase | Cuántos | Cómo se eligen |
|---|---|---|
| locales | 2 | sucesor y predecesor en el anillo |
| lejanos | k ≈ 4 | `dist = 2^(63·u)` con `u` uniforme en [0,1); se enruta hacia esa coordenada y se toma el nodo que responde |
| directos | los que ya hay | los pares estáticos de `IPV7_PEERS` siguen siendo contactos válidos |

Sortear `2^(63·u)` es equivalente a muestrear con densidad `1/d`: da tantos
atajos cortos como largos en escala logarítmica, que es justo lo que necesita el
paso voraz para dividir la distancia a la mitad en cada salto.

Con `k = 4` y 2 vecinos, cada nodo mantiene **6 contactos** sin importar si la
red tiene 10 o 10 000 000 de nodos.

### 3.3 La regla de reenvío

```
reenviar al contacto c que minimiza d(coord(c), coord(destino))
si ninguno esta mas cerca que yo  ->  el destino soy yo (o esta caido)
```

Una comparación de enteros por contacto: sigue siendo O(1) y cabe en el modo
Explorador actual, sin agregar un `z8` nuevo.

### 3.4 Costo esperado

| n nodos | saltos esperados (≈ log²n / k) |
|---|---|
| 100 | ~4 |
| 10 000 | ~14 |
| 1 000 000 | ~28 |

Los "seis grados" literales corresponden a redes de cientos de nodos; a escala
mundial el número real es unas decenas. El TTL de 16 que hoy lleva
`hops_remaining` alcanza para unos ~5 000 nodos: habría que subirlo a 64.

## 4. Qué cambia en el código

| Pieza | Cambio |
|---|---|
| `packet.rs` | `coord(did)` de 64 bits y `ring_distance(a,b)` |
| `router.rs` | `resolve_heat` pasa a comparar distancias; el prefijo textual queda como anulación manual |
| `router.rs` | tabla de contactos (2 vecinos + k atajos) con soft-state, refrescada por `Z_HELLO` |
| `main.rs` | `IPV7_SHORTCUTS` (por defecto 4), TTL configurable |
| tracker | pasa de directorio a punto de arranque: sólo hace falta **un** contacto para entrar al anillo |

No hace falta tocar el encabezado de 8 bytes ni los modos `z8`. El `anchor` de
16 bits ya es un hash del DID destino; se puede reinterpretar como los 16 bits
altos de la coordenada y usarlo para descartar en O(1) sin leer el DID completo.

## 5. Qué ganamos y qué arriesgamos

**Ganamos**

- Se puede entrar a la red conociendo a un solo nodo, no a todos.
- El estado por nodo deja de crecer con la red (6 contactos, no n-1).
- La autocuración deja de ser inundación: si un contacto muere, se sortea otro.
- El mapa de calor deja de escribirse a mano.

**Arriesgamos**

- Se paga latencia: hoy el túnel de dos nodos es de un salto; con anillo, dos
  nodos siguen siendo un salto, pero un tercero puede quedar a dos o tres, y cada
  salto agrega un cifrado y descifrado.
- La coordenada es un hash: dos nodos vecinos en el anillo pueden estar en
  continentes distintos. Kleinberg optimiza saltos, no milisegundos. Lo habitual
  es sesgar la elección de atajos por RTT medido (que ya lo tenemos en
  `cfg.latencies`).
- Un atacante que elige sus DIDs puede colocarse alrededor de una víctima en el
  anillo (ataque de eclipse). La defensa estándar es derivar el DID de una clave
  pública, y no de una cadena elegida por el usuario.
- NAT: un atajo sólo sirve si se puede establecer. Con NAT simétrico hará falta
  relay, y eso rompe la uniformidad del muestreo.

## 6. Orden sugerido

1. `coord` + `ring_distance` + reenvío voraz, con la topología estática actual
   (sin atajos). Verificable con los scripts de prueba que ya existen.
2. Contactos de anillo (sucesor/predecesor) por `Z_HELLO`.
3. Atajos sorteados con `1/d` y refresco periódico.
4. Sesgo por latencia y derivación del DID desde clave pública.

Los pasos 1 y 2 se pueden hacer sin romper la compatibilidad: mientras
`IPV7_HEAT` tenga entradas, esas mandan.

## Referencias

- S. Milgram, *The Small World Problem*, 1967.
- J. Kleinberg, *The Small-World Phenomenon: An Algorithmic Perspective*, 2000.
  <https://www.cs.cornell.edu/info/people/kleinber/swn.pdf>
- Manku, Bawa, Raghavan, *Symphony: Distributed Hashing in a Small World*, 2003.
