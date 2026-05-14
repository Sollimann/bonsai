//! Visualize-only state attached to every `BT<A, B>` when the feature is on.
//! Bundling these fields into one struct keeps `bt.rs` free of per-field
//! `#[cfg(feature = "visualize")]` attributes — only the single `telemetry`
//! field on `BT` carries the gate.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::telemetry::TickTrace;
use crate::tracer::NodeMeta;

/// RAII handle that shuts down the visualizer acceptor thread when dropped.
///
/// The acceptor thread spawned by [`spawn_server`](crate::visualizer_server::spawn_server)
/// blocks on `TcpListener::accept`, so a flag alone is not enough to stop it.
/// On drop we (1) set the shared shutdown flag, (2) self-connect to the bound
/// port to unblock the parked `accept`, then (3) join the acceptor thread so
/// the `TcpListener` is dropped and the port is released before `Drop` returns.
/// Without the join step, a caller that immediately rebinds the same port can
/// still hit `AddrInUse` because the acceptor thread hasn't yet been scheduled
/// to break out of its loop.
///
/// Wrapped in `Arc` inside [`TelemetryState`] so clones of a telemetry-attached
/// `BT` share one acceptor — only the last surviving clone runs the teardown.
#[derive(Debug)]
pub(crate) struct AcceptorGuard {
    shutdown: Arc<AtomicBool>,
    /// Address to self-connect to in order to wake the parked acceptor. If the
    /// listener was bound to an unspecified address (`0.0.0.0` / `::`), this
    /// is rewritten to the matching loopback before storage, because some
    /// platforms reject `connect()` to an unspecified address.
    wake_addr: SocketAddr,
    /// `Option` because `Drop` only has `&mut self` and `JoinHandle::join`
    /// consumes — we `.take()` it. `Some` for the entire lifetime up to Drop.
    acceptor_handle: Option<JoinHandle<()>>,
}

impl AcceptorGuard {
    pub(crate) fn new(shutdown: Arc<AtomicBool>, bound_addr: SocketAddr, acceptor_handle: JoinHandle<()>) -> Self {
        Self {
            shutdown,
            wake_addr: wake_addr_for(bound_addr),
            acceptor_handle: Some(acceptor_handle),
        }
    }
}

impl Drop for AcceptorGuard {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        // Short timeout so a misbehaving firewall can't stall BT drop.
        let _ = std::net::TcpStream::connect_timeout(&self.wake_addr, Duration::from_millis(500));
        if let Some(handle) = self.acceptor_handle.take() {
            // Ignore Err (acceptor thread panicked) — there's nothing useful
            // to do in a destructor; the listener still drops on unwind.
            let _ = handle.join();
        }
    }
}

/// Rewrite unspecified bind addresses (`0.0.0.0`, `::`) to the matching
/// loopback so `connect_timeout` succeeds on every platform.
fn wake_addr_for(addr: SocketAddr) -> SocketAddr {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    if !addr.ip().is_unspecified() {
        return addr;
    }
    let loopback = match addr {
        SocketAddr::V4(_) => IpAddr::V4(Ipv4Addr::LOCALHOST),
        SocketAddr::V6(_) => IpAddr::V6(Ipv6Addr::LOCALHOST),
    };
    SocketAddr::new(loopback, addr.port())
}

#[derive(Clone, Debug, Default)]
pub(crate) struct TelemetryState {
    /// Preorder node metadata, computed once at `BT::new`. Used by
    /// `RecordingTracer` to advance past unvisited subtrees in O(1).
    pub node_metas: Vec<NodeMeta>,
    /// Channel sender for shipping `TickTrace`s to the broadcaster thread.
    /// `None` until [`BT::with_telemetry_at`](crate::BT::with_telemetry_at)
    /// attaches a sender; cleared back to `None` when the broadcaster drops.
    pub sender: Option<SyncSender<TickTrace>>,
    /// Shared cleanup handle for the visualizer acceptor thread. Held behind
    /// `Arc` so cloning a telemetry-attached `BT` shares the guard across
    /// clones — only the last drop runs teardown.
    pub acceptor_guard: Option<Arc<AcceptorGuard>>,
    /// Number of `TickTrace`s dropped because the channel was full. Reset on
    /// `BT::reset_bt`. Useful for diagnosing slow visualizer clients.
    pub dropped_traces: u64,
    /// Reusable buffer for the recording trace. Held for the BT's lifetime;
    /// `tick_recording` clears it on entry, preserving capacity. Avoids one
    /// `HashMap` allocation per tick on the hot path.
    pub trace_buffer: TickTrace,
}

impl TelemetryState {
    pub fn new(node_metas: Vec<NodeMeta>) -> Self {
        Self {
            node_metas,
            sender: None,
            acceptor_guard: None,
            dropped_traces: 0,
            trace_buffer: TickTrace::default(),
        }
    }
}
