//! Acceptance tests for the WebSocket visualizer server
//!
//! Gated on `feature = "visualize"`. Registered in `tests.rs`.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{sync_channel, SyncSender, TrySendError};
use std::time::Duration;

use bonsai_bt::telemetry::{TickTrace, TreeDefinition};
use bonsai_bt::{Action, Behavior, Status};

/// A real, well-formed tree-definition JSON. Using a built `TreeDefinition`
/// (not `"{}"`) means `parsed["root"]["id"]` actually exists for test 4.
fn fixture_tree_json() -> String {
    let behavior: Behavior<&str> = Action("a");
    let definition = TreeDefinition::build(&behavior);
    serde_json::to_string(&definition).expect("TreeDefinition serializes")
}

/// Bind a listener on an OS-assigned port. Returns `(listener, port)`.
fn bind_localhost_random() -> (TcpListener, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind 127.0.0.1:0");
    let port = listener.local_addr().expect("local_addr").port();
    (listener, port)
}

/// Spin up a server with the fixture tree. Caller owns `tx` — drop it at end
/// of the test to let the broadcaster thread exit.
fn start_server() -> (u16, SyncSender<TickTrace>) {
    let (listener, port) = bind_localhost_random();
    let (tx, rx) = sync_channel::<TickTrace>(1024);
    bonsai_bt::spawn_server(listener, fixture_tree_json(), rx).expect("spawn_server");
    (port, tx)
}

/// Open a raw TCP stream and complete the WS handshake on it. Returns
/// `WebSocket<TcpStream>` (not `WebSocket<MaybeTlsStream<_>>` like
/// `tungstenite::connect` would) — simpler to set TCP-level timeouts on.
fn ws_connect(port: u16) -> tungstenite::WebSocket<TcpStream> {
    let stream = TcpStream::connect(("127.0.0.1", port)).expect("tcp connect");
    stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    stream.set_write_timeout(Some(Duration::from_secs(2))).unwrap();
    let url = format!("ws://127.0.0.1:{port}/");
    let (ws, _resp) = tungstenite::client::client(url, stream).expect("ws handshake");
    ws
}

/// Read a single Text frame, skipping Ping/Pong control frames. Bounded by
/// the underlying TcpStream's read_timeout.
fn read_text(ws: &mut tungstenite::WebSocket<TcpStream>) -> String {
    loop {
        match ws.read().expect("ws read") {
            tungstenite::Message::Text(s) => return s,
            tungstenite::Message::Ping(_) | tungstenite::Message::Pong(_) => continue,
            other => panic!("unexpected ws frame: {other:?}"),
        }
    }
}

#[test]
fn bind_succeeds_on_port_zero() {
    let (p1, _tx1) = start_server();
    let (p2, _tx2) = start_server();
    assert_ne!(p1, p2, "OS should pick distinct ports");
}

#[test]
fn bind_fails_on_conflicting_port() {
    let (_listener, port) = bind_localhost_random();
    let result = TcpListener::bind(format!("127.0.0.1:{port}"));
    // Windows is sometimes more permissive about duplicate binds on 127.0.0.1
    // — assert is_err rather than the specific ErrorKind for portability.
    assert!(result.is_err(), "second bind on occupied port must fail");
}

#[test]
fn http_get_root_returns_200() {
    let (port, _tx) = start_server();

    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("tcp connect");
    stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    stream.set_write_timeout(Some(Duration::from_secs(2))).unwrap();
    stream
        .write_all(b"GET / HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n")
        .unwrap();

    // Read tolerantly: serve_http writes the full response then drops the
    // stream. On Linux, dropping a TcpStream with unread data in the recv
    // buffer (the original request bytes) sends RST rather than FIN, so
    // `read_to_string` returns ConnectionReset *after* the response bytes
    // have already arrived. Accumulate what we get and stop on either Ok(0)
    // or any error.
    let mut response = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => response.extend_from_slice(&buf[..n]),
            Err(_) => break,
        }
    }
    let response = String::from_utf8(response).expect("response is utf-8");

    assert!(
        response.starts_with("HTTP/1.1 200 OK"),
        "status: {}",
        &response[..32.min(response.len())]
    );
    assert!(
        response.contains("Content-Type: text/html"),
        "Content-Type header missing"
    );
    assert!(response.contains("Cache-Control: no-store"));

    let body = response.split_once("\r\n\r\n").expect("headers/body separator").1;
    assert_eq!(
        body.len(),
        bonsai_bt::telemetry::VISUALIZER_HTML.len(),
        "body length mismatch — peek/serve_http drift?"
    );
}

#[test]
fn ws_connect_receives_tree_definition_first() {
    let (port, _tx) = start_server();
    let mut ws = ws_connect(port);

    let frame = read_text(&mut ws);
    let parsed: serde_json::Value = serde_json::from_str(&frame).expect("first frame is JSON");

    assert_eq!(parsed["root"]["id"], serde_json::json!(0));
    assert!(parsed["root"]["children"].is_array());
}

#[test]
fn ws_receives_subsequent_ticks() {
    let (port, tx) = start_server();
    let mut ws = ws_connect(port);

    // Drain the tree-definition frame.
    let _ = read_text(&mut ws);

    let mut states = HashMap::new();
    states.insert(0, Status::Success);
    tx.send(TickTrace { tick_id: 7, states }).unwrap();

    let frame = read_text(&mut ws);
    let trace: TickTrace = serde_json::from_str(&frame).expect("tick frame parses");
    assert_eq!(trace.tick_id, 7);
    assert_eq!(trace.states.get(&0), Some(&Status::Success));
}

#[test]
fn broadcaster_evicts_dead_client() {
    let (port, tx) = start_server();
    let mut alive = ws_connect(port);
    let mut dead = ws_connect(port);

    // Drain tree-definition frame from both, then drop the dead one.
    let _ = read_text(&mut alive);
    let _ = read_text(&mut dead);
    drop(dead);

    // Push 5 traces. The broadcaster's send to the dead client will fail on
    // some iteration ≤ 5 (TCP buffers may absorb the first few writes).
    for i in 1..=5u64 {
        tx.send(TickTrace {
            tick_id: i,
            states: HashMap::new(),
        })
        .unwrap();
    }

    // Surviving client must see all 5 in order.
    for expected in 1..=5u64 {
        let frame = read_text(&mut alive);
        let trace: TickTrace = serde_json::from_str(&frame).unwrap();
        assert_eq!(trace.tick_id, expected, "out-of-order or dropped on survivor");
    }
}

#[test]
#[ignore = "fragile under load"]
fn channel_full_drop_semantics() {
    let (port, tx) = start_server();
    let _slow = ws_connect(port); // never reads from this client

    // Wait for handshake to settle and the broadcaster to park on the first send.
    std::thread::sleep(Duration::from_millis(50));

    let mut full_seen = false;
    for i in 0..2_000u64 {
        match tx.try_send(TickTrace {
            tick_id: i,
            states: HashMap::new(),
        }) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                full_seen = true;
                break;
            }
            Err(TrySendError::Disconnected(_)) => panic!("broadcaster disconnected unexpectedly"),
        }
    }
    assert!(full_seen, "channel never reported Full — kernel buffer too generous?");
}

#[test]
fn broadcaster_exits_when_sender_dropped() {
    let (port, tx) = start_server();
    let mut ws = ws_connect(port);

    // Drain tree-definition frame, push 5 traces, drain them.
    let _ = read_text(&mut ws);
    for i in 1..=5u64 {
        tx.send(TickTrace {
            tick_id: i,
            states: HashMap::new(),
        })
        .unwrap();
    }
    for expected in 1..=5u64 {
        let frame = read_text(&mut ws);
        let trace: TickTrace = serde_json::from_str(&frame).unwrap();
        assert_eq!(trace.tick_id, expected);
    }

    // Drop the sender → broadcaster sees Err(Disconnected), exits, sets shutdown.
    drop(tx);
    std::thread::sleep(Duration::from_millis(100));

    // Wake the acceptor with a fresh TCP connection. It should observe shutdown
    // and break out of `for stream in listener.incoming()` *before* calling
    // handle_connection, so the connection is closed with no response.
    let mut probe = TcpStream::connect(("127.0.0.1", port)).expect("tcp connect");
    probe.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let mut buf = [0u8; 16];
    let n = probe.read(&mut buf).expect("read after shutdown");
    assert_eq!(n, 0, "acceptor should drop the probe connection at EOF");
}
