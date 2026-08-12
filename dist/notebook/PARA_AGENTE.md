# Instrucciones para el agente en el notebook (nodo B)

## Contexto

- Eres un agente Devin/Windsurf ejecutandote en un notebook Windows limpio.
- En la carpeta de trabajo hay un paquete IPv7-SIMBI: `ipv7_simbi.exe`, `wintun.dll`, `ipv7-simbi.conf`, `run-first.bat`.
- Hay otro PC en la misma LAN (PC A) con IP `192.168.2.198`.
- PC A ya tiene IPv7-SIMBI listo con `IPV7_BIND = "0.0.0.0:9006"` y `IPV7_TUNNEL_DST = "B"`.
- Este notebook es el nodo B. PC A quiere navegar por internet a traves del tuyo.

## Objetivo

1. Levantar el nodo B de IPv7-SIMBI en este notebook.
2. Crear el adaptador TUN `ipv7` con IP `10.0.0.1`.
3. Configurar el notebook para que reenvie el trafico del tuyo hacia internet (NAT/ICS).
4. Verificar que PC A pueda hacer `ping 10.0.0.1` y navegar.

## Paso 1 - Obtener la IP local del notebook

```cmd
ipconfig
```

Anota la IP del adaptador Wi-Fi (ej. `192.168.2.123`). Escribe esa IP en `ipv7-simbi.conf` del PC A manualmente, no puedes hacerlo desde aqui.

## Paso 2 - Verificar la configuracion del notebook

Edita `ipv7-simbi.conf` y asegurate de que diga exactamente lo siguiente (reemplaza `IP_DEL_PC_A` por `192.168.2.198` si aun no esta reemplazado):

```conf
IPV7_NODE_ID = "B"
IPV7_BIND = "0.0.0.0:9002"
IPV7_PSK = "clave"
IPV7_TUNNEL_DST = "A"
IPV7_TUNNEL_BIND = "ipv7"
IPV7_PEERS = "2:192.168.2.198:9006"
IPV7_TUN_DEVICE = "ipv7"
IPV7_TUN_ADDR = "10.0.0.1"
IPV7_TUN_NETMASK = "255.255.255.0"
IPV7_TUN_DEST = "10.0.0.2"
IPV7_TUN_MTU = "1400"
```

## Paso 3 - Levantar el nodo B

Ejecutar en PowerShell o CMD **como administrador**:

```cmd
run-first.bat
```

Esto crea el adaptador TUN `ipv7` e inicia el nodo B escuchando en `0.0.0.0:9002`.

Verificar que aparece:
- `ipv7 Tunnel` en `Get-NetAdapter`
- `10.0.0.1` en `ipconfig`
- El proceso `ipv7_simbi.exe` escuchando UDP en el puerto 9002 con `netstat -an | findstr 9002`.

## Paso 4 - Habilitar el reenvio de trafico (gateway)

Para que PC A navegue por internet a traves de este notebook, este equipo debe:

1. Reenviar paquetes del adaptador `ipv7` a la interfaz Wi-Fi.
2. Hacer NAT para que los paquetes privados `10.0.0.0/24` salgan con la IP publica del notebook.

Windows no incluye NAT por linea de comandos de forma sencilla. La forma directa es
ejecutar el script incluido, en PowerShell **como administrador**:

```powershell
.\gateway.ps1            # activa ICS del uplink hacia el adaptador ipv7
.\gateway.ps1 -Off       # lo revierte
```

Si prefieres hacerlo a mano, estas son las opciones:

### Opcion A - Internet Connection Sharing (ICS) - recomendada

1. Abrir `ncpa.cpl` (Conexiones de red).
2. Click derecho en el adaptador Wi-Fi > `Propiedades` > pestana `Compartir`.
3. Marcar `Permitir a otros usuarios de la red conectarse a traves de la conexion a Internet de este equipo`.
4. En `Conexion de red domestica`, seleccionar `ipv7`.
5. Aceptar.

Esto configura NAT automaticamente. Windows dara a `ipv7` una IP en el rango 192.168.137.x. Eso entrara en conflicto con la IP 10.0.0.1 de IPv7-SIMBI. Antes de compartir, cambia la IP del adaptador `ipv7` a `10.0.0.1/255.255.255.0` y la puerta de enlace a `10.0.0.2` (o deja que IPv7-SIMBI lo reconfigure al reiniciar el nodo).

### Opcion B - Activar IP Forwarding (sin NAT, no suficiente solo)

```powershell
Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters" -Name "IPEnableRouter" -Value 1
```

Requiere reiniciar. Aun asi, sin NAT el trafico volvera con la IP del notebook, no con 10.0.0.1, asi que el retorno fallara a menos que el router de la LAN tenga ruta estatica. Por eso ICS es preferible.

### Opcion C - Usar un proxy (alternativa rapida si ICS falla)

Si solo se quiere probar navegacion web, instalar un proxy HTTP en el notebook (por ejemplo `px` o `mitmproxy` si tienes Python), pero como el notebook esta limpio, esta opcion no es viable sin instalar mas software.

## Paso 5 - Verificacion

Una vez que el nodo B este arriba y el gateway configurado:

```cmd
ping 10.0.0.2
```

Debe responder (eso es el PC A).

En PC A, el usuario deberia poder:

```cmd
ping 10.0.0.1
ping 8.8.8.8
```

Si `ping 8.8.8.8` desde PC A funciona, la navegacion esta activa. Si no, revisar ICS, firewall o reglas de ruta.

## Notas de troubleshooting

- Si `ipv7_simbi.exe` no arranca, verificar que `wintun.dll` esta en la misma carpeta.
- Si el adaptador `ipv7` no aparece, ejecutar `run-first.bat` como administrador.
- Si ICS cambia la IP de `ipv7`, reinicia `ipv7_simbi.exe` o reconfigura manualmente la IP del adaptador.
- Firewall de Windows puede bloquear `ipv7_simbi.exe`; permitir acceso publico/privado si se pregunta.
