#!/usr/bin/env bash
# Prueba end-to-end de la VPN: dos nodos IPv7-SIMBI en espacios de red
# separados, unidos por un veth, hacen ping a traves del tunel TUN.
# Requiere root (usa network namespaces).
set -euo pipefail

BIN=${BIN:-$(dirname "$0")/target/release/ipv7_simbi}
WORK=$(mktemp -d)
cleanup() {
    ip netns pids nsA 2>/dev/null | xargs -r kill 2>/dev/null || true
    ip netns pids nsB 2>/dev/null | xargs -r kill 2>/dev/null || true
    ip netns del nsA 2>/dev/null || true
    ip netns del nsB 2>/dev/null || true
    rm -rf "$WORK"
}
trap cleanup EXIT

ip netns add nsA
ip netns add nsB
ip link add vethA type veth peer name vethB
ip link set vethA netns nsA
ip link set vethB netns nsB
ip -n nsA addr add 10.10.0.1/24 dev vethA
ip -n nsB addr add 10.10.0.2/24 dev vethB
ip -n nsA link set vethA up
ip -n nsB link set vethB up
ip -n nsA link set lo up
ip -n nsB link set lo up

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

echo "--- adaptadores ---"
ip -n nsA addr show ipv7 | sed -n '1,3p'
ip -n nsB addr show ipv7 | sed -n '1,3p'

echo "--- ping A(10.0.0.1) -> B(10.0.0.2) ---"
if ip netns exec nsA ping -c 4 -W 2 10.0.0.2; then
    echo "RESULTADO: OK - trafico IP real sobre el tunel IPv7-SIMBI"
    rc=0
else
    echo "RESULTADO: FALLO"
    rc=1
fi

echo "--- log nodo A (ultimas lineas) ---"
tail -5 "$WORK/A.out" || true
echo "--- log nodo B (ultimas lineas) ---"
tail -5 "$WORK/B.out" || true
exit $rc
