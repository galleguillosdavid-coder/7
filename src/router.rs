use std::collections::{HashMap, HashSet};
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex, mpsc};

use crate::packet::*;

pub(crate) struct NodeConfig {
    pub(crate) id: String,
    pub(crate) bind: SocketAddr,
    pub(crate) nonce: u8,
    pub(crate) iot_mask: u8,
    pub(crate) unverified_mask: u8,
    pub(crate) peers: HashMap<u8, SocketAddr>,
    pub(crate) dynamic: Arc<Mutex<HashMap<u8, SocketAddr>>>,
    pub(crate) routes: [u8; 16],
    pub(crate) heat: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    pub(crate) whitelist: HashSet<String>,
    pub(crate) down_ports: HashSet<u8>,
    pub(crate) secret: u64,
    pub(crate) latencies: Arc<Mutex<HashMap<u8, u64>>>,
    pub(crate) last_seen: Arc<Mutex<HashMap<u8, u64>>>,
    pub(crate) is_tracker: bool,
    pub(crate) tracker_registry: Arc<Mutex<HashMap<String, SocketAddr>>>,
    pub(crate) key: Option<[u8; 32]>,
    pub(crate) seen: Arc<Mutex<HashMap<u64, u64>>>,
    pub(crate) seen_window: u64,
}

fn find_in_port(cfg: &NodeConfig, from: SocketAddr) -> u8 {
    for (port, addr) in &cfg.peers {
        if *addr == from {
            return *port;
        }
    }
    for (port, addr) in cfg.dynamic.lock().unwrap().iter() {
        if *addr == from {
            return *port;
        }
    }
    0xFF
}

fn peer_addr(cfg: &NodeConfig, port: u8) -> Option<SocketAddr> {
    cfg.peers.get(&port).copied().or_else(|| cfg.dynamic.lock().unwrap().get(&port).copied())
}

fn encrypt_packet(pkt: &Packet, key: &[u8; 32]) -> Packet {
    let mut out = pkt.clone();
    out.payload = encrypt_payload(&pkt.payload, key);
    out.header.length = out.payload.len() as u16;
    out
}

fn out_pkt(cfg: &NodeConfig, pkt: &Packet) -> Packet {
    cfg.key.map(|k| encrypt_packet(pkt, &k)).unwrap_or(pkt.clone())
}

fn forward(cfg: &NodeConfig, socket: &UdpSocket, pkt: Packet, out_port: u8, log: &mpsc::Sender<String>, reason: &str) -> Option<Packet> {
    if cfg.down_ports.contains(&out_port) {
        log.send(format!("[{}] puerto {} caído: {}", cfg.id, out_port, reason)).ok();
        return Some(pkt);
    }
    if let Some(addr) = peer_addr(cfg, out_port) {
        let out = cfg.key.map(|k| encrypt_packet(&pkt, &k)).unwrap_or(pkt.clone());
        if let Err(e) = socket.send_to(&serialize(&out), addr) {
            log.send(format!("[{}] error envío a {}: {}", cfg.id, addr, e)).ok();
            return Some(pkt);
        }
        log.send(format!("[{}] {} -> puerto {} ({})", cfg.id, out.did_dst, out_port, reason)).ok();
        None
    } else {
        log.send(format!("[{}] {} entregado localmente", cfg.id, pkt.did_dst)).ok();
        None
    }
}

fn is_data(z8: u8) -> bool {
    matches!(z8, Z_TREN_BALA | Z_EXPLORADOR | Z_AUTOCURACION | Z_BIT_FLIP)
}

fn resolve_heat(cfg: &NodeConfig, did: &str) -> Option<u8> {
    let heat = cfg.heat.lock().unwrap();
    let mut best_pref: Option<&str> = None;
    let mut best_ports: Option<&Vec<u8>> = None;
    for (pref, ports) in heat.iter() {
        if did.starts_with(pref) {
            if best_pref.map(|b| b.len() < pref.len()).unwrap_or(true) {
                best_pref = Some(pref.as_str());
                best_ports = Some(ports);
            }
        }
    }
    let ports = best_ports?;
    if ports.len() == 1 {
        return Some(ports[0]);
    }
    let mut weights = Vec::with_capacity(ports.len());
    let latencies = cfg.latencies.lock().unwrap();
    for port in ports {
        let lat = latencies.get(port).copied().unwrap_or(1000);
        weights.push((1.0 / (lat as f64 + 1.0), *port));
    }
    drop(latencies);
    let total: f64 = weights.iter().map(|(w, _)| w).sum();
    let mut pick = (now() as f64) % total;
    for (w, port) in &weights {
        if pick < *w {
            return Some(*port);
        }
        pick -= *w;
    }
    Some(weights.last().map(|(_, p)| *p).unwrap_or(ports[0]))
}

fn process(cfg: &NodeConfig, socket: &UdpSocket, mut pkt: Packet, from: SocketAddr, log: &mpsc::Sender<String>, data: &mpsc::Sender<Vec<u8>>) {
    let mask = if cfg.whitelist.contains(&pkt.did_src) {
        cfg.iot_mask
    } else {
        cfg.unverified_mask
    };
    if !verify_pow(pkt.header.pow_signature, cfg.nonce, mask) {
        log.send(format!("[{}] PoW rechazado de {} para {}", cfg.id, pkt.did_src, pkt.did_dst)).ok();
        return;
    }

    let _sig = pos_sign(&cfg.id, &pkt.did_dst, cfg.secret);
    log.send(format!("[{}] PoS firmado para {} (latencia {}ms)", cfg.id, pkt.did_dst, now() % 1000)).ok();

    let in_port = find_in_port(cfg, from);
    if in_port != 0xFF {
        cfg.last_seen.lock().unwrap().insert(in_port, now());
    }
    let nonce_id = replay_id(&pkt, cfg.key.is_some());
    if let Some(key) = cfg.key {
        match decrypt_payload(&pkt.payload, &key) {
            Some(plain) => {
                pkt.header.length = plain.len() as u16;
                pkt.payload = plain;
            }
            None => {
                log.send(format!("[{}] descifrado fallido de {}", cfg.id, pkt.did_src)).ok();
                return;
            }
        }
    }
    let now_ts = now();
    let mut seen = cfg.seen.lock().unwrap();
    if let Some(ts) = seen.get(&nonce_id) {
        if now_ts.saturating_sub(*ts) < cfg.seen_window {
            log.send(format!("[{}] REPLAY detectado de {}", cfg.id, pkt.did_src)).ok();
            return;
        }
    }
    seen.insert(nonce_id, now_ts);
    if seen.len() > 8192 {
        let window = cfg.seen_window;
        seen.retain(|_, ts| now_ts.saturating_sub(*ts) < window);
    }
    drop(seen);

    if !pkt.did_dst.is_empty() && pkt.did_dst == cfg.id && is_data(pkt.header.z8) {
        log.send(format!("[{}] ENTREGADO local: {} bytes de {}", cfg.id, pkt.payload.len(), pkt.did_src)).ok();
        data.send(pkt.payload).ok();
        return;
    }

    match pkt.header.z8 {
        Z_TREN_BALA => {
            let idx = pkt.header.route_index as usize;
            if idx < 16 && cfg.routes[idx] != 0xFF {
                let port = cfg.routes[idx];
                pkt.header.route_index += 1;
                if let Some(failed) = forward(cfg, socket, pkt, port, log, "Tren Bala") {
                    let mut degraded = failed;
                    degraded.header.z8 = Z_AUTOCURACION;
                    log.send(format!("[{}] Tren Bala caído por puerto {}, degradando", cfg.id, port)).ok();
                    for (p, _) in &cfg.peers {
                        if *p != in_port && *p != port {
                            let _ = forward(cfg, socket, degraded.clone(), *p, log, "Tren Bala degradado");
                        }
                    }
                }
            } else {
                data.send(pkt.payload).ok();
                log.send(format!("[{}] ENTREGADO Tren Bala: {}", cfg.id, pkt.did_dst)).ok();
            }
        }
        Z_EXPLORADOR => {
            if pkt.hops_remaining == 0 {
                log.send(format!("[{}] Explorador TTL agotado", cfg.id)).ok();
                return;
            }
            pkt.hops_remaining -= 1;
            if let Some(port) = resolve_heat(cfg, &pkt.did_dst) {
                if port == 0 {
                    data.send(pkt.payload).ok();
                    log.send(format!("[{}] ENTREGADO Explorador: {}", cfg.id, pkt.did_dst)).ok();
                } else if port != in_port {
                    let _ = forward(cfg, socket, pkt, port, log, "Explorador gravitacional");
                } else {
                    log.send(format!("[{}] Explorador evita remolino", cfg.id)).ok();
                }
            } else {
                let peers: Vec<u8> = cfg.peers.keys().copied().filter(|p| *p != in_port).collect();
                if !peers.is_empty() {
                    let idx = (now() as usize) % peers.len();
                    let port = peers[idx];
                    let _ = forward(cfg, socket, pkt, port, log, "Explorador aleatorio");
                } else {
                    log.send(format!("[{}] Explorador sin gradiente ni pares", cfg.id)).ok();
                }
            }
        }
        Z_MULTICAST => {
            log.send(format!("[{}] Multicast delta", cfg.id)).ok();
            let mut all_ports: Vec<u8> = cfg.peers.keys().copied().collect();
            all_ports.extend(cfg.dynamic.lock().unwrap().keys().copied());
            for port in all_ports {
                if port != in_port {
                    let mut p = pkt.clone();
                    p.header.z8 = Z_TREN_BALA;
                    let _ = forward(cfg, socket, p, port, log, "Multicast");
                }
            }
        }
        Z_BIT_FLIP => {
            pkt.header.flex = !pkt.header.flex;
            if let Some(port) = resolve_heat(cfg, &pkt.did_dst) {
                let _ = forward(cfg, socket, pkt, port, log, "Bit-Flip");
            }
        }
        Z_PRUEBA_SERVICIO => {
            if pkt.did_src == cfg.id {
                if let Ok(s) = std::str::from_utf8(&pkt.payload) {
                    if let Some(ts_str) = s.strip_prefix("ping:") {
                        if let Ok(ts) = ts_str.parse::<u64>() {
                            let rtt = now().saturating_sub(ts);
                            cfg.latencies.lock().unwrap().insert(in_port, rtt);
                            log.send(format!("[{}] RTT hacia puerto {} = {} ms", cfg.id, in_port, rtt)).ok();
                        }
                    }
                }
            } else if in_port != 0xFF {
                let _ = forward(cfg, socket, pkt, in_port, log, "PoS eco");
            }
        }
        Z_AUTOCURACION => {
            let mut all_ports: Vec<u8> = cfg.peers.keys().copied().collect();
            all_ports.extend(cfg.dynamic.lock().unwrap().keys().copied());
            for port in all_ports {
                if port != in_port {
                    let _ = forward(cfg, socket, pkt.clone(), port, log, "Autocuración");
                }
            }
            if cfg.peers.is_empty() && cfg.dynamic.lock().unwrap().is_empty() {
                data.send(pkt.payload).ok();
                log.send(format!("[{}] ENTREGADO Autocuración: {}", cfg.id, pkt.did_dst)).ok();
            }
        }
        Z_LOCKDOWN => {
            log.send(format!("[{}] Lockdown activado", cfg.id)).ok();
        }
        Z_REGISTER => {
            if cfg.is_tracker {
                let addr = std::str::from_utf8(&pkt.payload)
                    .ok()
                    .and_then(|s| s.trim().parse::<SocketAddr>().ok())
                    .unwrap_or(from);
                cfg.tracker_registry.lock().unwrap().insert(pkt.did_src.clone(), addr);
                log.send(format!("[{}] REGISTRO: {} -> {}", cfg.id, pkt.did_src, addr)).ok();
            }
        }
        Z_LOOKUP => {
            if cfg.is_tracker {
                let reg = cfg.tracker_registry.lock().unwrap();
                if let Some(addr) = reg.get(&pkt.did_dst) {
                    let payload = addr.to_string().into_bytes();
                    let mut resp = Packet::new(&cfg.id, &pkt.did_src, &payload, Z_RESOLVE, false, 0);
                    resp.header.pow_signature = cfg.nonce;
                    let _ = socket.send_to(&serialize(&out_pkt(cfg, &resp)), from);
                    log.send(format!("[{}] LOOKUP {} -> {}", cfg.id, pkt.did_dst, addr)).ok();
                } else {
                    log.send(format!("[{}] LOOKUP {} no encontrado", cfg.id, pkt.did_dst)).ok();
                }
            }
        }
        Z_RESOLVE => {
            let len = pkt.payload.len();
            data.send(pkt.payload).ok();
            log.send(format!("[{}] RESOLVE recibido de {} ({} bytes)", cfg.id, pkt.did_src, len)).ok();
        }
        Z_HELLO => {
            log.send(format!("[{}] HELLO de {} en puerto {}", cfg.id, pkt.did_src, in_port)).ok();
        }
        _ => {
            data.send(pkt.payload).ok();
            log.send(format!("[{}] ENTREGADO por defecto: {}", cfg.id, pkt.did_dst)).ok();
        }
    }
}

pub(crate) fn node_loop(
    cfg: NodeConfig,
    log: mpsc::Sender<String>,
    data: mpsc::Sender<Vec<u8>>,
    control: mpsc::Receiver<(Packet, SocketAddr)>,
) {
    let socket = UdpSocket::bind(cfg.bind).expect("bind");
    socket.set_nonblocking(true).expect("nonblocking");
    log.send(format!("[{}] escuchando {}", cfg.id, cfg.bind)).ok();
    let mut buf = [0u8; 4096];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, from)) => {
                if let Some(pkt) = parse(&buf[..n]) {
                    process(&cfg, &socket, pkt, from, &log, &data);
                } else {
                    log.send(format!("[{}] paquete corrupto o CRC-16 rechazado", cfg.id)).ok();
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => {
                log.send(format!("[{}] recv error: {}", cfg.id, e)).ok();
            }
        }
        while let Ok((pkt, to)) = control.try_recv() {
            if let Err(e) = socket.send_to(&serialize(&out_pkt(&cfg, &pkt)), to) {
                eprintln!("control send error: {}", e);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

