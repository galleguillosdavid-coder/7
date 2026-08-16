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

Para NAVEGAR por internet a traves del notebook (nodo B):
8. En el notebook, en PowerShell como administrador:  .\gateway.ps1
   Activa el NAT de Windows (ICS) desde el Wi-Fi hacia el adaptador ipv7.
   Si ICS cambia la IP del adaptador, reinicia ipv7_simbi.exe o corre:
   netsh interface ip set address name="ipv7" static 10.0.0.1 255.255.255.0
9. En el PC (A), manda todo el trafico por el tunel:
   route add 0.0.0.0 mask 0.0.0.0 10.0.0.1 metric 1
   Para revertirlo:  route delete 0.0.0.0 mask 0.0.0.0 10.0.0.1
10. Verifica en el PC:  ping 8.8.8.8   y abre una web.

Para desactivar el NAT en el notebook:  .\gateway.ps1 -Off
