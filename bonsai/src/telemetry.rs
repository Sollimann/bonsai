use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Behavior, Status};

/// The per-tick payload: maps each visited node's preorder ID to its returned Status.
#[derive(Serialize, Deserialize, Clone, Debug)]
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

/// Preorder metadata for one node — computed once at `BT::new`, used by step-2
/// tracers to cheaply advance the id counter past unvisited subtrees.
#[derive(Clone, Debug)]
pub struct NodeMeta {
    /// Number of nodes in this subtree, including the root (self).
    pub subtree_size: usize,
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
/// `build_node_metas` (step 1) and `RecordingTracer::skip_subtree` (step 2)
/// must call this rather than re-implementing the ordering independently.
pub(crate) fn children_of<A>(b: &Behavior<A>) -> Vec<&Behavior<A>> {
    use Behavior::*;
    match b {
        Action(_) | Wait(_) | WaitForever => vec![],
        Invert(c) | AlwaysSucceed(c) => vec![c.as_ref()],
        // [condition, on_success, on_failure] — must match skip_subtree logic in step 2.
        If(cond, ok, ko) => vec![cond.as_ref(), ok.as_ref(), ko.as_ref()],
        While(cond, body) | WhileAll(cond, body) => {
            let mut v = Vec::with_capacity(1 + body.len());
            v.push(cond.as_ref());
            v.extend(body.iter());
            v
        }
        Select(xs) | Sequence(xs) | WhenAll(xs) | WhenAny(xs) | After(xs) | Race(xs) => {
            xs.iter().collect()
        }
    }
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
