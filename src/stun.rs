use std::io;
use std::net::{SocketAddr, SocketAddrV4, UdpSocket};
use std::time::Duration;

use crate::packet::now;

const STUN_MAGIC_COOKIE: [u8; 4] = [0x21, 0x12, 0xa4, 0x42];
const STUN_BINDING_REQUEST: [u8; 2] = [0x00, 0x01];
const STUN_BINDING_RESPONSE: [u8; 2] = [0x01, 0x01];
const ATTR_MAPPED_ADDRESS: u16 = 0x0001;
const ATTR_XOR_MAPPED_ADDRESS: u16 = 0x0020;

/// Consulta el mapeo publico del NAT. `local_bind` debe ser la misma direccion
/// que usa el router: el mapeo de un puerto efimero distinto no sirve para
/// hole punching ni para registrarse en el tracker.
pub fn discover(stun_server: SocketAddr, local_bind: SocketAddr) -> io::Result<SocketAddr> {
    let socket = UdpSocket::bind(local_bind)?;
    socket.set_read_timeout(Some(Duration::from_secs(2)))?;

    let ts = now().to_le_bytes();
    let mut tx: [u8; 12] = [0; 12];
    tx[..ts.len().min(12)].copy_from_slice(&ts[..ts.len().min(12)]);

    let mut req = Vec::with_capacity(20);
    req.extend_from_slice(&STUN_BINDING_REQUEST);
    req.extend_from_slice(&[0x00, 0x00]);
    req.extend_from_slice(&STUN_MAGIC_COOKIE);
    req.extend_from_slice(&tx);

    socket.send_to(&req, stun_server)?;

    let mut buf = [0u8; 512];
    let (n, _from) = socket.recv_from(&mut buf)?;
    if n < 20 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "stun response too short"));
    }
    if buf[0..2] != STUN_BINDING_RESPONSE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "stun response not a binding response"));
    }
    if buf[4..8] != STUN_MAGIC_COOKIE || buf[8..20] != tx {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "stun magic/transaction mismatch"));
    }

    let attrs_start = 20;
    let msg_len = u16::from_be_bytes([buf[2], buf[3]]) as usize + attrs_start;
    let mut p = attrs_start;
    while p + 4 <= msg_len && p + 4 <= n {
        let attr_type = u16::from_be_bytes([buf[p], buf[p + 1]]);
        let attr_len = u16::from_be_bytes([buf[p + 2], buf[p + 3]]) as usize;
        let padded_len = if attr_len % 4 == 0 { attr_len } else { attr_len + (4 - attr_len % 4) };
        if p + 4 + padded_len > n {
            break;
        }
        if attr_type == ATTR_MAPPED_ADDRESS && attr_len >= 8 && buf[p + 5] == 1 {
            let port = u16::from_be_bytes([buf[p + 6], buf[p + 7]]);
            let ip = [buf[p + 8], buf[p + 9], buf[p + 10], buf[p + 11]];
            return Ok(SocketAddr::V4(SocketAddrV4::new(ip.into(), port)));
        }
        if attr_type == ATTR_XOR_MAPPED_ADDRESS && attr_len >= 8 && buf[p + 5] == 1 {
            let port_bytes = [buf[p + 6], buf[p + 7]];
            let magic = [STUN_MAGIC_COOKIE[0], STUN_MAGIC_COOKIE[1]];
            let port = u16::from_be_bytes([port_bytes[0] ^ magic[0], port_bytes[1] ^ magic[1]]);
            let mut ip = [buf[p + 8], buf[p + 9], buf[p + 10], buf[p + 11]];
            for i in 0..4 {
                ip[i] ^= STUN_MAGIC_COOKIE[i];
            }
            return Ok(SocketAddr::V4(SocketAddrV4::new(ip.into(), port)));
        }
        p += 4 + padded_len;
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "stun mapped address not found"))
}
