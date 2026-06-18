//! Visualize-only telemetry types. The whole module is gated on the
//! `visualize` feature in [`lib.rs`](crate); per-item `#[cfg]` gates are
//! therefore unnecessary inside this file.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// `NodeMeta` lives in the always-on [`crate::tracer`] module because
/// `State::tick`'s signature depends on it regardless of the `visualize`
/// feature. Re-exported here so the public path `bonsai_bt::telemetry::NodeMeta`
/// stays valid for downstream code.
pub use crate::tracer::NodeMeta;

use crate::tracer::Tracer;
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

/// Walk `behavior` in DFS preorder and build a flat `Vec<NodeMeta>` indexed by
/// preorder ID.  The ordering matches `TreeDefinition::traverse` exactly because
/// both call `children_of`.
pub fn build_node_metas<A>(behavior: &Behavior<A>) -> Vec<NodeMeta> {
    let mut metas = Vec::new();
    fill(behavior, &mut metas);
    metas
}

fn fill<A>(b: &Behavior<A>, out: &mut Vec<NodeMeta>) -> usize {
    let my_idx = out.len();
    out.push(NodeMeta { subtree_size: 0 }); // placeholder, updated below
    let mut size = 1;
    for c in children_of(b) {
        size += fill(c, out);
    }
    out[my_idx].subtree_size = size;
    size
}

/// Returns the ordered children of a behavior node.
///
/// This is the **single source of truth** for preorder ID assignment order.
/// `build_node_metas` and `TreeDefinition::traverse` must call this rather
/// than re-implementing the ordering independently.
pub(crate) fn children_of<A>(b: &Behavior<A>) -> Vec<&Behavior<A>> {
    use Behavior::*;
    match b {
        Action(_) | Wait(_) | WaitForever => vec![],
        Invert(c) | AlwaysSucceed(c) => vec![c.as_ref()],
        // [condition, on_success, on_failure] — must match skip_subtree logic.
        If(cond, ok, ko) => vec![cond.as_ref(), ok.as_ref(), ko.as_ref()],
        While(cond, body) | WhileAll(cond, body) => {
            let mut v = Vec::with_capacity(1 + body.len());
            v.push(cond.as_ref());
            v.extend(body.iter());
            v
        }
        Select { children, .. } | Sequence { children, .. } => children.iter().collect(),
        WhenAll(xs) | WhenAny(xs) | After(xs) | Race(xs) => xs.iter().collect(),
    }
}

/// Returns the static node-type name and an optional dynamic label.
/// Dynamic label is `Some` only for variants with runtime data worth displaying
/// (Action debug repr, Wait duration); composites fall back to `node_type`.
fn classify<A: std::fmt::Debug>(b: &Behavior<A>) -> (&'static str, Option<String>) {
    use Behavior::*;
    match b {
        Action(a) => ("Action", Some(format!("{a:?}"))),
        Wait(t) => ("Wait", Some(format!("Wait({t:.2}s)"))),
        WaitForever => ("WaitForever", None),
        Invert(_) => ("Inverter", None),
        AlwaysSucceed(_) => ("AlwaysSucceed", None),
        Select { memory: true, .. } => ("Selector", None),
        Select { memory: false, .. } => ("MemorylessSelector", None),
        Sequence { memory: true, .. } => ("Sequence", None),
        Sequence { memory: false, .. } => ("MemorylessSequence", None),
        If(..) => ("If", None),
        While(..) => ("While", None),
        WhileAll(..) => ("WhileAll", None),
        WhenAll(_) => ("WhenAll", None),
        WhenAny(_) => ("WhenAny", None),
        After(_) => ("After", None),
        Race(_) => ("Race", None),
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

#[cfg(test)]
mod tests {
    use super::children_of;
    use crate::Behavior::{self, Action, AlwaysSucceed, If, Invert, Wait, WaitForever, While};

    #[derive(Clone, Debug)]
    enum Act {
        A,
        B,
        C,
    }

    fn ptrs<A>(xs: &[&Behavior<A>]) -> Vec<*const Behavior<A>> {
        xs.iter().map(|b| *b as *const _).collect()
    }

    #[test]
    fn children_of_leaves_yield_empty() {
        assert!(children_of(&Action(Act::A)).is_empty());
        assert!(children_of(&Wait::<Act>(1.0)).is_empty());
        assert!(children_of(&WaitForever::<Act>).is_empty());
    }

    #[test]
    fn children_of_if_is_cond_ok_ko() {
        let cond: Box<Behavior<Act>> = Box::new(Action(Act::A));
        let ok = Box::new(Action(Act::B));
        let ko = Box::new(Action(Act::C));
        let (cp, op, kp) = (&*cond as *const _, &*ok as *const _, &*ko as *const _);
        let b = If(cond, ok, ko);
        assert_eq!(ptrs(&children_of(&b)), vec![cp, op, kp], "If must be [cond, ok, ko]");
    }

    #[test]
    fn children_of_while_is_cond_then_body() {
        let cond: Box<Behavior<Act>> = Box::new(Action(Act::A));
        let body0 = Action(Act::B);
        let body1 = Action(Act::C);
        let cp = &*cond as *const _;
        let b = While(cond, vec![body0, body1]);
        let kids = children_of(&b);
        assert_eq!(kids.len(), 3, "While(cond, [B, C]) has 3 children");
        assert_eq!(kids[0] as *const _, cp, "first child must be the condition");
    }

    #[test]
    fn children_of_decorators_yield_single_child() {
        let inner = Box::new(Action(Act::A));
        let ip = &*inner as *const _;
        let b = Invert(inner);
        assert_eq!(ptrs(&children_of(&b)), vec![ip]);

        let inner2 = Box::new(Action(Act::A));
        let ip2 = &*inner2 as *const _;
        let b2 = AlwaysSucceed(inner2);
        assert_eq!(ptrs(&children_of(&b2)), vec![ip2]);
    }

    #[test]
    fn children_of_composites_preserve_order() {
        let items = vec![Action(Act::A), Action(Act::B), Action(Act::C)];
        let seq = Behavior::sequence(items.clone());
        let sel = Behavior::select(items.clone());
        let r_seq = Behavior::memoryless_sequence(items.clone());
        let r_sel = Behavior::memoryless_selector(items.clone());
        let seq_kids = children_of(&seq);
        let sel_kids = children_of(&sel);
        let r_seq_kids = children_of(&r_seq);
        let r_sel_kids = children_of(&r_sel);
        assert_eq!(seq_kids.len(), 3);
        assert_eq!(sel_kids.len(), 3);
        assert_eq!(r_seq_kids.len(), 3);
        assert_eq!(r_sel_kids.len(), 3);
        for i in 0..3 {
            assert_eq!(
                format!("{:?}", seq_kids[i]),
                format!("{:?}", &items[i]),
                "Sequence child {i}"
            );
            assert_eq!(
                format!("{:?}", sel_kids[i]),
                format!("{:?}", &items[i]),
                "Select child {i}"
            );
            assert_eq!(
                format!("{:?}", r_seq_kids[i]),
                format!("{:?}", &items[i]),
                "MemorylessSequence child {i}"
            );
            assert_eq!(
                format!("{:?}", r_sel_kids[i]),
                format!("{:?}", &items[i]),
                "MemorylessSelector child {i}"
            );
        }
    }
}
