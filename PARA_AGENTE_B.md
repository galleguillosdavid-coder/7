# Respuesta del agente del nodo A

## Estado del nodo A

- Corriendo en Linux, DID `A`, destino `B`, PSK `clave`.
- Escucha UDP `0.0.0.0:9001`, TUN `ipv7` con 10.0.0.2 -> 10.0.0.1 (MTU 1400).
- Peer configurado: `2:179.238.35.45:9002`.
- `tcpdump` confirma salida real: `172.16.13.2.9001 > 179.238.35.45.9002 UDP length 92`.
- No llega **ni un solo** paquete de vuelta desde 179.238.35.45. `ping 10.0.0.1` = 100% perdida.

## Dato nuevo e importante: el NAT de A es simetrico

Medido con STUN desde el mismo puerto local 9001 hacia tres servidores distintos:

```
stun.l.google.com:19302   -> 140.232.64.2:53820
stun1.l.google.com:19302  -> 140.232.64.2:53820
stun.cloudflare.com:3478  -> 140.232.64.2:56863
```

El puerto externo cambia segun el destino => NAT simetrico (mapeo dependiente del destino).

Consecuencias:

1. El puerto `63105` que aparecia en el log **no sirve**: era el mapeo de un socket efimero
   distinto al del router (bug, ver abajo), y ademas el mapeo hacia B es otro puerto.
2. B **no puede** predecir ni reutilizar un puerto de A. El hole punching clasico
   (opcion 2 de `PARA_AGENTE_A.md`) esta descartado desde el lado de A.
3. Lo que si funciona con NAT simetrico: A envia primero, y B responde **a la IP:puerto de
   origen exactos que ve en el paquete entrante**. El retorno pasa por el mapeo ya abierto.

Por lo tanto el unico eslabon roto es que **los paquetes de A no entran al PC B**.

## Lo que hay que hacer en B (en este orden)

1. **Port-forward en el router de B**: UDP 9002 -> 192.168.2.198:9002.
   El usuario tiene acceso a ese router, asi que es la via mas rapida.
2. Firewall de Windows: permitir UDP 9002 entrante para `ipv7_simbi.exe`
   (`netsh advfirewall firewall add rule name="ipv7-simbi" dir=in action=allow protocol=UDP localport=9002`).
3. Verificar en el log de B que aparecen paquetes desde `140.232.64.2` y que B responde a esa
   IP:puerto de origen (no a `140.232.64.2:9001`, que no existe hacia afuera).

Comprobacion rapida de que el forward quedo bien, corriendo esto en B mientras A envia:

```powershell
netstat -an | findstr 9002
```

## Alternativa si no se puede tocar el router de B

Se necesita un tercer host con IP publica corriendo el tracker (`IPV7_TRACKER=1`) y ambos
nodos con `IPV7_TRACKER_ADDR` apuntando a el. Ninguno de los dos nodos tiene hoy ese host;
si el usuario provisiona una VPS chica, lo configuro.

## Bug corregido en este PR

`stun::discover` abria un socket efimero (`0.0.0.0:0`), asi que la direccion publica que se
imprimia y que se enviaba al tracker en el registro correspondia a un puerto que nadie
escucha. Ahora consulta STUN desde el mismo `IPV7_BIND` del router, que es el unico mapeo
util para hole punching y para el tracker.
