# IPv7-SIMBI — Benchmarks estimados

Este documento contiene estimaciones teóricas de rendimiento comparando IPv7-SIMBI con las pilas de red actuales. No son mediciones reales; son proyecciones basadas en las propiedades del diseño (encabezado mínimo, enrutamiento O(1), zero-copy, multicast en cascada, sin broadcast).

---

## 1. Latencia por salto en un router

| Escenario | IPv4/TCP | IPv6/UDP | IPv7-SIMBI (estimado) | Justificación |
|-----------|----------|----------|----------------------|---------------|
| Procesamiento por salto | 5–20 µs | 3–10 µs | 0.5–2 µs | El header cabe en 64 bits y la decisión es una lectura de índice O(1). |
| Búsqueda de ruta | Tabla hash / árbol | Tabla hash | Acceso directo a array | No hay búsqueda: el índice está en el paquete. |
| Encolado en kernel | 2–10 µs | 1–5 µs | <1 µs (con NIC/SmartNIC) | Zero-copy permite dejar el payload en el búfer del NIC. |

### Notas
- En hardware de 1 GHz, un ciclo es 1 ns. Un encabezado de 64 bits puede procesarse en unos pocos ciclos.
- TCP requiere actualizar números de secuencia, ventanas y checksums, lo que aumenta el tiempo por salto.

---

## 2. Overhead de cabecera

| Protocolo | Cabecera base mínima | Con opciones | Payload de 64 B | Payload de 1 500 B | Payload de 9 000 B |
|-----------|----------------------|--------------|-----------------|--------------------|--------------------|
| IPv4 + TCP | 40 B | 40–60 B | 62–100% | 2.7–4% | 0.4–0.7% |
| IPv6 + UDP | 48 B | 48–96 B | 75–150% | 3.2–6.4% | 0.5–1.1% |
| IPv7-SIMBI | 8 B | 8 B + 2 B CRC | 15.6% | 0.67% | 0.11% |

### Impacto
- Para paquetes de control de 32–64 B, IPv7-SIMBI reduce el overhead de 2x–5x.
- En streaming de video pesado, la diferencia es menor pero sigue siendo positiva.

---

## 3. Throughput en streaming de video

| Métrica | TCP/IPv4 | UDP/IPv6 | IPv7-SIMBI multicast (estimado) |
|---------|----------|----------|---------------------------------|
| Usuarios soportados por servidor | ~10 000–50 000 | ~100 000 con replicación en red | ~1 000 000+ con cascada |
| Ancho de banda del origen | Crece con cada usuario | Crecimiento lineal | Casi constante |
| Latencia al añadir usuarios | Aumenta | Aumenta | Crece logarítmica con la profundidad del árbol |

### Justificación
- En multicast en cascada, cada nodo intermediario multiplica localmente el puntero al paquete. El origen envía un solo flujo.
- TCP/IPv4 requiere una conexión y un buffer por usuario.

---

## 4. Consumo energético por paquete

| Componente | IPv4/TCP | IPv7-SIMBI (estimado) |
|------------|----------|------------------------|
| CPU por paquete (nodo intermediario) | 100–500 nJ | 5–50 nJ |
| Accesos a memoria RAM | 3–10 | 1–2 (sólo el header) |
| Adecuado para sensores de batería | Regular | Alto |

### Justificación
- Procesar 64 bits de control consume mucho menos que interpretar cabeceras de 40–60 bytes, actualizar tablas y mantener estado de conexión.
- En FPGA o ASIC, el módulo IPv7-SIMBI podría consumir fracciones de vatio a 1 Gpps.

---

## 5. Tiempo de failover y autoreparación

| Escenario | BGP + OSPF | SD-WAN | IPv7-SIMBI (estimado) |
|-----------|------------|--------|------------------------|
| Detectar enlace caído | Segundos a minutos | 50 ms – 1 s | <1 ms a nivel local |
| Recalcular ruta | Consultas a tablas y vecinos | Controlador central | Explorador local inmediato |
| Degradación elegante | No existe en capa 3 | Parcial | Nativa, cambio a Autocuración en un salto |

### Notas
- La red no necesita elegir un líder ni consenso global. El primer paquete afectado degrada a Explorador/Autocuración.
- En BGP, el reconvergimiento puede tardar minutos; en IPv7-SIMBI es una decisión de hardware por paquete.

---

## 6. Escalabilidad de multicast

| Usuarios | Tráfico desde origen (IPv4) | Tráfico desde origen (IPv7-SIMBI) |
|----------|------------------------------|-----------------------------------|
| 1 | 1x | 1x |
| 100 | 100x | 1x |
| 10 000 | 10 000x | 1x |
| 1 000 000 | Impracticable | 1x (la multiplicación ocurre en nodos) |

---

## 7. Jitter en tráfico en tiempo real

| Protocolo | Jitter típico (RT) | Motivo |
|-----------|---------------------|--------|
| TCP | Alto (retransmisiones) | Ventanas, ACKs, reenvíos |
| UDP | Medio | Sin garantías, pero sin retransmisiones |
| IPv7-SIMBI efímero | Muy bajo | Fire-and-forget, cola de prioridad, no ACKs |
| IPv7-SIMBI confiable | Medio-bajo | ACKs mínimos sólo cuando el paquete lo requiere |

---

## 8. Consumo de memoria por nodo

| Pila | Tablas | Estado por conexión | Consumo en router core |
|------|--------|---------------------|------------------------|
| IPv4/TCP | ARP, FIB, conexiones | ~1 KB por flujo | 10–100 MB por 100k flujos |
| IPv6 | Neighbor, FIB, conexiones | Similar | Similar |
| IPv7-SIMBI | Mapa de calor, routing table | Ninguno por flujo | KBs de memoria por nodo |

### Justificación
- IPv7-SIMBI no mantiene conexiones. Guarda un mapa de calor de prefijos y una tabla de 16 entradas.
- Cada paquete es autosuficiente: no requiere estado compartido más allá de la identidad y el nonce.

---

## 9. Resumen de mejoras proyectadas

| Área | Mejora sobre IPv4/TCP (estimada) |
|------|----------------------------------|
| Latencia por salto | 5x–20x menor |
| Overhead para paquetes pequeños | 5x–10x menor |
| Throughput multicast | 100x–10 000x mayor |
| Energía por paquete | 5x–50x menor |
| Tiempo de failover | 1000x menor |
| Memoria por conexión | ~infinitamente menor (no mantiene estado por flujo) |

---

## 10. Advertencias importantes

- Estas cifras son **proyecciones teóricas**, no benchmarks medidos.
- Un prototipo real sobre UDP no alcanza los mismos números que un ASIC o un módulo de kernel.
- La seguridad real, el cifrado y la gestión de claves añadirían latencia y overhead que aún no están cuantificados.
- El rendimiento depende del hardware: un router IPv7-SIMBI en FPGA se acercaría más a estas estimaciones que el mismo código en una CPU de propósito general.
