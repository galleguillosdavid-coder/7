use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::mpsc;
use std::thread;

pub fn start_tun(
    name: &str,
    address: (u8, u8, u8, u8),
    netmask: (u8, u8, u8, u8),
    destination: (u8, u8, u8, u8),
    mtu: u16,
    router_addr: std::net::SocketAddr,
    did_src: String,
    did_dst: String,
    nonce: u8,
    data_rx: mpsc::Receiver<Vec<u8>>,
    key: Option<[u8; 32]>,
) {
    let mut config = tun::Configuration::default();
    config.tun_name(name).address(address).netmask(netmask).destination(destination).mtu(mtu).up();
    #[cfg(target_os = "linux")]
    config.platform_config(|c| {
        c.ensure_root_privileges(true);
    });
    let dev = tun::create(&config).expect("tun create");
    let (mut reader, mut writer) = dev.split();

    let sender = UdpSocket::bind("0.0.0.0:0").expect("tun sender bind");
    // Un router en 0.0.0.0 tambien escucha en loopback, que es donde el TUN
    // entrega sus paquetes.
    let send_addr = if router_addr.ip().is_unspecified() {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), router_addr.port())
    } else {
        router_addr
    };
    thread::spawn(move || {
        let mut buf = vec![0u8; mtu as usize];
        let mut warned = false;
        loop {
            match reader.read(&mut buf) {
                Ok(n) => {
                    let mut pkt = crate::Packet::new(&did_src, &did_dst, &buf[..n], crate::Z_EXPLORADOR, false, 0);
                    pkt.header.pow_signature = nonce;
                    if let Some(k) = key {
                        pkt.payload = crate::packet::encrypt_payload(&pkt.payload, &k);
                        pkt.header.length = pkt.payload.len() as u16;
                    }
                    if let Err(e) = sender.send_to(&crate::serialize(&pkt), send_addr) {
                        if !warned {
                            eprintln!("tun send error: {}", e);
                            warned = true;
                        }
                    }
                }
                Err(e) => eprintln!("tun read error: {}", e),
            }
        }
    });

    thread::spawn(move || {
        while let Ok(payload) = data_rx.recv() {
            if let Err(e) = writer.write_all(&payload) {
                eprintln!("tun write error: {}", e);
            }
        }
    });
}
