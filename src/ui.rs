//! Panel de estado minimo: un servidor HTTP de la biblioteca estandar que
//! sirve una sola pagina y un JSON con los contadores del nodo.

use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::router::Stats;

pub(crate) struct UiState {
    pub(crate) id: String,
    pub(crate) bind: String,
    pub(crate) tun: String,
    pub(crate) peers: Vec<(u8, SocketAddr)>,
    pub(crate) encrypted: bool,
    pub(crate) stats: Arc<Stats>,
    pub(crate) latencies: Arc<Mutex<HashMap<u8, u64>>>,
    pub(crate) log: Arc<Mutex<VecDeque<String>>>,
}

const PAGE: &str = r#"<!doctype html><html lang="es"><meta charset="utf-8">
<title>IPv7-SIMBI</title><meta name="viewport" content="width=device-width,initial-scale=1">
<style>
body{background:#0f1115;color:#d7dae0;font:14px system-ui,sans-serif;margin:0;padding:24px}
h1{font-size:16px;letter-spacing:.14em;text-transform:uppercase;color:#7dd3a0;margin:0 0 18px}
.g{display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:10px;margin-bottom:18px}
.c{background:#171a21;border-radius:8px;padding:12px}
.k{font-size:11px;text-transform:uppercase;letter-spacing:.08em;color:#7c828d}
.v{font-size:20px;margin-top:4px}
pre{background:#171a21;border-radius:8px;padding:12px;margin:0;max-height:40vh;overflow:auto;white-space:pre-wrap}
#dot{color:#f0a}
</style>
<h1>IPv7-SIMBI <span id="dot">&#9679;</span> <span id="id"></span></h1>
<div class="g" id="g"></div>
<pre id="log"></pre>
<script>
const card=(k,v)=>`<div class="c"><div class="k">${k}</div><div class="v">${v}</div></div>`;
async function tick(){
  try{
    const s=await (await fetch('/api')).json();
    document.getElementById('id').textContent=s.id+' @ '+s.bind;
    document.getElementById('dot').style.color='#7dd3a0';
    document.getElementById('g').innerHTML=
      card('tunel',s.tun)+card('cifrado',s.encrypted?'si':'no')+
      card('pares',s.peers.map(p=>p.port+' '+p.addr+(p.rtt>=0?' ('+p.rtt+'ms)':'')).join('<br>')||'-')+
      card('recibidos',s.rx)+card('reenviados',s.tx)+card('entregados',s.local)+
      card('descartados',s.dropped)+card('trafico',s.mb+' MB');
    document.getElementById('log').textContent=s.log.join('\n');
  }catch(e){document.getElementById('dot').style.color='#f0a';}
}
tick();setInterval(tick,1000);
</script></html>"#;

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn snapshot(state: &UiState) -> String {
    let s = &state.stats;
    let lat = state.latencies.lock().unwrap();
    let peers: Vec<String> = state
        .peers
        .iter()
        .map(|(port, addr)| {
            let rtt = lat.get(port).map(|v| *v as i64).unwrap_or(-1);
            format!(r#"{{"port":{},"addr":"{}","rtt":{}}}"#, port, addr, rtt)
        })
        .collect();
    drop(lat);
    let log: Vec<String> = state
        .log
        .lock()
        .unwrap()
        .iter()
        .map(|l| format!("\"{}\"", esc(l)))
        .collect();
    let mb = (s.bytes_rx.load(Ordering::Relaxed) + s.bytes_tx.load(Ordering::Relaxed)) as f64 / 1e6;
    format!(
        r#"{{"id":"{}","bind":"{}","tun":"{}","encrypted":{},"peers":[{}],"rx":{},"tx":{},"local":{},"dropped":{},"mb":{:.2},"log":[{}]}}"#,
        esc(&state.id),
        esc(&state.bind),
        esc(&state.tun),
        state.encrypted,
        peers.join(","),
        s.rx.load(Ordering::Relaxed),
        s.tx.load(Ordering::Relaxed),
        s.local.load(Ordering::Relaxed),
        s.dropped.load(Ordering::Relaxed),
        mb,
        log.join(",")
    )
}

fn serve(mut stream: TcpStream, state: &UiState) {
    let mut line = String::new();
    if BufReader::new(&stream).read_line(&mut line).is_err() {
        return;
    }
    let (ctype, body) = if line.starts_with("GET /api") {
        ("application/json", snapshot(state))
    } else {
        ("text/html; charset=utf-8", PAGE.to_string())
    };
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        ctype,
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body.as_bytes());
}

/// Levanta el panel. Solo escucha en loopback: es una consola local, no un
/// servicio de red.
pub(crate) fn start(port: u16, state: UiState) {
    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[ui] no se pudo abrir el panel en el puerto {}: {}", port, e);
            return;
        }
    };
    println!("[ui] panel en http://127.0.0.1:{}", port);
    let state = Arc::new(state);
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            serve(stream, &state);
        }
    });
}
