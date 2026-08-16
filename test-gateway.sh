#!/usr/bin/env bash
# Prueba de navegacion: el nodo A sale a internet a traves del tunel
# IPv7-SIMBI usando al nodo B como gateway con NAT.
# Requiere root (usa network namespaces).
set -euo pipefail

BIN=${BIN:-$(dirname "$0")/target/release/ipv7_simbi}
WORK=$(mktemp -d)
UPLINK=${UPLINK:-$(ip route show default | awk '{print $5; exit}')}
cleanup() {
    ip netns pids nsA 2>/dev/null | xargs -r kill 2>/dev/null || true
    ip netns pids nsB 2>/dev/null | xargs -r kill 2>/dev/null || true
    ip netns del nsA 2>/dev/null || true
    ip netns del nsB 2>/dev/null || true
    ip link del vethH 2>/dev/null || true
    iptables -t nat -D POSTROUTING -s 10.20.0.0/24 -o "$UPLINK" -j MASQUERADE 2>/dev/null || true
    iptables -D FORWARD -i vethH -j ACCEPT 2>/dev/null || true
    iptables -D FORWARD -o vethH -j ACCEPT 2>/dev/null || true
    rm -rf "$WORK"
}
trap cleanup EXIT

ip netns add nsA
ip netns add nsB

# enlace nsA <-> nsB (la "LAN" por donde viaja el tunel)
ip link add vethA type veth peer name vethB
ip link set vethA netns nsA
ip link set vethB netns nsB
ip -n nsA addr add 10.10.0.1/24 dev vethA
ip -n nsB addr add 10.10.0.2/24 dev vethB
ip -n nsA link set vethA up
ip -n nsB link set vethB up
ip -n nsA link set lo up
ip -n nsB link set lo up

# enlace nsB <-> host (la salida a internet del nodo B)
ip link add vethH type veth peer name vethW
ip link set vethW netns nsB
ip addr add 10.20.0.1/24 dev vethH
ip link set vethH up
ip -n nsB addr add 10.20.0.2/24 dev vethW
ip -n nsB link set vethW up
ip -n nsB route add default via 10.20.0.1
sysctl -qw net.ipv4.ip_forward=1
iptables -t nat -C POSTROUTING -s 10.20.0.0/24 -o "$UPLINK" -j MASQUERADE 2>/dev/null ||
    iptables -t nat -A POSTROUTING -s 10.20.0.0/24 -o "$UPLINK" -j MASQUERADE
# el host puede tener FORWARD en DROP (p.ej. con Docker instalado)
iptables -I FORWARD 1 -i vethH -j ACCEPT
iptables -I FORWARD 1 -o vethH -j ACCEPT

cat > "$WORK/A.conf" <<EOF
IPV7_NODE_ID = "A"
IPV7_BIND = "0.0.0.0:9001"
IPV7_PSK = "clave-de-prueba"
IPV7_PEERS = "2:10.10.0.2:9002"
IPV7_HEAT = "B:2"
IPV7_TUNNEL_DST = "B"
IPV7_TUNNEL_BIND = "ipv7"
IPV7_TUN_ADDR = "10.0.0.1"
IPV7_TUN_NETMASK = "255.255.255.0"
IPV7_TUN_DEST = "10.0.0.2"
IPV7_TUN_MTU = "1400"
IPV7_LOG = "$WORK/A.log"
EOF

cat > "$WORK/B.conf" <<EOF
IPV7_NODE_ID = "B"
IPV7_BIND = "0.0.0.0:9002"
IPV7_PSK = "clave-de-prueba"
IPV7_PEERS = "2:10.10.0.1:9001"
IPV7_HEAT = "A:2"
IPV7_TUNNEL_DST = "A"
IPV7_TUNNEL_BIND = "ipv7"
IPV7_TUN_ADDR = "10.0.0.2"
IPV7_TUN_NETMASK = "255.255.255.0"
IPV7_TUN_DEST = "10.0.0.1"
IPV7_TUN_MTU = "1400"
IPV7_LOG = "$WORK/B.log"
EOF

ip netns exec nsA "$BIN" --config "$WORK/A.conf" > "$WORK/A.out" 2>&1 &
ip netns exec nsB "$BIN" --config "$WORK/B.conf" > "$WORK/B.out" 2>&1 &
sleep 3

# nodo B actua de gateway: forwarding + NAT del rango del tunel
ip netns exec nsB sysctl -qw net.ipv4.ip_forward=1
ip netns exec nsB iptables -t nat -A POSTROUTING -s 10.0.0.0/24 -o vethW -j MASQUERADE
ip netns exec nsB iptables -A FORWARD -i ipv7 -o vethW -j ACCEPT
ip netns exec nsB iptables -A FORWARD -i vethW -o ipv7 -m state --state RELATED,ESTABLISHED -j ACCEPT

# nodo A manda todo su trafico por el tunel, salvo la ruta al propio peer
ip -n nsA route replace default dev ipv7

echo "--- rutas nodo A ---"
ip -n nsA route

echo "--- ping 8.8.8.8 por el tunel ---"
ip netns exec nsA ping -c 3 -W 3 8.8.8.8 || true

echo "--- HTTP a example.com por el tunel ---"
WEB_IP=$(getent ahostsv4 example.com | awk '{print $1; exit}')
echo "example.com -> $WEB_IP"
if ip netns exec nsA curl -s --max-time 20 --resolve "example.com:80:$WEB_IP" -o "$WORK/page.html" -w "http_code=%{http_code}\n" http://example.com/; then
    head -c 200 "$WORK/page.html"; echo
    echo "RESULTADO: OK - navegacion HTTP real a traves del tunel IPv7-SIMBI"
    rc=0
else
    echo "RESULTADO: FALLO en la peticion HTTP"
    rc=1
fi
exit $rc
