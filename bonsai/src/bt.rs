use std::fmt::Debug;

use petgraph::dot::{Config, Dot};
use petgraph::Graph;

use crate::visualizer::NodeType;
use crate::{ActionArgs, Behavior, State, Status, UpdateEvent};

/// A "blackboard" is a simple key/value storage shared by all the nodes of the Tree.
///
/// It is essentially a database in which the behavior tree can store information
/// whilst traversing the tree. Certain action nodes depend on state that might be
/// dynamically created by other nodes in the tree. State is written to and read from
/// a blackboard, a messaging capability that allows nodes to share state in the behavior tree.
///
/// An "entry" of the Blackboard is a key/value pair.
#[derive(Clone, Debug)]
pub struct BlackBoard<K>(K);

impl<K> BlackBoard<K> {
    pub fn get_db(&mut self) -> &mut K {
        &mut self.0
    }
}

/// The BT struct contains a compiled (immutable) version
/// of the behavior and a blackboard key/value storage
#[derive(Clone, Debug)]
pub struct BT<A, K> {
    /// constructed behavior tree
    pub state: State<A>,
    /// keep the initial state
    initial_behavior: Behavior<A>,
    /// blackboard
    bb: BlackBoard<K>,
}

impl<A: Clone + Debug, K: Debug> BT<A, K> {
    pub fn new(behavior: Behavior<A>, blackboard: K) -> Self {
        let backup_behavior = behavior.clone();
        let bt = State::new(behavior);

        Self {
            state: bt,
            initial_behavior: backup_behavior,
            bb: BlackBoard(blackboard),
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
        F: FnMut(ActionArgs<E, A>, &mut BlackBoard<K>) -> (Status, f64),
        A: Debug,
    {
        self.state.tick(e, &mut self.bb, f)
    }

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

    /// Retrieve a mutable reference to the blackboard for
    /// this Behavior Tree
    pub fn get_blackboard(&mut self) -> &mut BlackBoard<K> {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::BlackBoard;

    #[test]
    fn test_bb() {
        // add some values to blackboard
        let mut db: HashMap<String, f32> = HashMap::new();
        db.insert("win_width".to_string(), 10.0);
        db.insert("win_height".to_string(), 12.0);

        let mut blackboard = BlackBoard(db);
        let win_width = blackboard.get_db().get("win_width").unwrap().to_owned();
        assert_eq!(win_width, 10.0);
    }
}
