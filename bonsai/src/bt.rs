use std::fmt::Debug;

use crate::visualizer::NodeType;
use crate::{ActionArgs, Behavior, State, Status, UpdateEvent};
use petgraph::dot::{Config, Dot};
use petgraph::Graph;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The BT struct contains a compiled (immutable) version
/// of the behavior and a blackboard key/value storage
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BT<A, K> {
    /// constructed behavior tree
    pub state: State<A>,
    /// keep the initial state
    initial_behavior: Behavior<A>,
    /// blackboard
    bb: K,
}

impl<A: Clone, K> BT<A, K> {
    pub fn new(behavior: Behavior<A>, blackboard: K) -> Self {
        let backup_behavior = behavior.clone();
        let bt = State::new(behavior);

        Self {
            state: bt,
            initial_behavior: backup_behavior,
            bb: blackboard,
        }
    }

    /// Updates the cursor that tracks an event.
    ///
    /// The action need to return status and remaining delta time.
    /// Returns status and the remaining delta time.
    ///
    /// Passes event, delta time in seconds, action and state to closure.
    /// The closure should return a status and remaining delta time.
    ///
    /// return: (Status, f64)
    /// function returns the result of the tree traversal, and how long
    /// it actually took to complete the traversal and propagate the
    /// results back up to the root node
    #[inline]
    pub fn tick<E, F>(&mut self, e: &E, f: &mut F) -> (Status, f64)
    where
        E: UpdateEvent,
        F: FnMut(ActionArgs<E, A>, &mut K) -> (Status, f64),
    {
        self.state.tick(e, &mut self.bb, f)
    }

    /// Retrieve a mutable reference to the blackboard for
    /// this Behavior Tree
    pub fn get_blackboard(&mut self) -> &mut K {
        &mut self.bb
    }

    /// Retrieve a mutable reference to the internal state
    /// of the Behavior Tree
    pub fn get_state(bt: &mut BT<A, K>) -> &mut State<A> {
        &mut bt.state
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
        self.state = State::new(initial_behavior)
    }
}

impl<A: Clone + Debug, K: Debug> BT<A, K> {
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

    pub(crate) fn get_graphviz_with_graph_instance(&mut self) -> (String, Graph<NodeType<A>, u32>) {
        let behavior = self.initial_behavior.to_owned();

        let mut graph = Graph::<NodeType<A>, u32, petgraph::Directed>::new();
        let root_id = graph.add_node(NodeType::Root);

        Self::dfs_recursive(&mut graph, behavior, root_id);

        let digraph = Dot::with_config(&graph, &[Config::EdgeNoLabel]);
        (format!("{:?}", digraph), graph)
    }
}
