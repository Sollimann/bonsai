//! Always-compiled telemetry primitives. Lives outside `telemetry.rs` so
//! `State::tick`'s signature carries the same `Tracer`/`NodeMeta` types whether
//! or not the `visualize` feature is on.

use crate::{Behavior, Status};

/// Tick-time recording sink, monomorphized into `State::tick`.
///
/// `IS_RECORDING` is a const switch — when `false`, the optimizer constant-
/// folds every `if T::IS_RECORDING { ... }` site to a no-op, allowing the
/// child-id arithmetic and `metas` indexing to be dead-code-eliminated. Code
/// reading `T::IS_RECORDING` MUST be inside an `if` so the compiler can elide
/// the entire branch at monomorphization time.
///
/// Implementations should:
/// - Set `IS_RECORDING = false` only for no-op tracers (use `#[inline(always)]`
///   on `record` so the call disappears).
/// - Set `IS_RECORDING = true` for any tracer that actually consumes the
///   `(id, status)` pair.
pub trait Tracer {
    const IS_RECORDING: bool;
    fn record(&mut self, id: usize, status: Status);
}

pub struct NoopTracer;
impl Tracer for NoopTracer {
    const IS_RECORDING: bool = false;
    #[inline(always)]
    fn record(&mut self, _id: usize, _status: Status) {}
}

/// Preorder metadata for one node — computed once at `BT::new`,
/// tracers to cheaply advance the id counter past unvisited subtrees.
#[derive(Clone, Debug)]
pub struct NodeMeta {
    /// Number of nodes in this subtree, including the root (self).
    pub subtree_size: usize,
}

/// Compute the preorder id of the first child of `self_id`, or a sentinel
/// when telemetry is off. Inlined; the const-fold of `T::IS_RECORDING` removes
/// all arithmetic in the noop path.
#[inline(always)]
pub(crate) fn first_child_id<T: Tracer>(self_id: usize) -> usize {
    if T::IS_RECORDING { self_id + 1 } else { usize::MAX }
}

/// Compute the preorder id of `child_id`'s next sibling. `metas[child_id]` is
/// only read when `T::IS_RECORDING`, so the noop path elides the index.
#[inline(always)]
pub(crate) fn next_sibling_id<T: Tracer>(metas: &[NodeMeta], child_id: usize) -> usize {
    if T::IS_RECORDING {
        child_id + metas[child_id].subtree_size
    } else {
        usize::MAX
    }
}

/// Walk `behavior` in DFS preorder and build a flat `Vec<NodeMeta>` indexed by
/// preorder ID.  The ordering matches `TreeDefinition::traverse` exactly because
/// both call `children_of`.
///
/// Only consumed by visualize-gated code; `allow(dead_code)` silences the
/// warning in non-visualize builds since `mod tracer` is private.
#[cfg_attr(not(feature = "visualize"), allow(dead_code))]
pub fn build_node_metas<A>(behavior: &Behavior<A>) -> Vec<NodeMeta> {
    let mut metas = Vec::new();
    fill(behavior, &mut metas);
    metas
}

#[cfg_attr(not(feature = "visualize"), allow(dead_code))]
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
#[cfg_attr(not(feature = "visualize"), allow(dead_code))]
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
        Select(xs) | Sequence(xs) | WhenAll(xs) | WhenAny(xs) | After(xs) | Race(xs) => xs.iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::children_of;
    use crate::Behavior::{self, Action, AlwaysSucceed, If, Invert, Select, Sequence, Wait, WaitForever, While};

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
        let seq = Sequence(items.clone());
        let sel = Select(items.clone());
        let seq_kids = children_of(&seq);
        let sel_kids = children_of(&sel);
        assert_eq!(seq_kids.len(), 3);
        assert_eq!(sel_kids.len(), 3);
        for i in 0..3 {
            assert_eq!(format!("{:?}", seq_kids[i]), format!("{:?}", &items[i]), "Sequence child {i}");
            assert_eq!(format!("{:?}", sel_kids[i]), format!("{:?}", &items[i]), "Select child {i}");
        }
    }
}
