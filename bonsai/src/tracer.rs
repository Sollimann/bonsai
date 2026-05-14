//! Always-compiled telemetry primitives. Lives outside `telemetry.rs` so
//! `State::tick`'s signature carries the same `Tracer`/`NodeMeta` types whether
//! or not the `visualize` feature is on.

use crate::Status;

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
    if T::IS_RECORDING {
        self_id + 1
    } else {
        usize::MAX
    }
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
