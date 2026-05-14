#![allow(dead_code)]

use std::io::{self, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::telemetry::{TickTrace, VISUALIZER_HTML};

/// Slowloris budget: drop a connection that hasn't delivered headers in this long.
const READ_TIMEOUT: Duration = Duration::from_secs(5);

/// Wedged-client budget: a single broadcast `send()` must complete in this long.
/// Combined with channel size 1024, the tick path can drop frames after at most
/// `WRITE_TIMEOUT * num_clients` of broadcaster blockage.
const WRITE_TIMEOUT: Duration = Duration::from_secs(2);

/// Peek buffer for HTTP-vs-WS dispatch. Single-packet localhost requests fit
/// comfortably; oversize headers are misclassified as HTTP (the client retries).
const PEEK_BUF_BYTES: usize = 1024;

struct Client {
    ws: tungstenite::WebSocket<TcpStream>,
    addr: std::net::SocketAddr,
}

/// Spawn the broadcaster thread + accept loop. Returns once the listener is
/// bound (so the caller can fail fast on `EADDRINUSE`); the actual loops run
/// detached on background threads.
///
/// `tree_definition_json` is cloned for each new WS handshake so late-joining
/// clients get the static layout before any tick frames.
///
/// `rx` is moved into the broadcaster thread; dropping the matching `Sender`
/// causes the broadcaster to exit cleanly.
pub fn spawn_server(listener: TcpListener, tree_definition_json: String, rx: Receiver<TickTrace>) -> io::Result<()> {
    // Listener arrives pre-bound from the caller. Stdlib `TcpListener` is
    // already blocking by default; no `set_nonblocking(false)` needed.

    let clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));
    // Shared shutdown flag: broadcaster sets it on exit; acceptor checks it between
    // connections. The acceptor will not terminate until the NEXT connection arrives
    // after shutdown — this is a known, documented limitation.
    let shutdown = Arc::new(AtomicBool::new(false));

    // Acceptor thread. The `move` closure takes ownership of
    // `tree_definition_json` directly — it's not used elsewhere in
    // `spawn_server`, so cloning it would be redundant.
    let clients_acceptor = Arc::clone(&clients);
    let shutdown_acceptor = Arc::clone(&shutdown);
    std::thread::Builder::new()
        .name("bonsai-viz-acceptor".into())
        .spawn(move || {
            for stream in listener.incoming() {
                let stream = match stream {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                if shutdown_acceptor.load(Ordering::Relaxed) {
                    break; // drop stream — connection closes immediately on our side
                }
                stream.set_nodelay(true).ok();
                stream.set_read_timeout(Some(READ_TIMEOUT)).ok();
                stream.set_write_timeout(Some(WRITE_TIMEOUT)).ok();
                handle_connection(stream, &clients_acceptor, &tree_definition_json);
            }
        })?;

    // Broadcaster thread
    let clients_broadcaster = Arc::clone(&clients);
    std::thread::Builder::new()
        .name("bonsai-viz-broadcaster".into())
        .spawn(move || {
            while let Ok(trace) = rx.recv() {
                // Wrap the per-trace work in catch_unwind so a malformed trace
                // (or a tungstenite bug) can't kill the broadcaster thread and
                // silently starve all connected clients.
                let clients = &clients_broadcaster;
                if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let Ok(json) = serde_json::to_string(&trace) else {
                        return;
                    };
                    let mut guard = clients.lock().expect("clients mutex poisoned");
                    let mut i = 0;
                    while i < guard.len() {
                        // Clone the String once per client — with 0–2 clients typical this is fine.
                        match guard[i].ws.send(tungstenite::Message::Text(json.clone())) {
                            Ok(()) => i += 1,
                            Err(_) => {
                                // Client dropped or write timed out — evict O(1).
                                let _ = guard.swap_remove(i);
                            }
                        }
                    }
                }))
                .is_err()
                {
                    eprintln!("bonsai-viz broadcaster: panic while broadcasting tick trace; continuing");
                }
            }
            // All senders dropped — signal acceptor to stop on its next wakeup.
            shutdown.store(true, Ordering::Relaxed);
        })?;

    Ok(())
}

fn handle_connection(stream: TcpStream, clients: &Mutex<Vec<Client>>, tree_json: &str) {
    // Peek without consuming — tungstenite::accept needs to re-read the headers.
    let mut peek = [0u8; PEEK_BUF_BYTES];
    let n = match stream.peek(&mut peek) {
        Ok(n) => n,
        Err(_) => return,
    };
    let head = std::str::from_utf8(&peek[..n]).unwrap_or("");
    let is_ws = head.lines().any(|l| {
        let lower = l.to_ascii_lowercase();
        lower.starts_with("upgrade:") && lower.contains("websocket")
    });

    if is_ws {
        if let Ok(mut ws) = tungstenite::accept(stream) {
            // First frame: the static tree definition.
            if ws.send(tungstenite::Message::Text(tree_json.to_owned())).is_err() {
                return;
            }
            let addr = ws
                .get_ref()
                .peer_addr()
                .unwrap_or_else(|_| ([0u8, 0, 0, 0], 0u16).into());
            let mut guard = clients.lock().expect("clients mutex poisoned");
            guard.push(Client { ws, addr });
        }
    } else {
        serve_http(stream, head);
    }
}

fn serve_http(mut stream: TcpStream, head: &str) {
    let path = head.split_whitespace().nth(1).unwrap_or("/");
    let is_root = path == "/" || path.starts_with("/?");
    let (status, body): (&str, &[u8]) = if is_root {
        ("200 OK", VISUALIZER_HTML.as_bytes())
    } else {
        ("404 Not Found", b"not found")
    };
    let header = format!(
        "HTTP/1.1 {status}\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Cache-Control: no-store\r\n\
         Connection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
}
