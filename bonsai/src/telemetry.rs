//! Visualize-only telemetry types. The whole module is gated on the
//! `visualize` feature in [`lib.rs`](crate); per-item `#[cfg]` gates are
//! therefore unnecessary inside this file.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Re-exports preserving the `bonsai_bt::telemetry::*` paths used by tests
/// and downstream code. The implementations live in the always-on
/// [`crate::tracer`] module.
pub use crate::tracer::{build_node_metas, NodeMeta};

use crate::tracer::{children_of, Tracer};
use crate::{Behavior, Status};

pub struct RecordingTracer<'a> {
    pub trace: &'a mut TickTrace,
    pub metas: &'a [NodeMeta],
}

impl Tracer for RecordingTracer<'_> {
    const IS_RECORDING: bool = true;
    #[inline]
    fn record(&mut self, id: usize, status: Status) {
        debug_assert_ne!(id, usize::MAX, "tracer.record called with sentinel id — gating bug");
        self.trace.states.insert(id, status);
    }
}

/// The per-tick payload: maps each visited node's preorder ID to its returned Status.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TickTrace {
    pub tick_id: u64,
    /// Maps Node ID to its return Status for the current tick.
    pub states: HashMap<usize, Status>,
}

/// The immutable structure of the tree, sent once upon WebSocket connection.
#[derive(Serialize, Debug, Clone)]
pub struct TreeDefinition {
    pub root: TreeNode,
}

/// A single node in the static tree layout.
#[derive(Serialize, Debug, Clone)]
pub struct TreeNode {
    pub id: usize,
    pub node_type: &'static str,
    pub label: String,
    pub children: Vec<TreeNode>,
}

/// Returns the static node-type name and an optional dynamic label.
/// Dynamic label is `Some` only for variants with runtime data worth displaying
/// (Action debug repr, Wait duration); composites fall back to `node_type`.
fn classify<A: std::fmt::Debug>(b: &Behavior<A>) -> (&'static str, Option<String>) {
    use Behavior::*;
    match b {
        Action(a)        => ("Action",        Some(format!("{a:?}"))),
        Wait(t)          => ("Wait",          Some(format!("Wait({t:.2}s)"))),
        WaitForever      => ("WaitForever",   None),
        Invert(_)        => ("Inverter",      None),
        AlwaysSucceed(_) => ("AlwaysSucceed", None),
        Select(_)        => ("Selector",      None),
        Sequence(_)      => ("Sequence",      None),
        If(..)           => ("If",            None),
        While(..)        => ("While",         None),
        WhileAll(..)     => ("WhileAll",      None),
        WhenAll(_)       => ("WhenAll",       None),
        WhenAny(_)       => ("WhenAny",       None),
        After(_)         => ("After",         None),
        Race(_)          => ("Race",          None),
    }
}

impl TreeDefinition {
    /// Walk the behavior tree in DFS preorder, assigning stable integer IDs.
    pub fn build<A: std::fmt::Debug>(behavior: &Behavior<A>) -> Self {
        let mut id_counter = 0;
        let root = Self::traverse(behavior, &mut id_counter);
        Self { root }
    }

    pub(crate) fn traverse<A: std::fmt::Debug>(behavior: &Behavior<A>, id_counter: &mut usize) -> TreeNode {
        let id = *id_counter;
        *id_counter += 1;
        let (node_type, label) = classify(behavior);
        let children = children_of(behavior)
            .iter()
            .map(|c| Self::traverse(c, id_counter))
            .collect();
        TreeNode {
            id,
            node_type,
            label: label.unwrap_or_else(|| node_type.to_string()),
            children,
        }
    }
}

/// Embedded HTML payload served at `GET /` by the visualizer server.
#[allow(dead_code)]
pub const VISUALIZER_HTML: &str = include_str!("index.html");
