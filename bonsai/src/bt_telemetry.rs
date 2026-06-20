//! Visualize-only `impl` blocks for [`crate::BT`]. The whole file is gated on
//! the `visualize` feature in `lib.rs`, so per-item `#[cfg]` attributes inside
//! are unnecessary.

use std::fmt::Debug;

use crate::telemetry::{RecordingTracer, TickTrace, TreeDefinition};
use crate::{ActionArgs, Float, Status, UpdateEvent, BT};

impl<A: Clone, B> BT<A, B> {
    /// Number of `TickTrace`s dropped because the broadcaster channel was full.
    /// Reset on `reset_bt`. Useful for diagnosing slow visualizer clients.
    /// Always returns 0 if the visualizer was never attached via `BT::with_telemetry`.
    pub fn dropped_traces(&self) -> u64 {
        self.telemetry.dropped_traces
    }

    /// Tick the tree once and return both the standard tick result and a
    /// [`TickTrace`](crate::telemetry::TickTrace) recording every node visited
    /// this frame. Used by the in-process telemetry channel and by integration
    /// tests.
    ///
    /// Returns `None` if the tree has already finished (mirroring [`tick`](Self::tick)).
    pub fn tick_recording<E, F>(&mut self, e: &E, f: &mut F) -> Option<((Status, Float), TickTrace)>
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
        self.telemetry.trace_buffer.tick_id = self.tick_count;
        self.telemetry.trace_buffer.states.clear();
        let result = {
            let mut tracer = RecordingTracer {
                trace: &mut self.telemetry.trace_buffer,
                metas: &self.telemetry.node_metas,
            };
            self.state
                .tick(0, &self.telemetry.node_metas, e, &mut self.bb, f, &mut tracer)
        };
        if matches!(result, (Status::Success | Status::Failure, _)) {
            self.finished = true;
        }
        // Try to ship the trace to the broadcaster thread. Uses as_ref().map() to
        // release the immutable borrow before the match arms take mutable borrows.
        if let Some(outcome) = self
            .telemetry
            .sender
            .as_ref()
            .map(|tx| tx.try_send(self.telemetry.trace_buffer.clone()))
        {
            use std::sync::mpsc::TrySendError;
            match outcome {
                Ok(()) => {}
                Err(TrySendError::Full(_)) => self.telemetry.dropped_traces += 1,
                Err(TrySendError::Disconnected(_)) => self.telemetry.sender = None,
            }
        }
        Some((result, self.telemetry.trace_buffer.clone()))
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
    /// # use bonsai_bt::{Action, BT, Sequence};
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
    /// Calling either telemetry method a second time replaces both the
    /// sender and the acceptor guard: the previous broadcaster exits on its
    /// next `recv` (sees `Disconnected`), and the previous acceptor is woken
    /// and torn down before the new `TcpListener::bind` runs, so re-attaching
    /// on the same `(addr, port)` works.
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
    /// # use bonsai_bt::{Action, BT, Sequence};
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
        use std::net::TcpListener;
        use std::sync::mpsc::sync_channel;
        use std::sync::Arc;

        // Drop any prior acceptor guard before binding so re-attaching on the
        // same `(addr, port)` releases the old port first. Replacing the
        // field afterward would only run the old guard's `Drop` *after* the
        // new bind, which would race with `AddrInUse`.
        self.telemetry.acceptor_guard = None;

        let listener = TcpListener::bind((addr, port))?;
        let definition = serde_json::to_string(&TreeDefinition::build(&self.initial_behavior))
            .expect("TreeDefinition is always serializable");
        let (tx, rx) = sync_channel::<TickTrace>(1024);
        let (acceptor_handle, shutdown, bound_addr) = crate::visualizer_server::spawn_server(listener, definition, rx)?;
        self.telemetry.sender = Some(tx);
        self.telemetry.acceptor_guard = Some(Arc::new(crate::telemetry_state::AcceptorGuard::new(
            shutdown,
            bound_addr,
            acceptor_handle,
        )));
        Ok(self)
    }
}

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
        let definition = TreeDefinition::build(&self.initial_behavior);
        serde_json::to_string_pretty(&definition).expect("TreeDefinition is always serializable")
    }
}
