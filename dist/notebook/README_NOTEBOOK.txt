IPv7-SIMBI - Paquete para el notebook (nodo B)

1. Copia toda esta carpeta al notebook.
2. En el notebook, abre CMD o PowerShell y ejecuta: ipconfig
   Busca la direccion IPv4 del Wi-Fi (ej: 192.168.2.123).
3. En el PC principal (nodo A), edita ipv7-simbi.conf y pon esa IP en:
   IPV7_PEERS = "2:IP_DEL_NOTEBOOK:9002"
4. Edita en ESTE notebook el archivo ipv7-simbi.conf y reemplaza IP_DEL_PC_A por la IP del PC (A).
   La IP del PC es: 192.168.2.198
5. En el PC principal, inicia IPv7-SIMBI como administrador:
   - Click derecho en run-first.bat > Ejecutar como administrador
6. En el notebook, inicia IPv7-SIMBI como administrador:
   - Click derecho en run-first.bat > Ejecutar como administrador
7. Prueba con ping desde el PC:  ping 10.0.0.1
   Y desde el notebook:          ping 10.0.0.2

Nota: si quieres navegar por internet a traves del tuyo, el PC (A) debe tener una ruta que
reenvie el trafico del tun a internet. Esto requiere compartir conexion o activar forwarding.
