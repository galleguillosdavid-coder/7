use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use chacha20poly1305::ChaCha20Poly1305;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, Nonce};

pub(crate) const Z_TREN_BALA: u8 = 1;
pub(crate) const Z_MULTICAST: u8 = 2;
pub(crate) const Z_EXPLORADOR: u8 = 3;
pub(crate) const Z_BIT_FLIP: u8 = 4;
pub(crate) const Z_PRUEBA_SERVICIO: u8 = 5;
pub(crate) const Z_AUTOCURACION: u8 = 6;
pub(crate) const Z_LOCKDOWN: u8 = 7;
pub(crate) const Z_REGISTER: u8 = 8;
pub(crate) const Z_LOOKUP: u8 = 9;
pub(crate) const Z_RESOLVE: u8 = 10;
pub(crate) const Z_HELLO: u8 = 11;

/// Bytes de nonce que se anteponen al texto cifrado.
pub(crate) const NONCE_LEN: usize = 12;

#[derive(Clone, Debug)]
pub(crate) struct Ipv7Header {
    pub(crate) z8: u8,
    pub(crate) flex: bool,
    pub(crate) route_index: u8,
    pub(crate) pow_signature: u8,
    pub(crate) anchor: u16,
    pub(crate) length: u16,
    pub(crate) sequence: u8,
}

impl Ipv7Header {
    pub(crate) fn pack(&self) -> u64 {
        let mut h: u64 = 0;
        h |= self.z8 as u64 & 0x7;
        h |= (self.flex as u64) << 3;
        h |= (self.route_index as u64 & 0xF) << 4;
        h |= (self.pow_signature as u64) << 8;
        h |= (self.anchor as u64) << 16;
        h |= (self.length as u64) << 32;
        h |= (self.sequence as u64) << 48;
        h
    }

    pub(crate) fn unpack(raw: u64) -> Self {
        Ipv7Header {
            z8: (raw & 0x7) as u8,
            flex: ((raw >> 3) & 1) != 0,
            route_index: ((raw >> 4) & 0xF) as u8,
            pow_signature: ((raw >> 8) & 0xFF) as u8,
            anchor: ((raw >> 16) & 0xFFFF) as u16,
            length: ((raw >> 32) & 0xFFFF) as u16,
            sequence: ((raw >> 48) & 0xFF) as u8,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Packet {
    pub(crate) header: Ipv7Header,
    pub(crate) did_src: String,
    pub(crate) did_dst: String,
    pub(crate) payload: Vec<u8>,
    pub(crate) hops_remaining: u8,
}

impl Packet {
    pub(crate) fn new(did_src: &str, did_dst: &str, payload: &[u8], z8: u8, flex: bool, sequence: u8) -> Self {
        Packet {
            header: Ipv7Header {
                z8,
                flex,
                route_index: 0,
                pow_signature: 0,
                anchor: (fnv1a_32(did_dst.as_bytes()) >> 16) as u16,
                length: payload.len() as u16,
                sequence,
            },
            did_src: did_src.to_string(),
            did_dst: did_dst.to_string(),
            payload: payload.to_vec(),
            hops_remaining: 16,
        }
    }
}

pub(crate) fn fnv1a_32(data: &[u8]) -> u32 {
    let mut h: u32 = 0x811c9dc5;
    for &b in data {
        h ^= b as u32;
        h = h.wrapping_mul(0x01000193);
    }
    h
}

pub(crate) fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

pub(crate) fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

pub(crate) fn verify_pow(pow: u8, nonce: u8, mask: u8) -> bool {
    ((pow ^ nonce) & mask) == 0
}

pub(crate) fn pos_sign(node: &str, did: &str, salt: u64) -> u32 {
    let mut buf = Vec::new();
    buf.extend_from_slice(node.as_bytes());
    buf.extend_from_slice(did.as_bytes());
    buf.extend_from_slice(&salt.to_le_bytes());
    fnv1a_32(&buf)
}

pub(crate) fn derive_key(psk: &str) -> [u8; 32] {
    let mut key = [0u8; 32];
    for (i, b) in psk.bytes().enumerate() {
        key[i % 32] ^= b;
    }
    let h = fnv1a_32(psk.as_bytes());
    key[28..32].copy_from_slice(&h.to_be_bytes());
    key
}

/// Nonce unico por paquete: 4 bytes de semilla del proceso y contador de 8.
/// Viaja en claro delante del texto cifrado.
fn next_nonce() -> [u8; NONCE_LEN] {
    static SEED: OnceLock<u32> = OnceLock::new();
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seed = *SEED.get_or_init(|| {
        let mut buf = now().to_be_bytes().to_vec();
        buf.extend_from_slice(&std::process::id().to_be_bytes());
        fnv1a_32(&buf)
    });
    let mut n = [0u8; NONCE_LEN];
    n[0..4].copy_from_slice(&seed.to_be_bytes());
    n[4..12].copy_from_slice(&COUNTER.fetch_add(1, Ordering::Relaxed).to_be_bytes());
    n
}

pub(crate) fn encrypt_payload(plaintext: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let raw = next_nonce();
    let nonce = Nonce::from_slice(&raw).to_owned();
    match cipher.encrypt(&nonce, plaintext) {
        Ok(v) => {
            let mut out = raw.to_vec();
            out.extend_from_slice(&v);
            out
        }
        Err(_) => plaintext.to_vec(),
    }
}

pub(crate) fn decrypt_payload(ciphertext: &[u8], key: &[u8; 32]) -> Option<Vec<u8>> {
    if ciphertext.len() < NONCE_LEN {
        return None;
    }
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = Nonce::from_slice(&ciphertext[..NONCE_LEN]).to_owned();
    cipher.decrypt(&nonce, &ciphertext[NONCE_LEN..]).ok()
}

/// Identificador anti-replay: el nonce transmitido cuando el payload viene
/// cifrado, o un hash del encabezado y del contenido cuando viaja en claro.
pub(crate) fn replay_id(pkt: &Packet, encrypted: bool) -> u64 {
    if encrypted && pkt.payload.len() >= NONCE_LEN {
        let mut id = [0u8; 8];
        id.copy_from_slice(&pkt.payload[4..12]);
        return u64::from_be_bytes(id) ^ ((fnv1a_32(&pkt.payload[..4]) as u64) << 32);
    }
    let mut buf = pkt.header.pack().to_be_bytes().to_vec();
    buf.extend_from_slice(pkt.did_src.as_bytes());
    buf.extend_from_slice(&pkt.payload);
    let lo = fnv1a_32(&buf) as u64;
    buf.reverse();
    ((fnv1a_32(&buf) as u64) << 32) | lo
}

pub(crate) fn serialize(pkt: &Packet) -> Vec<u8> {
    serialize_with(pkt, &pkt.payload)
}

/// Serializa el paquete sustituyendo el payload, sin copiar el original.
pub(crate) fn serialize_with(pkt: &Packet, payload: &[u8]) -> Vec<u8> {
    let mut header = pkt.header.pack();
    header &= !(0xFFFFu64 << 32);
    header |= (payload.len() as u64 & 0xFFFF) << 32;
    let mut v = Vec::with_capacity(payload.len() + pkt.did_src.len() + pkt.did_dst.len() + 13);
    v.extend_from_slice(&header.to_be_bytes());
    v.push(pkt.did_src.len() as u8);
    v.extend_from_slice(pkt.did_src.as_bytes());
    v.push(pkt.did_dst.len() as u8);
    v.extend_from_slice(pkt.did_dst.as_bytes());
    v.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    v.extend_from_slice(payload);
    let crc = crc16_ccitt(&v);
    v.extend_from_slice(&crc.to_be_bytes());
    v
}

pub(crate) fn parse(buf: &[u8]) -> Option<Packet> {
    if buf.len() < 8 {
        return None;
    }
    let raw = u64::from_be_bytes(buf[0..8].try_into().unwrap());
    let header = Ipv7Header::unpack(raw);
    let mut p = 8;
    let read_str = |buf: &[u8], p: &mut usize| -> Option<String> {
        if *p >= buf.len() {
            return None;
        }
        let len = buf[*p] as usize;
        *p += 1;
        if *p + len > buf.len() {
            return None;
        }
        let s = String::from_utf8_lossy(&buf[*p..*p + len]).to_string();
        *p += len;
        Some(s)
    };
    let did_src = read_str(buf, &mut p)?;
    let did_dst = read_str(buf, &mut p)?;
    if p + 2 > buf.len() {
        return None;
    }
    let plen = u16::from_be_bytes([buf[p], buf[p + 1]]) as usize;
    p += 2;
    if p + plen + 2 > buf.len() {
        return None;
    }
    let data_end = p + plen;
    let stored = u16::from_be_bytes(buf[data_end..data_end + 2].try_into().unwrap());
    if crc16_ccitt(&buf[..data_end]) != stored {
        return None;
    }
    let payload = buf[p..data_end].to_vec();
    Some(Packet {
        header,
        did_src,
        did_dst,
        payload,
        hops_remaining: 16,
    })
}
