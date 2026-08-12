# Distribuir IPv7-SIMBI VPN

## Paquete Windows

Crear una carpeta con:

- `ipv7_simbi.exe` (compilado con `cargo build --release`)
- `wintun.dll` (descargar de https://www.wintun.net)
- `setup.bat`
- `run-first.bat`
- `start.bat`
- `routes.ps1`
- `ipv7-simbi.conf.example`
- `VPN.md`
- `INSTALL.md`

El usuario hace doble click en `setup.bat` para configurar, y luego doble click en `run-first.bat` para iniciar la VPN.

## Paquete Linux

Crear una carpeta con:

- `ipv7_simbi` (compilado con `cargo build --release`)
- `setup.sh`
- `run-first.sh`
- `start.sh`
- `routes.sh`
- `ipv7-simbi.conf.example`
- `ipv7-simbi.service`
- `VPN.md`
- `INSTALL.md`

El usuario corre `./setup.sh`, luego `./run-first.sh`.

## Compilar release

```bash
cargo build --release
```

Windows binario: `target/release/ipv7_simbi.exe`  
Linux binario: `target/release/ipv7_simbi`
