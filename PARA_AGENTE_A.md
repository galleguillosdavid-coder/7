# Instrucciones para el agente del nodo A (la otra VM)

## Contexto

- Eres el nodo A. Este PC (Windows) es el nodo B.
- B esta configurado asi:
  - DID local: B
  - DID destino: A
  - Escucha UDP: 0.0.0.0:9002
  - PSK: clave
  - TUN local: 10.0.0.1
  - TUN destino: 10.0.0.2
  - IP publica: 179.238.35.45
  - IP LAN: 192.168.2.198
- Tu VM A esta configurada asi:
  - DID local: A
  - DID destino: B
  - Escucha UDP: 0.0.0.0:9001
  - TUN local: 10.0.0.2
  - TUN destino: 10.0.0.1
  - IP publica: 140.232.64.2
  - Puerto externo STUN: 63105

## Estado actual

- B intento levantar. El TUN se creo y escucha UDP 9002.
- `ping 10.0.0.2` (desde B) falla con 100% de perdida.
- Tu VM envia a 179.238.35.45:9002, pero B no recibe nada.
- B no puede iniciar conexion hacia ti porque tu VM esta detras de NAT sin port-forwarding.

## Problema: NAT

Para que B reciba tus paquetes UDP, una de estas condiciones debe cumplirse:

1. Tu VM debe aceptar conexiones UDP entrantes en el puerto que tu proceso escucha (9001).
   - Si tu NAT no tiene port-forward, eso no es posible.
2. El router/NAT de B debe reenviar UDP 9002 a 192.168.2.198.
   - Eso requiere acceso al router de B, que el agente de B no puede hacer.
3. Usar un tracker o relay intermedio con IP publica.
   - Ninguno de los dos nodos tiene un tracker/relay publico disponible.

## Acciones que puedes tomar desde tu VM

### Opcion 1: Verificar si tu VM puede recibir UDP entrante

Ejecuta en tu terminal:

```bash
ss -uln | grep 9001
```

Si escucha, intenta un `nc` o `iperf` UDP desde fuera. Probablemente no funcione sin port-forward.

### Opcion 2: Hole punching coordinado

Para que funcione, ambos nodos deben enviar paquetes UDP al otro casi al mismo tiempo, abriendo los mapeos NAT.

- Tu VM envia a `179.238.35.45:9002` (IP publica de B).
- B envia a tu `140.232.64.2:63105` (puerto STUN).
- Si ambos NAT son "cone", los paquetes pasan.
- Si cualquiera es "simetrico" o "restricto", falla.

### Opcion 3: Provisionar un relay/tracker

Si tienes acceso a una tercera maquina con IP publica, puedes:

- Levantar un tracker UDP alli.
- Configurar `IPV7_TRACKER_ADDR` en ambos nodos para registrarse.
- Los nodos descubren sus direcciones publicas a traves del tracker.

### Opcion 4: Usar la VM como gateway y B como cliente

Si tu VM tiene internet y puedes instalar/activar IP forwarding:

1. Configura tu VM como A con gateway (usar `gateway.sh` si es Linux).
2. Configura B con `IPV7_TUN_ADDR = 10.0.0.1` y `IPV7_TUN_DEST = 10.0.0.2` (ya esta).
3. Problema: B no puede abrir puerto en su router, asi que la conexion UDP sigue siendo unilateral.

## Conclusion

Sin abrir el puerto UDP 9002 en el router de B o sin un relay/tracker publico, la conexion directa P2P no va a funcionar. B ya esta listo; el cuello de botella es el NAT de B (y posiblemente el tuyo).

Si puedes provisionar un relay o un tracker, avisa y reconfiguramos ambos nodos para usarlo.
