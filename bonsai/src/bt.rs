use crate::{state::State, ActionArgs, Behavior, Float, Status, UpdateEvent};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Result of [`BT::try_route_recording`]: whether the recording helper consumed
/// the tick. `Handled` carries the value `tick` should return; `NotHandled`
/// tells the caller to continue with the no-op path. Under
/// `not(feature = "visualize")` the helper unconditionally returns
/// `NotHandled`, so the `Handled` variant is unconstructed.
#[allow(dead_code)]
enum TickRoute {
    NotHandled,
    Handled(Option<(Status, Float)>),
}

/// The execution state of a behavior tree, along with a "blackboard" (state
/// shared between all nodes in the tree).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BT<A, B> {
    /// constructed behavior tree
    pub(crate) state: State<A>,
    /// keep the initial state
    pub(crate) initial_behavior: Behavior<A>,
    /// The data storage shared by all nodes in the tree. This is generally
    /// referred to as a "blackboard". State is written to and read from a
    /// blackboard, allowing nodes to share state and communicate each other.
    pub(crate) bb: B,
    /// Whether the tree has been finished before.
    pub(crate) finished: bool,
    /// Monotonically increasing per-tick counter. Starts at 0; first completed
    /// `tick`/`tick_recording` call increments to 1. Survives `reset_bt`
    /// (the counter is global to the BT instance, not the current run).
    pub(crate) tick_count: u64,
    /// Bundle of visualize-only state: preorder node metadata, telemetry
    /// channel sender, dropped-trace counter, and the per-tick recording
    /// buffer. See [`crate::telemetry_state::TelemetryState`].
    #[cfg(feature = "visualize")]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) telemetry: crate::telemetry_state::TelemetryState,
}

impl<A: Clone, B> BT<A, B> {
    pub fn new(behavior: Behavior<A>, blackboard: B) -> Self {
        let backup_behavior = behavior.clone();
        let bt = State::new(behavior);

        #[cfg(feature = "visualize")]
        let telemetry =
            crate::telemetry_state::TelemetryState::new(crate::telemetry::build_node_metas(&backup_behavior));

        Self {
            state: bt,
            initial_behavior: backup_behavior,
            bb: blackboard,
            finished: false,
            tick_count: 0,
            #[cfg(feature = "visualize")]
            telemetry,
        }
    }

    /// Updates the cursor that tracks an event. Returns [`None`] if attempting
    /// to tick after this tree has already returned [`Status::Success`] or
    /// [`Status::Failure`].
    ///
    /// The action need to return status and remaining delta time.
    /// Returns status and the remaining delta time.
    ///
    /// Passes event, delta time in seconds, action and state to closure.
    /// The closure should return a status and remaining delta time.
    ///
    /// return: (Status, Float)
    /// function returns the result of the tree traversal, and how long
    /// it actually took to complete the traversal and propagate the
    /// results back up to the root node
    #[inline]
    pub fn tick<E, F>(&mut self, e: &E, f: &mut F) -> Option<(Status, Float)>
    where
        E: UpdateEvent,
        F: FnMut(ActionArgs<E, A>, &mut B) -> (Status, Float),
    {
        if self.finished {
            return None;
        }
        if let TickRoute::Handled(out) = self.try_route_recording(e, f) {
            return out;
        }
        self.tick_count += 1;
        let result = self.dispatch_noop_tick(e, f);
        if matches!(result, (Status::Success | Status::Failure, _)) {
            self.finished = true;
        }
        Some(result)
    }

    /// Run `State::tick` with a [`NoopTracer`](crate::tracer::NoopTracer) (the
    /// non-recording path). The cfg-gated `metas` binding lives inside this
    /// helper so `tick`'s body can stay free of `#[cfg]` directives.
    ///
    /// Disjoint-field borrows: `&self.telemetry.node_metas` (immutable) and
    /// `&mut self.state` / `&mut self.bb` (mutable) target distinct fields,
    /// so the borrow checker accepts the simultaneous borrows.
    ///
    /// `#[inline(always)]` ensures the cfg branches constant-fold at the
    /// monomorphization site, leaving identical generated code to the prior
    /// inlined-in-`tick` version.
    #[inline(always)]
    fn dispatch_noop_tick<E, F>(&mut self, e: &E, f: &mut F) -> (Status, Float)
    where
        E: UpdateEvent,
        F: FnMut(ActionArgs<E, A>, &mut B) -> (Status, Float),
    {
        let mut tracer = crate::tracer::NoopTracer;
        #[cfg(feature = "visualize")]
        let metas: &[crate::tracer::NodeMeta] = &self.telemetry.node_metas;
        #[cfg(not(feature = "visualize"))]
        let metas: &[crate::tracer::NodeMeta] = &[];
        self.state.tick(0, metas, e, &mut self.bb, f, &mut tracer)
    }

    /// If telemetry is attached, dispatch to `tick_recording` and return its
    /// result as [`TickRoute::Handled`]. Returns [`TickRoute::NotHandled`]
    /// otherwise — the caller (`tick`) should proceed with the no-op path.
    ///
    /// `#[inline(always)]` lets the optimizer constant-fold the no-op path:
    /// under `not(feature = "visualize")` the body unconditionally returns
    /// `TickRoute::NotHandled`, so the `if let TickRoute::Handled(_) = ...`
    /// branch in `tick` becomes unreachable and disappears.
    #[inline(always)]
    fn try_route_recording<E, F>(&mut self, e: &E, f: &mut F) -> TickRoute
    where
        E: UpdateEvent,
        F: FnMut(ActionArgs<E, A>, &mut B) -> (Status, Float),
    {
        #[cfg(feature = "visualize")]
        if self.telemetry.sender.is_some() {
            return TickRoute::Handled(self.tick_recording(e, f).map(|(result, _)| result));
        }
        // Suppress unused-variable warnings on the no-op path (visualize off,
        // or visualize on but no sender attached).
        let _ = (e, f);
        TickRoute::NotHandled
    }

    /// Retrieve an immutable reference to the blackboard for
    /// this Behavior Tree
    pub fn blackboard(&self) -> &B {
        &self.bb
    }

    /// Retrieve a mutable reference to the blackboard for
    /// this Behavior Tree
    pub fn blackboard_mut(&mut self) -> &mut B {
        &mut self.bb
    }

    /// The behavior tree is a stateful data structure in which the immediate
    /// state of the BT is allocated and updated in heap memory through the lifetime
    /// of the BT. The state of the BT is said to be `transient` meaning upon entering
    /// a this state, the process may never return this state again. If a behavior concludes,
    /// only the latest results will be stored in heap memory.
    ///
    /// If your BT has surpassed a desired state or that your BT has reached a steady state - meaning
    /// that the behavior has concluded and ticking the BT won't progress any further - then it could
    /// be desirable to return the BT to it's initial state at t=0.0 before it was ever ticked.
    ///
    /// PS! invoking reset_bt does not reset the Blackboard.
    pub fn reset_bt(&mut self) {
        let initial_behavior = self.initial_behavior.to_owned();
        self.state = State::new(initial_behavior);
        self.finished = false;
        // tick_count is intentionally NOT reset — it identifies tick events
        // across the BT's lifetime, including across reset_bt boundaries.
        // dropped_traces resets per-run: it's a diagnostic for the current session.
        #[cfg(feature = "visualize")]
        {
            self.telemetry.dropped_traces = 0;
        }
    }

    /// Returns the total number of ticks this BT has completed (across resets).
    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }

    /// Whether this behavior tree is in a completed state (the last tick returned
    /// [`Status::Success`] or [`Status::Failure`]).
    pub fn is_finished(&self) -> bool {
        self.finished
    }
}
