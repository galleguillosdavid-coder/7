# IPv7-SIMBI — Lista de implementaciones pendientes

> **ESTADO: CONGELADO** — Esta versión del documento es estable e inmodificable. Cualquier modificación futura requiere aprobación explícita del responsable del proyecto.

Este documento recoge las capacidades que se han conversado y las que faltan por materializar en el prototipo Rust. Cada item se describe en lenguaje natural, sin especificación formal.

---

## 0. Principios rectores

Antes de implementar una nueva capacidad, se evalúa contra los siguientes criterios:

- **Belleza matemática:** ¿La regla puede expresarse como una ecuación compacta e invariante? Se prefieren fórmulas a tablas de casos.
- **Economía algorítmica:** ¿El algoritmo es corto, de fácil ejecución y baja complejidad? Evitar optimizaciones prematuras que oscurezcan el código.
- **Simplicidad funcional:** ¿El componente hace una sola cosa y la hace bien? Lo más sencillo suele ser lo mejor.
- **Eficiencia en el transporte:** ¿Se evitan copias e inspecciones innecesarias del payload? El router debe tocar solo los 8 bytes de control.
- **Robustez por degradación:** ¿El fallo de un enlace produce una caída total o un camino alternativo local? Preferir recuperación sencilla a arquitecturas complejas de alta disponibilidad.

---

## 1. CRC-16 de integridad de cabecera

- **Lo que sí es:** Un verificador de integridad de 16 bits (CRC-16-CCITT) calculado sobre el encabezado y metadatos del paquete.
- **Lo que hace:** Detecta corrupción de bits antes de que el nodo gaste ciclos en PoW, PoS o enrutamiento.
- **Problema que resuelve:** Evita que un paquete dañado por ruido, buffers o errores de socket sea procesado como si fuera válido.
- **Lo que no es:** Cifrado, firma digital ni protección contra ataques maliciosos. Es solo detección de errores accidentales.
- **Ya implementado:** Sí, como trailer de 2 bytes en la serialización UDP.
- **Lo que podría o no ser:** Se podría mover al encabezado fijo de 8 bytes si se rediseña el formato, o reemplazarse por CRC-32 si se prefiere más robustez.

---

## 2. Familias de transmisión (confiable vs. efímera)

- **Lo que sí es:** Una clasificación de paquetes según la necesidad de garantía de entrega.
- **Lo que hace:** Marca si un paquete requiere ACK y retransmisión (confiable) o puede perderse sin consecuencias (efímero).
- **Problema que resuelve:** Evita forzar todo el tráfico a través de un protocolo pesado tipo TCP o dejarlo sin garantías como UDP.
- **Lo que no es:** Una ventana deslizante completa ni control de congestión. Solo es una decisión de encabezado.
- **Podría o no ser:** Implementarse usando bits libres del byte de control o extendiendo el encabezado con un campo de 2 bits.

---

## 3. Negociación de MTU mínimo común

- **Lo que sí es:** Un mecanismo de descubrimiento donde cada nodo anota el tamaño máximo de paquete que puede manejar y lo reduce si un intermediario es más restrictivo.
- **Lo que hace:** El paquete explorador lleva un campo MTU que cada nodo reescribe hacia el mínimo de su propio límite y el valor recibido.
- **Problema que resuelve:** Evita fragmentación en tránsito y asegura que todos los nodos de la ruta puedan procesar el paquete completo.
- **Lo que no es:** Un descubrimiento de topología ni un acuerdo de calidad de servicio. Es solo sincronización de tamaño.
- **Podría o no ser:** Implementada con un campo de 16 bits en el paquete explorador, o como parte de un handshake separado.

---

## 4. Multicast en cascada con niveles de protección

- **Lo que sí es:** Un nodo intermediario que recibe un flujo y lo multiplica localmente hacia varios pares sin saturar al emisor original.
- **Lo que hace:** Clona el paquete hacia cada suscriptor local según su nivel de acceso: abierto, firmado, cifrado o DRM.
- **Problema que resuelve:** Permite streaming masivo sin que el servidor mantenga miles de conexiones individuales.
- **Lo que no es:** Un sistema de gestión de derechos completo. Es una decisión de reenvío basada en bits de seguridad.
- **Podría o no ser:** Extendida con descifrado de clave de sesión en nodos autorizados o rechazo inmediato si no hay permiso.

---

## 5. Telemetría agregada en cascada

- **Lo que sí es:** Un resumen ascendente de cuántos dispositivos activos hay bajo cada rama del árbol de multicast.
- **Lo que hace:** Cada nodo intermediario suma sus suscriptores directos e indirectos y envía un micro-paquete de vuelta al emisor.
- **Problema que resuelve:** El emisor sabe cuántos receptores tiene en tiempo real sin recibir miles de mensajes individuales.
- **Lo que no es:** Un conteo exacto de paquetes entregados. Es una métrica de audiencia aproximada.
- **Podría o no ser:** Implementada con mapas atómicos, con un pulso por segundo o con acumulación bajo demanda.

---

## 6. Nodo híbrido prosumidor

- **Lo que sí es:** Un dispositivo que consume un flujo localmente y a la vez lo retransmite a otros nodos.
- **Lo que hace:** Bifurca el paquete: una copia va a la aplicación local y otra a la interfaz de red para fan-out.
- **Problema que resuelve:** Evita pedir varias copias del mismo contenido al enlace externo cuando varios dispositivos en la red local lo consumen.
- **Lo que no es:** Un servidor de aplicaciones. Es una función de reenvío dentro del mismo nodo.
- **Podría o no ser:** Incluido en la implementación actual como una flag que indica si el nodo también es destino final.

---

## 7. Token bucket para paquetes de control

- **Lo que sí es:** Un límite de cantidad y frecuencia de paquetes pequeños y prioritarios.
- **Lo que hace:** Permite paquetes de control mientras no excedan una cuota, y encola o descarta el exceso según la política.
- **Problema que resuelve:** Protege la red contra saturación por sensores descalibrados o ataques de denegación de servicio.
- **Lo que no es:** Un mecanismo de pago ni prioridad absoluta sin límites. Es solo control de tasa.
- **Podría o no ser:** Implementado por DID, por interfaz física o por tipo de paquete.

---

## 8. Handshake de descubrimiento y negociación

- **Lo que sí es:** El primer paquete entre dos dispositivos para descubrir ruta, negociar tamaño y condiciones de transmisión.
- **Lo que hace:** Viaja como explorador, recoge el MTU mínimo, la latencia de cada salto y los niveles de protección soportados.
- **Problema que resuelve:** Establece las reglas del enlace antes de enviar tráfico de datos real.
- **Lo que no es:** Una conexión orientada a sesión. No mantiene estado permanente; cristaliza una ruta.
- **Podría o no ser:** El mismo paquete explorador con campos adicionales, o un tipo de paquete nuevo.

---

## 9. Colas de prioridad por tipo de paquete

- **Lo que sí es:** Separar el tráfico entrante en colas distintas según sea de control, metadatos o carga útil.
- **Lo que hace:** Los paquetes pequeños y urgentes pasan primero; el resto espera o se procesa en segundo plano.
- **Problema que resuelve:** Garantiza latencia baja para telemetría crítica sin dejar de atender otras transmisiones.
- **Lo que no es:** Un sistema de calidad de servicio completo con reserva de ancho de banda.
- **Podría o no ser:** Implementado con una cola simple por prioridad o con varias colas limitadas por tamaño.

---

## 10. ACK y retransmisión confiable

- **Lo que sí es:** Un mecanismo básico de confirmación de recepción para paquetes marcados como confiables.
- **Lo que hace:** El receptor responde con un micro-ACK; si no llega, el emisor retransmite un número limitado de veces.
- **Problema que resuelve:** Da garantía de entrega para comandos críticos sin reimplementar TCP.
- **Lo que no es:** Una ventana deslizante con control de congestión. Es enviar y esperar, simple.
- **Podría o no ser:** Implementado con un bit de confiable y un contador de reintentos por paquete.

---

## 11. Identidad descentralizada DID / DID

- **Lo que sí es:** Un identificador persistente por dispositivo, independiente de la dirección física o red a la que esté conectado.
- **Lo que hace:** Permite que un dispositivo se mueva de una red a otra sin cambiar su identidad lógica.
- **Problema que resuelve:** Elimina la ruptura de sesiones al cambiar de Wi-Fi a móvil o al migrar entre ISPs.
- **Lo que no es:** Una dirección IP ni un nombre de dominio. Es una clave o hash de identidad.
- **Podría o no ser:** Una cadena semántica con prefijos que expresan grupos o capacidades.

---

## 12. Handover de movilidad sin cortes

- **Lo que sí es:** Actualización del vector de entrada de un DID cuando el dispositivo cambia de punto de acceso.
- **Lo que hace:** Los nodos cercanos aprenden que el dispositivo ahora se ve por otro puerto y redirigen el tráfico sin reiniciar la sesión.
- **Problema que resuelve:** Video, llamadas y transferencias no se caen al cambiar de red.
- **Lo que no es:** Un mecanismo de roaming de capa 2. Funciona a nivel de identidad DID.
- **Podría o no ser:** Implementado con micro-beacons de presencia que los nodos escuchan pasivamente.

---

## 13. Encriptación de cebolla opcional

- **Lo que sí es:** Cada salto descifra una capa exterior del paquete sin conocer el contenido final ni el emisor original.
- **Lo que hace:** El emisor envuelve el paquete con claves efímeras de cada nodo intermedio, de adentro hacia afuera.
- **Problema que resuelve:** Provee anonimato y privacidad sin confiar en ningún intermediario.
- **Lo que no es:** El modo por defecto. Solo se activa cuando el usuario lo exige.
- **Podría o no ser:** Implementada con Noise o con un intercambio de claves previo al cristalizar la ruta.

---

## 14. Redes miceliales y autocuración

- **Lo que sí es:** Cuando un enlace externo cae, los nodos vivos forman un ecosistema cerrado usando únicamente las conexiones locales disponibles.
- **Lo que hace:** Los exploradores reconfiguran automáticamente las rutas hacia los DIDs que aún respondan en la LAN.
- **Problema que resuelve:** Internet caído no deja a los dispositivos incomunicados si comparten una red física.
- **Lo que no es:** Un reemplazo de Internet. Es supervivencia local temporal.
- **Podría o no ser:** Activada automáticamente cuando la gravedad hacia el exterior se marca como infinita.

---

## 15. Back-pressure anti-congestión

- **Lo que sí es:** Un mecanismo por el cual un nodo ralentiza la aceptación de paquetes cuando sus colas o enlaces de salida están saturados.
- **Lo que hace:** Notifica o descarta paquetes de fondo antes de que se llene el búfer, conservando los de control.
- **Problema que resuelve:** Evita colapsos por ráfagas y mantiene latencia estable para tráfico prioritario.
- **Lo que no es:** Un protocolo de control de congestión completo con ventanas deslizantes.
- **Podría o no ser:** Implementado con umbrales de ocupación de cola por tipo de paquete.

---

## 16. Balanceo de carga por gradiente de calor

- **Lo que sí es:** Reparto de paquetes entre varios puertos que apuntan a destinos con el mismo prefijo DID.
- **Lo que hace:** Si hay varias rutas hacia un destino, elige la de menor latencia o menor carga según la última muestra.
- **Problema que resuelve:** Evita saturar un único enlace cuando existen múltiples caminos.
- **Lo que no es:** Un algoritmo de enrutamiento global óptimo. Es decisión local.
- **Podría o no ser:** Implementado manteniendo un histórico de latencia por par en el mapa de calor.

---

## 17. Forwarding zero-copy

- **Lo que sí es:** El router nunca lee ni copia el payload; solo toca los 8 bytes de control y deja que el NIC o el bus muevan el resto.
- **Lo que hace:** El paquete se despacha en un solo ciclo de decisión sin pasar por la CPU central.
- **Problema que resuelve:** Reduce latencia y consumo de energía drásticamente.
- **Lo que no es:** Real en el prototipo Rust actual. Es una meta de implementación en hardware o kernel.
- **Podría o no ser:** Aproximado en software con `sendfile`, `io_uring` o memoria mapeada.

---

## 18. Registro de DID tipo "DNS semántico"

- **Lo que sí es:** Un servicio ligero que resuelve un DID a uno o varios puntos de entrada de la red.
- **Lo que hace:** Permite encontrar por dónde empezar a buscar un destino sin difundir exploradores por toda la red.
- **Problema que resuelve:** Reduce la cantidad de tráfico de descubrimiento en redes grandes.
- **Lo que no es:** Un sistema centralizado de nombres. Es un índice local o federado.
- **Podría o no ser:** Distribuido como DHT, o centralizado en un nodo raíz por micelio.

---

## 19. Perfiles de energía para IoT

- **Lo que sí es:** Un DID verificado (Pase-E) puede dormir y despertar con un PoW mínimo para no gastar batería.
- **Lo que hace:** El router mantiene un registro de dispositivos de baja energía y no los descarta por silencio.
- **Problema que resuelve:** Sensores de batería no son obligados a transmitir continuamente para mantener su identidad.
- **Lo que no es:** Un protocolo de ahorro de energía de capa física. Es una política de soft-state.
- **Podría o no ser:** Combinado con ventanas de escucha programadas o beacons reducidos.

---

## 20. Co-procesador de hardware en Verilog

- **Lo que sí es:** Un módulo de FPGA o ASIC que procesa el encabezado de 8 bytes en un ciclo de reloj.
- **Lo que hace:** Decide puerto, valida PoW, aplica Z8 y encola el puntero a la memoria del payload.
- **Problema que resuelve:** Lleva el rendimiento a límites físicos con fracciones de vatio.
- **Lo que no es:** El mismo programa Rust. Es un complemento que puede dormir al procesador principal.
- **Podría o no ser:** Integrado primero como testbench en Icarus/Verilator y luego síntesis real.

---

## 21. Autenticación cero-confianza entre ELDs

- **Lo que sí es:** Validación de identidad entre entidades lógicas mediante micro-firmas criptográficas, sin depender de una autoridad de certificación central.
- **Lo que hace:** Cada DID demuestra quién es con una clave propia; el receptor verifica la firma sin preguntar a un servidor externo.
- **Problema que resuelve:** Permite que redes aisladas, vehículos o nodos en zonas sin internet se reconozcan y confíen entre sí.
- **Lo que no es:** Un sistema de certificados tradicional ni una blockchain de identidades.
- **Podría o no ser:** Implementada con par de claves Ed25519 o con derivación de llaves a partir del propio DID.

---

## 22. Degradación elegante de Tren Bala

- **Lo que sí es:** Un mecanismo de fallback automático cuando una ruta cristalizada ya no es válida.
- **Lo que hace:** Si un nodo no puede reenviar un paquete Tren Bala por el puerto cristalizado, cambia el Z8 a Explorador o Autocuración para buscar un camino alternativo sin cortar el flujo.
- **Problema que resuelve:** Mantiene el determinismo temporal mientras se recupera de fallos de enlace o de enrutador.
- **Lo que no es:** Un nuevo modo de paquete. Es una transición de estado dentro del nodo.
- **Podría o no ser:** Implementada con contadores de fallos por puerto y un umbral que dispara el cambio a explorador.

---

## 23. Sincronización de estado local con heartbeats ultraligeros

- **Lo que sí es:** Pequeños latidos periódicos entre nodos prosumidores para anunciar que siguen vivos y qué ramas locales mantienen.
- **Lo que hace:** Cada nodo envía un micro-beacon con su estado de sub-ramas, evitando que el vecino tenga que enviar ACKs por cada paquete.
- **Problema que resuelve:** Mantiene coherencia de topología local sin saturar el canal con confirmaciones.
- **Lo que no es:** Un protocolo de sincronización completo de base de datos. Es soft-state que expira si no se renueva.
- **Podría o no ser:** Integrado en el mismo paquete de presencia o como un tipo de paquete de control separado.

---

## 24. Postura de escucha activa sin broadcasts

- **Lo que sí es:** Un principio de diseño donde los nodos no anuncian su existencia masivamente; esperan a que un paquete con la intención correcta fluya hacia ellos.
- **Lo que hace:** El nodo filtra paquetes por signatura semántica y Z8, descartando lo que no le corresponde sin generar tráfico de respuesta.
- **Problema que resuelve:** Elimina broadcast storms, reduce interferencia electromagnética y ahorra energía en dispositivos de borde y vehiculares.
- **Lo que no es:** Silencio absoluto. Sigue habiendo beacons de presencia y exploradores estructurados, pero mínimos.
- **Podría o no ser:** Reforzado con un motor eBPF/XDP o con filtros de NIC que descarten en hardware lo que no coincide.

---

## 25. Decisiones probabilísticas en el router

- **Lo que sí es:** Un conjunto de ecuaciones sencillas que toman decisiones de enrutamiento o balanceo sin consultar tablas adicionales.
- **Lo que hace:** Calcula probabilidades de selección de puerto a partir de latencia, número de pares o contadores de fallos que ya existen.
- **Problema que resuelve:** Reduce la cantidad de tablas estáticas y permite comportamientos adaptativos con pocos datos.
- **Lo que no es:** Un reemplazo del mapa de calor ni de las rutas cristalizadas; esas siguen siendo tablas porque dependen de la topología real.
- **Podría o no ser:** Implementada como funciones puras en el módulo de enrutamiento y aplicada al balanceo de carga y a la exploración sin datos.

---

## 26. Hub gateway multiplataforma

- **Lo que sí es:** Un componente externo al núcleo IPv7-SIMBI que corre en una computadora y conecta dispositivos del hogar que no implementan IPv7 nativamente.
- **Lo que hace:** Recibe paquetes IPv7 de la red, los traduce a las interfaces locales del hogar (Wi-Fi, Ethernet, USB, Bluetooth, WebSocket) y viceversa; mantiene una tabla de presencia de DIDs locales.
- **Problema que resuelve:** Permite que el ecosistema crezca desde las computadoras hacia los dispositivos del hogar sin obligar a cada uno a tener un stack IPv7 completo.
- **Lo que no es:** Una modificación del protocolo de 8 bytes ni de los modos Z8. El núcleo sigue siendo mínimo y el hub es una capa de adaptación.
- **Podría o no ser:** Implementado como un proceso independiente en Rust que expone una API local y se comunica con el router IPv7 por localhost.

---

## 27. VPN ultra sencilla

- **Lo que sí es:** Una red privada virtual mínima que conecta computadoras como nodos IPv7-SIMBI y transporta paquetes entre ellas sin configuraciones complejas.
- **Lo que hace:** Crea una interfaz TUN/TAP o usa un proxy UDP para que las aplicaciones vean una red IPv7-SIMBI sin modificar el sistema operativo.
- **Problema que resuelve:** Da una utilidad real inmediata al protocolo y permite validar el núcleo antes de construir una pila completa.
- **Lo que no es:** Una VPN corporativa con cifrado complejo, control de acceso ni túneles punto a punto. Es el mínimo para interconectar nodos.
- **Podría o no ser:** Implementada como un binario adicional que levanta un adaptador de red virtual o un proxy SOCKS/HTTP local.

---

## 28. Interfaz TUN/TAP multiplataforma

- **Lo que sí es:** Un adaptador de red virtual que expone IPv7-SIMBI como una interfaz de red más en el sistema operativo.
- **Lo que hace:** Permite que cualquier aplicación use la red sin saber que existe IPv7 debajo, usando `wintun` en Windows, `utun` en macOS y `/dev/net/tun` en Linux.
- **Problema que resuelve:** Convierte el router en una VPN transparente para aplicaciones y servicios existentes.
- **Lo que no es:** Un driver de kernel ni una implementación real de capa 2. Es un puente usuario-sistema.
- **Podría o no ser:** Implementado con crates como `tun` o `libtuntap`, o con las librerías oficiales de cada sistema.

---

## 29. Criptografía ligera estilo Noise

- **Lo que sí es:** Un perfil criptográfico mínimo para autenticar y cifrar entre DIDs, inspirado en la simplicidad de WireGuard.
- **Lo que hace:** Usa Curve25519 para intercambio de claves, ChaCha20-Poly1305 para cifrado autenticado, y el patrón Noise IK o XX para handshakes.
- **Problema que resuelve:** Protege la privacidad y autenticidad sin arrastrar la complejidad de TLS, IPsec o certificados X.509.
- **Lo que no es:** Cifrado de cebolla ni anonimato total. Es cifrado punto a punto entre nodos conocidos por DID.
- **Podría o no ser:** Implementado con crates como `snow` para Noise o `chacha20poly1305` para cifrado directo.

---

## 30. NAT traversal

- **Lo que sí es:** Un mecanismo para que dos nodos IPv7-SIMBI se encuentren y se comuniquen aunque estén detrás de routers domésticos con NAT.
- **Lo que hace:** Usa STUN para descubrir direcciones públicas, TURN como relay de emergencia y ICE para probar combinaciones de candidatos.
- **Problema que resuelve:** Evita que los usuarios tengan que abrir puertos manualmente para conectar dispositivos.
- **Lo que no es:** Una red P2P perfecta sin relays. Si el NAT es simétrico agresivo, se necesitará un relay mínimo.
- **Podría o no ser:** Integrado en el proceso del nodo o delegado a un helper externo ligero.

---

## 31. Descubrimiento mesh sin coordenador central

- **Lo que sí es:** Un mecanismo para que los nodos descubran a otros nodos por DID sin depender de un servidor central como en Tailscale.
- **Lo que hace:** Cada nodo anuncia su presencia y DID a sus pares conocidos; la información se propaga de forma limitada por saltos, similar a un DHT muy ligero.
- **Problema que resuelve:** Permite conectar dispositivos de una misma identidad o círculo de confianza sin autoridad central.
- **Lo que no es:** Un DHT global como el de Yggdrasil ni un sistema de nombres DNS. Es descubrimiento local o federado.
- **Podría o no ser:** Implementado con micro-beacons de presencia y retransmisión controlada entre pares.

---

## 32. Aceleración con eBPF/XDP o io_uring

- **Lo que sí es:** Uso de mecanismos del kernel de Linux para acelerar el forwarding de paquetes IPv7-SIMBI.
- **Lo que hace:** eBPF/XDP filtra y desvía paquetes antes de que lleguen al espacio de usuario; io_uring reduce las syscalls de envío/recepción UDP.
- **Problema que resuelve:** Lleva el rendimiento a límites cercanos al hardware sin complicar el código del núcleo.
- **Lo que no es:** Obligatorio para el prototipo. Es una optimización posterior, solo en Linux.
- **Podría o no ser:** Agregado como una implementación alternativa del router para servidores o gateways de alto tráfico.
