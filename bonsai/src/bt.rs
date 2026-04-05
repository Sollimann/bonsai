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
}

impl<A: Clone, B> BT<A, B> {
    pub fn new(behavior: Behavior<A>, blackboard: B) -> Self {
        let backup_behavior = behavior.clone();
        let bt = State::new(behavior);

        Self {
            state: bt,
            initial_behavior: backup_behavior,
            bb: blackboard,
            finished: false,
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
        match self.state.tick(e, &mut self.bb, f) {
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
    }

    /// Whether this behavior tree is in a completed state (the last tick returned
    /// [`Status::Success`] or [`Status::Failure`]).
    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

#[cfg(feature = "visualize")]
pub mod telemetry {
    use crate::Behavior;
    use serde::Serialize;
    use std::collections::HashMap;
    use std::fmt::Debug;

    /// The immutable structure of the tree, sent once upon connection.
    #[derive(Serialize, Debug, Clone)]
    pub struct TreeDefinition {
        pub root: TreeNode,
    }

    /// A single node in the visualizer.
    #[derive(Serialize, Debug, Clone)]
    pub struct TreeNode {
        pub id: usize,
        pub node_type: String,
        pub label: String,
        pub children: Vec<TreeNode>,
    }

    /// The lightweight, continuous payload sent during `tick()`.
    #[derive(Serialize, Debug, Clone)]
    pub struct TreeStateUpdate {
        /// Map of Node ID -> Status (e.g., "Running", "Success", "Failure")
        pub states: HashMap<usize, String>,
    }

    impl TreeDefinition {
        /// Consumes a Behavior tree and produces a serialized definition.
        pub fn build<A: Debug>(behavior: &Behavior<A>) -> Self {
            let mut id_counter = 0;
            let root = Self::traverse(behavior, &mut id_counter);
            Self { root }
        }

        /// Recursively walks the tree to assign IDs and map types.
        fn traverse<A: Debug>(behavior: &Behavior<A>, id_counter: &mut usize) -> TreeNode {
            let id = *id_counter;
            *id_counter += 1;

            match behavior {
                Behavior::Action(action) => TreeNode {
                    id,
                    node_type: "Action".to_string(),
                    label: format!("{:?}", action), // Requires A: Debug
                    children: vec![],
                },
                Behavior::Sequence(children) => TreeNode {
                    id,
                    node_type: "Sequence".to_string(),
                    label: "Sequence".to_string(),
                    children: children.iter().map(|c| Self::traverse(c, id_counter)).collect(),
                },
                Behavior::Select(children) => TreeNode {
                    id,
                    node_type: "Selector".to_string(),
                    label: "Selector".to_string(),
                    children: children.iter().map(|c| Self::traverse(c, id_counter)).collect(),
                },
                Behavior::Wait(time) => TreeNode {
                    id,
                    node_type: "Wait".to_string(),
                    label: format!("Wait({:.2}s)", time),
                    children: vec![],
                },
                Behavior::WaitForever => TreeNode {
                    id,
                    node_type: "WaitForever".to_string(),
                    label: "WaitForever".to_string(),
                    children: vec![],
                },
                Behavior::Invert(child) => TreeNode {
                    id,
                    node_type: "Inverter".to_string(),
                    label: "Inverter".to_string(),
                    children: vec![Self::traverse(child, id_counter)],
                },
                Behavior::While(condition, body) => {
                    // While has a condition node and a body of nodes
                    let mut children = vec![Self::traverse(condition, id_counter)];
                    children.extend(body.iter().map(|c| Self::traverse(c, id_counter)));
                    TreeNode {
                        id,
                        node_type: "While".to_string(),
                        label: "While".to_string(),
                        children,
                    }
                },
                // Catch-all for any other bonsai-bt variants (If, WhenAll, etc.)
                _ => TreeNode {
                    id,
                    node_type: "Decorator/Other".to_string(),
                    label: "Unknown".to_string(),
                    children: vec![],
                }
            }
        }
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

    /// Compiles the behavior tree into a Groot-compatible JSON string
    /// representing the static hierarchy of the tree.
    pub fn get_telemetry_definition(&self) -> String {
        let definition = telemetry::TreeDefinition::build(&self.initial_behavior);
        
        // Serialize to a formatted JSON string (or use to_string() for minified)
        serde_json::to_string_pretty(&definition)
            .unwrap_or_else(|_| "{\"error\": \"Failed to serialize tree\"}".to_string())
    }
}
