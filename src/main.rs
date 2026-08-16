use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::io::{self, BufRead};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

mod config;
mod packet;
mod router;
mod stun;
mod tun;
mod ui;

use config::*;
use packet::*;
use router::*;

fn finish_packet(mut pkt: Packet, key: Option<&[u8; 32]>) -> Packet {
    if let Some(k) = key {
        pkt.payload = encrypt_payload(&pkt.payload, k);
        pkt.header.length = pkt.payload.len() as u16;
    }
    pkt
}

/// Resuelve "host:puerto" aceptando nombres DNS ademas de direcciones IP.
fn resolve_addr(spec: &str) -> Option<SocketAddr> {
    spec.parse().ok().or_else(|| spec.to_socket_addrs().ok().and_then(|mut it| it.next()))
}

fn run_demo() {
    let (log_tx, log_rx) = mpsc::channel::<String>();

    let configs = vec![
        ("A", "127.0.0.1:9000", vec![(2u8, "127.0.0.1:9001")], vec![(0usize, 2u8)], vec![("9", vec![2u8])]),
        ("B", "127.0.0.1:9001", vec![(5u8, "127.0.0.1:9002")], vec![(1usize, 5u8)], vec![("99", vec![5u8])]),
        ("C", "127.0.0.1:9002", vec![(1u8, "127.0.0.1:9003")], vec![(2usize, 1u8)], vec![("99X", vec![1u8])]),
        ("D", "127.0.0.1:9003", vec![], vec![], vec![("99X", vec![0u8])]),
    ];

    for (id, bind, peers, routes, heat) in configs {
        let mut peers_map = HashMap::new();
        for (port, addr) in peers {
            peers_map.insert(port, addr.parse().unwrap());
        }
        let mut routes_arr = [0xFFu8; 16];
        for (idx, port) in routes {
            routes_arr[idx] = port;
        }
        let heat_map = Arc::new(Mutex::new(HashMap::new()));
        for (pref, ports) in heat {
            heat_map.lock().unwrap().insert(pref.to_string(), ports);
        }
        let cfg = NodeConfig {
            id: id.to_string(),
            bind: bind.parse().unwrap(),
            nonce: 0xA5,
            iot_mask: 0x00,
            unverified_mask: 0xFF,
            peers: peers_map,
            dynamic: Arc::new(Mutex::new(HashMap::new())),
            routes: routes_arr,
            heat: heat_map,
            whitelist: ["IoT-Luz"].iter().map(|s| s.to_string()).collect(),
            down_ports: HashSet::new(),
            secret: now(),
            latencies: Arc::new(Mutex::new(HashMap::new())),
            last_seen: Arc::new(Mutex::new(HashMap::new())),
            is_tracker: false,
            tracker_registry: Arc::new(Mutex::new(HashMap::new())),
            key: None,
            seen: Arc::new(Mutex::new(HashMap::new())),
            seen_window: 60_000,
            verbose: true,
            stats: Arc::new(Stats::default()),
        };
        let tx = log_tx.clone();
        let (data_tx, _data_rx) = mpsc::channel::<Vec<u8>>();
        let (_ctrl_tx, ctrl_rx) = mpsc::channel::<(Packet, SocketAddr)>();
        thread::spawn(move || node_loop(cfg, tx, data_tx, ctrl_rx));
    }

    let sender = UdpSocket::bind("127.0.0.1:9100").expect("sender bind");
    thread::sleep(Duration::from_millis(300));

    println!("=== IPv7-SIMBI Red Real (UDP localhost) ===\n");

    let mut explorer = Packet::new("", "99X", b"hola", Z_EXPLORADOR, false, 0);
    explorer.header.pow_signature = 0xA5;
    sender.send_to(&serialize(&explorer), "127.0.0.1:9000").unwrap();
    println!("[driver] Explorador enviado a A\n");
    thread::sleep(Duration::from_millis(500));

    let mut tren = Packet::new("", "99X", b"stream-0", Z_TREN_BALA, true, 0);
    tren.header.pow_signature = 0xA5;
    sender.send_to(&serialize(&tren), "127.0.0.1:9000").unwrap();
    println!("[driver] Tren Bala enviado a A\n");
    thread::sleep(Duration::from_millis(500));

    let mut m = Packet::new("", "*", b"live", Z_MULTICAST, true, 0);
    m.header.pow_signature = 0xA5;
    sender.send_to(&serialize(&m), "127.0.0.1:9000").unwrap();
    println!("[driver] Multicast enviado a A\n");

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while std::time::Instant::now() < deadline {
        while let Ok(msg) = log_rx.try_recv() {
            println!("{}", msg);
        }
        thread::sleep(Duration::from_millis(50));
    }
    while let Ok(msg) = log_rx.try_recv() {
        println!("{}", msg);
    }
    std::process::exit(0);
}

fn run_node(id: &str, bind: &str) {
    let (log_tx, log_rx) = mpsc::channel::<String>();
    let log_path = env::var("IPV7_LOG").unwrap_or_else(|_| format!("ipv7-simbi-{}.log", id));
    let recent = Arc::new(Mutex::new(VecDeque::<String>::new()));
    let recent_log = recent.clone();
    thread::spawn(move || {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();
        while let Ok(msg) = log_rx.recv() {
            println!("{}", msg);
            if let Some(ref mut f) = file {
                let _ = writeln!(f, "{}", msg);
            }
            let mut r = recent_log.lock().unwrap();
            if r.len() == 40 {
                r.pop_front();
            }
            r.push_back(msg);
        }
    });
    let mut peers_map = HashMap::new();
    if let Ok(p) = env::var("IPV7_PEERS") {
        for part in p.split(';') {
            let part = part.trim();
            if part.is_empty() { continue; }
            if let Some((port, addr)) = part.split_once(':') {
                match (port.parse::<u8>(), resolve_addr(addr)) {
                    (Ok(port), Some(addr)) => {
                        peers_map.insert(port, addr);
                    }
                    _ => eprintln!("[config] par inválido en IPV7_PEERS: {}", part),
                }
            }
        }
    }
    let mut routes_arr = [0xFFu8; 16];
    if let Ok(r) = env::var("IPV7_ROUTES") {
        for part in r.split(';') {
            let part = part.trim();
            if part.is_empty() { continue; }
            if let Some((idx, port)) = part.split_once(':') {
                if let (Ok(idx), Ok(port)) = (idx.parse::<usize>(), port.parse::<u8>()) {
                    if idx < 16 { routes_arr[idx] = port; }
                }
            }
        }
    }
    let heat_map = Arc::new(Mutex::new(HashMap::new()));
    if let Ok(h) = env::var("IPV7_HEAT") {
        for part in h.split(';') {
            let part = part.trim();
            if part.is_empty() { continue; }
            if let Some((pref, ports)) = part.rsplit_once(':') {
                let ports: Vec<u8> = ports.split(',').filter_map(|s| s.trim().parse().ok()).collect();
                if !ports.is_empty() { heat_map.lock().unwrap().insert(pref.to_string(), ports); }
            }
        }
    }
    let whitelist: HashSet<String> = env::var("IPV7_WHITELIST")
        .unwrap_or_default()
        .split(';')
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let parse_hex = |s: String| -> u8 {
        u8::from_str_radix(&s.replace("0x", ""), 16).unwrap_or(0xA5)
    };
    let nonce = env::var("IPV7_NONCE").map(parse_hex).unwrap_or(0xA5);
    let iot_mask = env::var("IPV7_IOT_MASK").map(parse_hex).unwrap_or(0x00);
    let unverified_mask = env::var("IPV7_UNVERIFIED_MASK").map(parse_hex).unwrap_or(0xFF);
    let down_ports: HashSet<u8> = env::var("IPV7_DOWN_PORTS")
        .unwrap_or_default()
        .split(';')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    let public_addr = if let Ok(s) = env::var("IPV7_STUN_SERVER") {
        resolve_addr(&s).and_then(|srv| stun::discover(srv).ok())
    } else {
        None
    };
    if let Some(addr) = public_addr {
        println!("[stun] dirección pública descubierta: {}", addr);
    }

    // El log por paquete cuesta mas que el reenvio: solo con IPV7_VERBOSE.
    let verbose = env::var("IPV7_VERBOSE").map(|v| v != "0").unwrap_or(false);
    let stats = Arc::new(Stats::default());

    let psk = env::var("IPV7_PSK").unwrap_or_default();
    let key = if psk.is_empty() { None } else { Some(derive_key(&psk)) };
    let is_tracker = env::var("IPV7_TRACKER").is_ok();
    let dynamic = Arc::new(Mutex::new(HashMap::new()));
    let heat_ref = heat_map.clone();
    let last_seen = Arc::new(Mutex::new(HashMap::new()));
    let static_ports: HashSet<u8> = peers_map.keys().copied().collect();
    let cfg = NodeConfig {
        id: id.to_string(),
        bind: bind.parse().expect("bind"),
        nonce,
        iot_mask,
        unverified_mask,
        peers: peers_map,
        dynamic: dynamic.clone(),
        routes: routes_arr,
        heat: heat_map,
        whitelist,
        down_ports,
        secret: now(),
        latencies: Arc::new(Mutex::new(HashMap::new())),
        last_seen: last_seen.clone(),
        is_tracker,
        tracker_registry: Arc::new(Mutex::new(HashMap::new())),
        key,
        seen: Arc::new(Mutex::new(HashMap::new())),
        seen_window: 60_000,
        verbose,
        stats: stats.clone(),
    };
    let chat_dst = env::var("IPV7_CHAT_DST").ok();
    let chat_id = cfg.id.clone();
    let chat_bind = cfg.bind;
    let chat_nonce = cfg.nonce;
    let chat_z8 = env::var("IPV7_CHAT_Z8").ok().and_then(|s| s.parse().ok()).unwrap_or(Z_EXPLORADOR);
    let tracker_id = chat_id.clone();
    let tracker_nonce = chat_nonce;
    let tracker_dst = env::var("IPV7_TUNNEL_DST").unwrap_or_default();
    let heat_dst = tracker_dst.clone();
    let hello_id = chat_id.clone();
    let hello_nonce = chat_nonce;
    let tracker_addr = env::var("IPV7_TRACKER_ADDR").ok();
    let tun_id = cfg.id.clone();
    let tun_nonce = cfg.nonce;
    let tun_router = cfg.bind;
    let tunnel_peer = env::var("IPV7_TUNNEL_PEER").ok();
    let tunnel_bind = env::var("IPV7_TUNNEL_BIND").ok();
    // Panel local de estado. IPV7_UI=0 lo apaga.
    let ui_port: Option<u16> = match env::var("IPV7_UI").as_deref() {
        Ok("0") | Ok("off") | Ok("no") => None,
        Ok(v) => v.parse().ok().or(Some(7777)),
        Err(_) => Some(7777),
    };
    let tunnel_dst = env::var("IPV7_TUNNEL_DST").ok();
    let ping_peers: Vec<(u8, SocketAddr)> = cfg.peers.iter().map(|(p, a)| (*p, *a)).collect();
    let ping_dynamic = dynamic.clone();
    let ping_last = last_seen.clone();
    let ping_id = cfg.id.clone();
    let ping_nonce = cfg.nonce;
    let ui_peers = ping_peers.clone();
    let ui_latencies = cfg.latencies.clone();
    println!("[{}] IPv7-SIMBI router en {} - Ctrl+C para salir", id, bind);
    if let Some(port) = ui_port {
        ui::start(
            port,
            ui::UiState {
                id: id.to_string(),
                bind: bind.to_string(),
                tun: tunnel_bind.clone().unwrap_or_else(|| "sin tunel".to_string()),
                peers: ui_peers,
                encrypted: key.is_some(),
                stats: stats.clone(),
                latencies: ui_latencies,
                log: recent,
            },
        );
    }
    let (data_tx, data_rx) = mpsc::channel::<Vec<u8>>();
    let (control_tx, control_rx) = mpsc::channel::<(Packet, SocketAddr)>();
    let tx = log_tx.clone();
    thread::spawn(move || node_loop(cfg, tx, data_tx, control_rx));
    if let Some(dst) = chat_dst {
        thread::spawn(move || {
            let sender = UdpSocket::bind("0.0.0.0:0").expect("chat bind");
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                let line = line.unwrap_or_default();
                if line.is_empty() { continue; }
                let mut pkt = Packet::new(&chat_id, &dst, line.as_bytes(), chat_z8, false, 0);
                pkt.header.pow_signature = chat_nonce;
                let pkt = finish_packet(pkt, key.as_ref());
                if let Err(e) = sender.send_to(&serialize(&pkt), chat_bind) {
                    eprintln!("chat send error: {}", e);
                }
            }
        });
    }
    if let Some(t_addr) = tracker_addr {
        let t_addr = resolve_addr(&t_addr).expect("IPV7_TRACKER_ADDR inválida");
        thread::spawn(move || {
            let sender = UdpSocket::bind("0.0.0.0:0").expect("tracker client bind");
            loop {
                let reg_payload = public_addr.as_ref().map(|a| a.to_string().into_bytes()).unwrap_or_default();
                let mut reg = Packet::new(&tracker_id, "", &reg_payload, Z_REGISTER, false, 0);
                reg.header.pow_signature = tracker_nonce;
                let reg = finish_packet(reg, key.as_ref());
                if let Err(e) = sender.send_to(&serialize(&reg), t_addr) {
                    eprintln!("tracker register error: {}", e);
                }
                if !tracker_dst.is_empty() {
                    let mut lookup = Packet::new(&tracker_id, &tracker_dst, b"", Z_LOOKUP, false, 0);
                    lookup.header.pow_signature = tracker_nonce;
                    let lookup = finish_packet(lookup, key.as_ref());
                    if let Err(e) = sender.send_to(&serialize(&lookup), t_addr) {
                        eprintln!("tracker lookup error: {}", e);
                    }
                }
                thread::sleep(Duration::from_secs(5));
            }
        });
    }
    if let (Some(tun_bind), Some(tun_dst)) = (tunnel_bind, tunnel_dst) {
        let parse_ip = |s: &str| -> (u8, u8, u8, u8) {
            let mut it = s.split('.').filter_map(|p| p.parse().ok());
            (it.next().unwrap_or(10), it.next().unwrap_or(0), it.next().unwrap_or(0), it.next().unwrap_or(2))
        };
        let tun_addr = env::var("IPV7_TUN_ADDR").as_deref().map(parse_ip).unwrap_or((10, 0, 0, 2));
        let tun_netmask = env::var("IPV7_TUN_NETMASK").as_deref().map(parse_ip).unwrap_or((255, 255, 255, 0));
        let tun_dest = env::var("IPV7_TUN_DEST").as_deref().map(parse_ip).unwrap_or((10, 0, 0, 1));
        let tun_mtu = env::var("IPV7_TUN_MTU").ok().and_then(|s| s.parse().ok()).unwrap_or(1400);
        tun::start_tun(&tun_bind, tun_addr, tun_netmask, tun_dest, tun_mtu, tun_router, tun_id, tun_dst, tun_nonce, data_rx, key);
        loop {
            thread::sleep(Duration::from_millis(50));
        }
    }
    thread::spawn(move || {
        let sender = UdpSocket::bind("0.0.0.0:0").expect("ping bind");
        let timeout = 15_000u64;
        loop {
            thread::sleep(Duration::from_secs(5));
            let ts = now();
            let payload = format!("ping:{}", ts);
            for (_port, addr) in &ping_peers {
                let mut pkt = Packet::new(&ping_id, "", payload.as_bytes(), Z_PRUEBA_SERVICIO, false, 0);
                pkt.header.pow_signature = ping_nonce;
                let pkt = finish_packet(pkt, key.as_ref());
                if let Err(e) = sender.send_to(&serialize(&pkt), *addr) {
                    eprintln!("ping send error to {}: {}", addr, e);
                }
            }
            let mut d = ping_dynamic.lock().unwrap();
            let mut seen = ping_last.lock().unwrap();
            let dyn_addrs: Vec<(u8, SocketAddr)> = d.iter().map(|(p, a)| (*p, *a)).collect();
            let mut expired = Vec::new();
            for (port, addr) in &dyn_addrs {
                if let Some(t) = seen.get(port) {
                    if ts.saturating_sub(*t) > timeout {
                        expired.push(*port);
                    } else {
                        let mut pkt = Packet::new(&ping_id, "", payload.as_bytes(), Z_PRUEBA_SERVICIO, false, 0);
                        pkt.header.pow_signature = ping_nonce;
                        let pkt = finish_packet(pkt, key.as_ref());
                        if let Err(e) = sender.send_to(&serialize(&pkt), *addr) {
                            eprintln!("ping send error to {}: {}", addr, e);
                        }
                    }
                }
            }
            for port in expired {
                d.remove(&port);
                seen.remove(&port);
                eprintln!("[heartbeat] peer dinámico {} expirado", port);
            }
        }
    });
    let tunnel_out = tunnel_peer.as_deref().and_then(resolve_addr).map(|peer| {
        let sock = UdpSocket::bind("0.0.0.0:0").expect("tunnel out bind");
        (sock, peer)
    });
    loop {
        while let Ok(payload) = data_rx.try_recv() {
            if let Some((ref out, peer)) = tunnel_out {
                let _ = out.send_to(&payload, peer);
            }
            if tunnel_peer.is_none() {
                if let Ok(s) = std::str::from_utf8(&payload) {
                    if let Ok(addr) = s.trim().parse::<SocketAddr>() {
                        let mut d = dynamic.lock().unwrap();
                        let mut port = 0u8;
                        while d.contains_key(&port) || static_ports.contains(&port) {
                            port = port.wrapping_add(1);
                        }
                        d.insert(port, addr);
                        if !heat_dst.is_empty() {
                            heat_ref.lock().unwrap().insert(heat_dst.clone(), vec![port]);
                        }
                        println!("[dynamic] peer agregado en puerto {} -> {} (DID {})", port, addr, heat_dst);
                        let mut hello = Packet::new(&hello_id, &heat_dst, b"holepunch", Z_HELLO, false, 0);
                        hello.header.pow_signature = hello_nonce;
                        let hello = finish_packet(hello, key.as_ref());
                        let _ = control_tx.send((hello, addr));
                        continue;
                    }
                }
                println!("[data] {}", String::from_utf8_lossy(&payload));
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn run_send(to: &str) {
    let z8 = env::var("IPV7_SEND_Z8").ok().and_then(|s| s.parse().ok()).unwrap_or(Z_EXPLORADOR);
    let dst = env::var("IPV7_SEND_DST").unwrap_or_else(|_| "99X".to_string());
    let src = env::var("IPV7_SEND_SRC").unwrap_or_else(|_| "".to_string());
    let payload = env::var("IPV7_SEND_PAYLOAD").unwrap_or_else(|_| "hola".to_string()).into_bytes();
    let seq = env::var("IPV7_SEND_SEQ").ok().and_then(|s| s.parse().ok()).unwrap_or(0u8);
    let flex = env::var("IPV7_SEND_FLEX").ok().map(|s| s == "1" || s == "true").unwrap_or(false);
    let parse_hex = |s: String| -> u8 {
        u8::from_str_radix(&s.replace("0x", ""), 16).unwrap_or(0xA5)
    };
    let pow = env::var("IPV7_SEND_POW").map(parse_hex).unwrap_or(0xA5);
    let route = env::var("IPV7_SEND_ROUTE").ok().and_then(|s| s.parse().ok()).unwrap_or(0u8);
    let psk = env::var("IPV7_PSK").unwrap_or_default();
    let key = if psk.is_empty() { None } else { Some(derive_key(&psk)) };

    let mut pkt = Packet::new(&src, &dst, &payload, z8, flex, seq);
    pkt.header.pow_signature = pow;
    pkt.header.route_index = route;
    let pkt = finish_packet(pkt, key.as_ref());
    let socket = UdpSocket::bind("0.0.0.0:0").expect("sender bind");
    socket.send_to(&serialize(&pkt), to).unwrap();
    println!("[driver] paquete IPv7-SIMBI enviado a {}", to);
}

fn print_help() {
    println!("IPv7-SIMBI - red overlay P2P con TUN/VPN");
    println!();
    println!("Uso:");
    println!("  ipv7_simbi                    inicia el nodo con ipv7-simbi.conf");
    println!("  ipv7_simbi --demo             ejecuta el demo de localhost");
    println!("  ipv7_simbi --help             muestra esta ayuda");
    println!("  ipv7_simbi --config <ruta>    usa otro archivo de configuracion");
    println!("  ipv7_simbi --send <addr>      envia un paquete de prueba");
    println!();
    println!("Doble click en ipv7_simbi.exe requiere ipv7-simbi.conf en la misma carpeta.");
    println!();
    println!("Variables de entorno con prioridad sobre el archivo de configuracion:");
    println!("  IPV7_NODE_ID, IPV7_BIND, IPV7_PEERS, IPV7_HEAT, IPV7_PSK,");
    println!("  IPV7_STUN_SERVER, IPV7_TRACKER_ADDR, IPV7_TUNNEL_DST,");
    println!("  IPV7_TUN_DEVICE, IPV7_TUN_ADDR, IPV7_LOG, IPV7_UI, IPV7_VERBOSE");
    println!();
    println!("El panel de estado queda en http://127.0.0.1:7777 (IPV7_UI=0 lo apaga).");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut config_path = "ipv7-simbi.conf".to_string();
    let mut demo = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--config" | "-c" => {
                i += 1;
                if i < args.len() {
                    config_path = args[i].clone();
                }
            }
            "--demo" => demo = true,
            "--send" => {
                i += 1;
                if i < args.len() {
                    return run_send(&args[i]);
                }
            }
            _ => {}
        }
        i += 1;
    }
    if demo {
        run_demo();
        return;
    }
    let _cfg = set_defaults_from_file(&config_path);
    if env::var("IPV7_SEND_TO").is_ok() {
        run_send(&env::var("IPV7_SEND_TO").unwrap());
        return;
    }
    if let (Ok(id), Ok(bind)) = (env::var("IPV7_NODE_ID"), env::var("IPV7_BIND")) {
        #[cfg(target_os = "windows")]
        if env::var("IPV7_TUN_DEVICE").is_ok() && std::fs::metadata("wintun.dll").is_err() {
            eprintln!("[ERROR] wintun.dll no encontrado.");
            eprintln!("Descargalo desde https://www.wintun.net y colocalo junto al .exe");
            return;
        }
        run_node(&id, &bind);
    } else {
        eprintln!("[ERROR] No se encontro configuracion valida en '{}'", config_path);
        eprintln!("Copia ipv7-simbi.conf.example a ipv7-simbi.conf, editalo y volve a ejecutar.");
        print_help();
    }
}
