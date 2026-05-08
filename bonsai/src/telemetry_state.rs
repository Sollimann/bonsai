//! Visualize-only state attached to every `BT<A, B>` when the feature is on.
//! Bundling these fields into one struct keeps `bt.rs` free of per-field
//! `#[cfg(feature = "visualize")]` attributes — only the single `telemetry`
//! field on `BT` carries the gate.

use std::sync::mpsc::SyncSender;

use crate::telemetry::TickTrace;
use crate::tracer::NodeMeta;

#[derive(Clone, Debug, Default)]
pub(crate) struct TelemetryState {
    /// Preorder node metadata, computed once at `BT::new`. Used by
    /// `RecordingTracer` to advance past unvisited subtrees in O(1).
    pub node_metas: Vec<NodeMeta>,
    /// Channel sender for shipping `TickTrace`s to the broadcaster thread.
    /// `None` until [`BT::with_telemetry_at`](crate::BT::with_telemetry_at)
    /// attaches a sender; cleared back to `None` when the broadcaster drops.
    pub sender: Option<SyncSender<TickTrace>>,
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
            dropped_traces: 0,
            trace_buffer: TickTrace::default(),
        }
    }
}
