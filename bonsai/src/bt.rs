use std::fmt::Debug;

use crate::{state::State, ActionArgs, Behavior, Float, Status, UpdateEvent};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The execution state of a behavior tree, along with a "blackboard" (state
/// shared between all nodes in the tree).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BT<A, B> {
    /// constructed behavior tree
    state: State<A>,
    /// keep the initial state
    initial_behavior: Behavior<A>,
    /// The data storage shared by all nodes in the tree. This is generally
    /// referred to as a "blackboard". State is written to and read from a
    /// blackboard, allowing nodes to share state and communicate each other.
    bb: B,
    /// Whether the tree has been finished before.
    finished: bool,
    /// Monotonically increasing per-tick counter. Starts at 0; first completed
    /// `tick`/`tick_recording` call increments to 1. Survives `reset_bt`
    /// (the counter is global to the BT instance, not the current run).
    tick_count: u64,
    /// Preorder node metadata, computed once at `BT::new`.
    /// Used by `RecordingTracer` to advance past unvisited subtrees in O(1).
    #[cfg(feature = "visualize")]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) node_metas: Vec<crate::tracer::NodeMeta>,
    /// Channel sender for shipping `TickTrace`s to the broadcaster thread.
    /// `None` when telemetry is not active or after the broadcaster has exited.
    #[cfg(feature = "visualize")]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) telemetry_sender: Option<std::sync::mpsc::SyncSender<crate::telemetry::TickTrace>>,
    /// Number of `TickTrace`s dropped because the channel was full.
    /// Reset on `reset_bt`. Useful for diagnosing slow clients.
    #[cfg(feature = "visualize")]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) dropped_traces: u64,
    /// Reusable buffer for the recording trace. Held for the BT's lifetime;
    /// `tick_recording` clears it on entry, preserving capacity. Avoids one
    /// `HashMap` allocation per tick on the hot path
    #[cfg(feature = "visualize")]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) trace_buffer: crate::telemetry::TickTrace,
}

impl<A: Clone, B> BT<A, B> {
    pub fn new(behavior: Behavior<A>, blackboard: B) -> Self {
        let backup_behavior = behavior.clone();
        let bt = State::new(behavior);

        #[cfg(feature = "visualize")]
        let node_metas = crate::tracer::build_node_metas(&backup_behavior);

        Self {
            state: bt,
            initial_behavior: backup_behavior,
            bb: blackboard,
            finished: false,
            tick_count: 0,
            #[cfg(feature = "visualize")]
            node_metas,
            #[cfg(feature = "visualize")]
            telemetry_sender: None,
            #[cfg(feature = "visualize")]
            dropped_traces: 0,
            #[cfg(feature = "visualize")]
            trace_buffer: crate::telemetry::TickTrace {
                tick_id: 0,
                states: std::collections::HashMap::new(),
            },
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
        // When telemetry is attached, delegate to the recording path so the
        // broadcaster receives a TickTrace. One runtime branch; both sides
        // keep their Tracer monomorphizations.
        #[cfg(feature = "visualize")]
        if self.telemetry_sender.is_some() {
            return self.tick_recording(e, f).map(|(result, _)| result);
        }
        self.tick_count += 1;
        let mut tracer = crate::tracer::NoopTracer;
        #[cfg(feature = "visualize")]
        let metas: &[crate::tracer::NodeMeta] = &self.node_metas;
        #[cfg(not(feature = "visualize"))]
        let metas: &[crate::tracer::NodeMeta] = &[];
        match self.state.tick(0, metas, e, &mut self.bb, f, &mut tracer) {
            result @ (Status::Success | Status::Failure, _) => {
                self.finished = true;
                Some(result)
            }
            result => Some(result),
        }
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
            self.dropped_traces = 0;
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


#[cfg(feature = "visualize")]
impl<A: Clone, B> BT<A, B> {
    /// Number of `TickTrace`s dropped because the broadcaster channel was full.
    /// Reset on `reset_bt`. Useful for diagnosing slow visualizer clients.
    /// Always returns 0 if the visualizer was never attached via `BT::with_telemetry`.
    pub fn dropped_traces(&self) -> u64 {
        self.dropped_traces
    }

    /// Tick the tree once and return both the standard tick result and a
    /// [`TickTrace`](crate::telemetry::TickTrace) recording every node visited
    /// this frame. Used by the in-process telemetry channel and by integration
    /// tests.
    ///
    /// Returns `None` if the tree has already finished (mirroring [`tick`](Self::tick)).
    pub fn tick_recording<E, F>(
        &mut self,
        e: &E,
        f: &mut F,
    ) -> Option<((Status, Float), crate::telemetry::TickTrace)>
    where
        E: UpdateEvent,
        F: FnMut(ActionArgs<E, A>, &mut B) -> (Status, Float),
    {
        if self.finished {
            return None;
        }
        self.tick_count += 1;
        // Reuse the long-lived buffer instead of fresh-allocating per tick.
        // `clear()` preserves the HashMap's capacity, so once warmed up the
        // fill phase doesn't reallocate.
        self.trace_buffer.tick_id = self.tick_count;
        self.trace_buffer.states.clear();
        let result = {
            let mut tracer = crate::telemetry::RecordingTracer {
                trace: &mut self.trace_buffer,
                metas: &self.node_metas,
            };
            self.state
                .tick(0, &self.node_metas, e, &mut self.bb, f, &mut tracer)
        };
        if matches!(result, (Status::Success | Status::Failure, _)) {
            self.finished = true;
        }
        // Try to ship the trace to the broadcaster thread. Uses as_ref().map() to
        // release the immutable borrow before the match arms take mutable borrows.
        if let Some(outcome) = self
            .telemetry_sender
            .as_ref()
            .map(|tx| tx.try_send(self.trace_buffer.clone()))
        {
            use std::sync::mpsc::TrySendError;
            match outcome {
                Ok(()) => {}
                Err(TrySendError::Full(_)) => self.dropped_traces += 1,
                Err(TrySendError::Disconnected(_)) => self.telemetry_sender = None,
            }
        }
        Some((result, self.trace_buffer.clone()))
    }

    /// Attach a live visualizer at `http://127.0.0.1:{port}/`.
    ///
    /// Convenience for [`with_telemetry_at`](Self::with_telemetry_at) with the
    /// loopback address. Use that method instead if you need to bind a
    /// different host (e.g. `"0.0.0.0"` to expose the visualizer on the LAN).
    ///
    /// Builder method: consumes and returns `Self` for chaining off [`BT::new`].
    /// Spawns a listener thread and a broadcaster thread. Telemetry is
    /// **best-effort** — [`TickTrace`](crate::telemetry::TickTrace)s are dropped
    /// silently when the channel is full; the count is surfaced via
    /// [`dropped_traces`](Self::dropped_traces).
    ///
    /// After calling this method, [`tick`](Self::tick) automatically records and
    /// ships a trace on every call — no API change required.
    ///
    /// # Errors
    /// Returns `io::Error` if `127.0.0.1:{port}` cannot be bound.
    ///
    /// # Example
    /// ```no_run
    /// # use bonsai_bt::{Action, BT, Behavior::Sequence};
    /// # use std::collections::HashMap;
    /// let behavior = Sequence(vec![Action("step")]);
    /// let bb: HashMap<String, i32> = HashMap::new();
    /// let mut bt = BT::new(behavior, bb).with_telemetry(8910)?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn with_telemetry(self, port: u16) -> std::io::Result<Self>
    where
        A: std::fmt::Debug,
    {
        self.with_telemetry_at("127.0.0.1", port)
    }

    /// Attach a live visualizer at `http://{addr}:{port}/`.
    ///
    /// Like [`with_telemetry`](Self::with_telemetry), but lets you pick the
    /// bind address. Pass `"0.0.0.0"` to listen on every interface (so peers
    /// on the LAN can connect), `"127.0.0.1"` for loopback only, or any
    /// specific interface IP. The address is parsed by `TcpListener::bind`,
    /// so hostnames and IPv6 literals (e.g. `"::1"`) also work.
    ///
    /// Spawns a listener thread and a broadcaster thread. Telemetry is
    /// **best-effort** — [`TickTrace`](crate::telemetry::TickTrace)s are dropped
    /// silently when the channel is full; the count is surfaced via
    /// [`dropped_traces`](Self::dropped_traces).
    ///
    /// After calling this method, [`tick`](Self::tick) automatically records and
    /// ships a trace on every call — no API change required.
    ///
    /// Calling either telemetry method a second time replaces the existing
    /// sender; the previous broadcaster exits on its next `recv` (sees
    /// `Disconnected`). If the same `(addr, port)` is still bound, the second
    /// bind returns `AddrInUse`.
    ///
    /// # Errors
    /// Returns `io::Error` if `{addr}:{port}` cannot be bound (invalid address,
    /// `AddrInUse`, permission denied for low ports, etc.).
    ///
    /// # Edge cases
    /// - **Cloning a telemetry-attached `BT` is not supported.** `BT` derives
    ///   `Clone`, and `Option<SyncSender<_>>` clones share the underlying
    ///   channel — both BTs would interleave their `TickTrace`s into the same
    ///   broadcaster, producing nonsense in the visualizer. Either clone
    ///   *before* attaching telemetry, or call `with_telemetry_at` again on
    ///   the clone (with a different port).
    /// - **`reset_bt` preserves telemetry.** The sender survives, `tick_count`
    ///   keeps climbing, `dropped_traces` resets to 0. Connected browsers see
    ///   no disruption.
    /// - **Already-finished BT.** `with_telemetry_at` still works — the tree
    ///   definition is sent to clients on connect — but no `TickTrace`s flow
    ///   until [`reset_bt`](Self::reset_bt) is called (`tick` returns `None`).
    /// - **Deserializing a BT.** `telemetry_sender` is `#[serde(skip)]`, so the
    ///   deserialized BT runs without telemetry until a telemetry method is
    ///   called again — the typical "fresh run" workflow.
    /// - **Pre-connect tick backlog.** Traces queue in the channel (capacity
    ///   1024) until the first WS client connects. If no one connects within
    ///   1024 ticks, older traces are dropped (`dropped_traces` increments);
    ///   the first connecting client sees the tree definition plus the *next*
    ///   tick onward, not the backlog.
    /// - **Binding `0.0.0.0`.** The URL printed to stdout is the address you
    ///   passed; modern browsers map `http://0.0.0.0` to localhost, but for a
    ///   peer to connect they need this machine's actual LAN IP.
    ///
    /// # Example
    /// ```no_run
    /// # use bonsai_bt::{Action, BT, Behavior::Sequence};
    /// # use std::collections::HashMap;
    /// let behavior = Sequence(vec![Action("step")]);
    /// let bb: HashMap<String, i32> = HashMap::new();
    /// // Expose to the LAN — anyone who can reach this host can view the tree.
    /// let mut bt = BT::new(behavior, bb).with_telemetry_at("0.0.0.0", 8910)?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn with_telemetry_at(mut self, addr: &str, port: u16) -> std::io::Result<Self>
    where
        A: std::fmt::Debug,
    {
        use crate::telemetry::{TickTrace, TreeDefinition};
        use std::net::TcpListener;
        use std::sync::mpsc::sync_channel;

        let listener = TcpListener::bind((addr, port))?;
        // local_addr reflects the resolved socket — useful when port=0 (kernel-
        // assigned) or when the user passes a hostname rather than a literal IP.
        let bound = listener.local_addr()?;
        let definition = serde_json::to_string(&TreeDefinition::build(&self.initial_behavior))
            .expect("TreeDefinition is always serializable");
        let (tx, rx) = sync_channel::<TickTrace>(1024);
        crate::visualizer_server::spawn_server(listener, definition, rx)?;
        self.telemetry_sender = Some(tx);
        println!("bonsai-bt visualizer: open http://{bound}/");
        Ok(self)
    }
}

#[cfg(feature = "visualize")]
impl<A: Clone + Debug, B: Debug> BT<A, B> {
    /// Compile the behavior tree into a [graphviz](https://graphviz.org/) compatible [DiGraph](https://docs.rs/petgraph/latest/petgraph/graph/type.DiGraph.html).
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use bonsai_bt::{
    ///     Behavior::{Action, Sequence, Wait, WaitForever, While},
    ///     BT
    /// };
    ///
    /// #[derive(Clone, Debug, Copy)]
    /// pub enum Counter {
    ///     // Increment accumulator.
    ///     Inc,
    ///     // Decrement accumulator.
    ///     Dec,
    /// }
    ///
    ///
    /// // create the behavior
    /// let behavior = While(Box::new(WaitForever), vec![Wait(0.5), Action(Counter::Inc), WaitForever]);
    ///
    /// let h: HashMap<String, i32> = HashMap::new();
    /// let mut bt = BT::new(behavior, h);
    ///
    /// // produce a string DiGraph compatible with graphviz
    /// // paste the contents in graphviz, e.g: https://dreampuf.github.io/GraphvizOnline/#
    /// let g = bt.get_graphviz();
    /// println!("{}", g);
    /// ```
    pub fn get_graphviz(&mut self) -> String {
        self.get_graphviz_with_graph_instance().0
    }

    pub(crate) fn get_graphviz_with_graph_instance(
        &mut self,
    ) -> (String, petgraph::Graph<crate::visualizer::NodeType<A>, u32>) {
        use crate::visualizer::NodeType;
        use petgraph::dot::{Config, Dot};
        use petgraph::Graph;

        let behavior = self.initial_behavior.to_owned();

        let mut graph = Graph::<NodeType<A>, u32, petgraph::Directed>::new();
        let root_id = graph.add_node(NodeType::Root);

        Self::dfs_recursive(&mut graph, behavior, root_id);

        let digraph = Dot::with_config(&graph, &[Config::EdgeNoLabel]);
        (format!("{:?}", digraph), graph)
    }

    /// Compiles the behavior tree into a JSON string representing the static hierarchy.
    pub fn get_telemetry_definition(&self) -> String {
        let definition = crate::telemetry::TreeDefinition::build(&self.initial_behavior);
        serde_json::to_string_pretty(&definition)
            .expect("TreeDefinition is always serializable")
    }
}
