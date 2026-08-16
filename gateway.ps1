# Convierte este equipo Windows en la salida a internet del tunel IPv7-SIMBI.
# Usa Internet Connection Sharing (ICS), que es el unico NAT integrado en
# Windows cliente. Ejecutar en PowerShell COMO ADMINISTRADOR.
#
#   .\gateway.ps1                 comparte la conexion con mas trafico hacia internet
#   .\gateway.ps1 -Uplink "Wi-Fi" elige la interfaz de salida
#   .\gateway.ps1 -Off            desactiva el compartir
param(
    [string]$Uplink,
    [string]$Tun = "ipv7",
    [switch]$Off
)

$ErrorActionPreference = "Stop"

$admin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
    [Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $admin) {
    Write-Error "Ejecuta este script como Administrador."
    exit 1
}

if (-not $Uplink) {
    $Uplink = (Get-NetIPConfiguration | Where-Object { $_.IPv4DefaultGateway -ne $null } |
        Select-Object -First 1).InterfaceAlias
}
if (-not $Uplink) {
    Write-Error "No se encontro una interfaz con salida a internet. Usa -Uplink '<nombre>'."
    exit 1
}

Write-Host "Uplink: $Uplink   Tunel: $Tun"

# El servicio de ICS debe estar disponible
Set-Service -Name SharedAccess -StartupType Manual
Start-Service -Name SharedAccess

$netShare = New-Object -ComObject HNetCfg.HNetShare
$conns = @($netShare.EnumEveryConnection)

function Get-Conn($alias) {
    foreach ($c in $conns) {
        if ($netShare.NetConnectionProps.Invoke($c).Name -eq $alias) { return $c }
    }
    return $null
}

$pub = Get-Conn $Uplink
$priv = Get-Conn $Tun
if (-not $pub) { Write-Error "No existe la conexion '$Uplink'"; exit 1 }
if (-not $priv) { Write-Error "No existe la conexion '$Tun'. Levanta IPv7-SIMBI primero."; exit 1 }

$pubCfg = $netShare.INetSharingConfigurationForINetConnection.Invoke($pub)
$privCfg = $netShare.INetSharingConfigurationForINetConnection.Invoke($priv)

if ($Off) {
    $pubCfg.DisableSharing()
    $privCfg.DisableSharing()
    Write-Host "Compartir desactivado."
    exit 0
}

# 0 = publica (con internet), 1 = privada (la red que recibe el NAT)
$pubCfg.EnableSharing(0)
$privCfg.EnableSharing(1)

Write-Host "ICS activado: $Uplink -> $Tun"
Write-Host "OJO: ICS reasigna la IP del adaptador $Tun (normalmente 192.168.137.1)."
Write-Host "Reinicia ipv7_simbi.exe para que vuelva a fijar la IP del tunel, o ajustala a mano:"
Write-Host "  netsh interface ip set address name=`"$Tun`" static 10.0.0.1 255.255.255.0"
Write-Host ""
Write-Host "En el otro nodo, para enviar todo el trafico por el tunel:"
Write-Host "  route add 0.0.0.0 mask 0.0.0.0 10.0.0.1 metric 1"
