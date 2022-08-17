#![allow(dead_code, unused_variables)]
use std::collections::HashMap;
use std::fmt::Debug;

use petgraph::dot::{Config, Dot};
use petgraph::{stable_graph::NodeIndex, Graph};

use crate::visualizer::NodeType;
use crate::{Behavior, State};

/// A "blackboard" is a simple key/value storage shared by all the nodes of the Tree.
/// It is esseintially a database in which the behavior tree can store information
/// whilst traversing the tree
///
/// An "entry" of the Blackboard is a key/value pair.
#[derive(Clone, Debug)]
pub struct BlackBoard<K, V>(HashMap<K, V>);

impl<K, V> BlackBoard<K, V> {
    pub fn get_db(&mut self) -> &mut HashMap<K, V> {
        &mut self.0
    }
}

/// The BT struct contains a compiled (immutable) version
/// of the behavior and a blackboard key/value storage
#[derive(Clone, Debug)]
pub struct BT<A, K, V> {
    /// constructed behavior tree
    pub state: State<A>,
    /// keep the initial state
    initial_behavior: Behavior<A>,
    /// blackboard
    bb: BlackBoard<K, V>,
    /// Tree formulated as PetGraph
    pub(crate) graph: Graph<NodeType<A>, u32, petgraph::Directed>,
    /// root node
    root_id: NodeIndex,
}

impl<A: Clone + Debug, K: Debug, V: Debug> BT<A, K, V> {
    pub fn new(behavior: Behavior<A>, blackboard: HashMap<K, V>) -> Self {
        let backup_behavior = behavior.clone();
        let bt = State::new(behavior);

        // generate graph
        let mut graph = Graph::<NodeType<A>, u32, petgraph::Directed>::new();
        let root_id = graph.add_node(NodeType::Root);

        Self {
            state: bt,
            initial_behavior: backup_behavior,
            bb: BlackBoard(blackboard),
            graph,
            root_id,
        }
    }

    pub fn get_graphviz(&mut self) -> String {
        let behavior = self.initial_behavior.to_owned();
        self.dfs_recursive(behavior, self.root_id);
        let digraph = Dot::with_config(&self.graph, &[Config::EdgeNoLabel]);
        format!("{:?}", digraph)
    }

    /// Retrieve a mutable reference to the blackboard for
    /// this Behavior Tree
    pub fn get_blackboard(&mut self) -> &mut BlackBoard<K, V> {
        &mut self.bb
    }

    /// Retrieve a mutable reference to the internal state
    /// of the Behavior Tree
    pub fn get_state(bt: &mut BT<A, K, V>) -> &mut State<A> {
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
