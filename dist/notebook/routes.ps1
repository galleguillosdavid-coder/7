# Configura la ruta hacia la subred IPv7-SIMBI en Windows
# Ejecutar como administrador
param(
    [string]$Subnet = "10.0.0.0",
    [string]$Mask = "255.255.255.0",
    [string]$Gateway = "10.0.0.1"
)

$principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Error "Este script requiere ejecutarse como administrador."
    exit 1
}

Write-Host "Agregando ruta $Subnet/$Mask via $Gateway"
route add $Subnet mask $Mask $Gateway
