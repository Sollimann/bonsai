#![allow(dead_code, unused_imports, unused_variables)]
use crate::{state::State, Behavior, Float, BT};
use petgraph::{graph::Graph, stable_graph::NodeIndex, Direction::Outgoing};
use std::{collections::VecDeque, fmt::Debug};

#[derive(Debug, Clone)]
pub(crate) enum NodeType<A> {
    Root,
    Wait(Float),
    WaitForever,
    Action(A),
    Invert,
    AlwaysSucceed,
    Select,
    If,
    Sequence,
    MemorylessSequence,
    MemorylessSelector,
    WhileAll,
    While,
    WhenAll,
    WhenAny,
    After,
    Race,
}

impl<A: Clone + Debug, K: Debug> BT<A, K> {
    pub(crate) fn dfs_recursive(
        graph: &mut Graph<NodeType<A>, u32, petgraph::Directed>,
        behavior: Behavior<A>,
        parent_node: NodeIndex,
    ) {
        match behavior {
            Behavior::Action(action) => {
                let node_id = graph.add_node(NodeType::Action(action));
                graph.add_edge(parent_node, node_id, 1);
            }
            Behavior::Invert(ev) => {
                let node_id = graph.add_node(NodeType::Invert);
                graph.add_edge(parent_node, node_id, 1);
                Self::dfs_recursive(graph, *ev, node_id)
            }
            Behavior::AlwaysSucceed(ev) => {
                let node_id = graph.add_node(NodeType::AlwaysSucceed);
                graph.add_edge(parent_node, node_id, 1);
                Self::dfs_recursive(graph, *ev, node_id)
            }
            Behavior::Wait(dt) => {
                let node_id = graph.add_node(NodeType::Wait(dt));
                graph.add_edge(parent_node, node_id, 1);
            }
            Behavior::WaitForever => {
                let node_id = graph.add_node(NodeType::WaitForever);
                graph.add_edge(parent_node, node_id, 1);
            }
            Behavior::If(condition, success, failure) => {
                let node_id = graph.add_node(NodeType::If);
                graph.add_edge(parent_node, node_id, 1);

                // left (if condition)
                let left = *condition;
                Self::dfs_recursive(graph, left, node_id);

                // middle (execute if condition is True)
                let middle = *success;
                Self::dfs_recursive(graph, middle, node_id);

                // right (execute if condition is False)
                let right = *failure;
                Self::dfs_recursive(graph, right, node_id);
            }
            Behavior::Select { children, memory } => {
                let node_id = graph.add_node(if memory {
                    NodeType::Select
                } else {
                    NodeType::MemorylessSelector
                });
                graph.add_edge(parent_node, node_id, 1);
                for b in children {
                    Self::dfs_recursive(graph, b, node_id)
                }
            }
            Behavior::Sequence { children, memory } => {
                let node_id = graph.add_node(if memory {
                    NodeType::Sequence
                } else {
                    NodeType::MemorylessSequence
                });
                graph.add_edge(parent_node, node_id, 1);
                for b in children {
                    Self::dfs_recursive(graph, b, node_id)
                }
            }
            Behavior::While(ev, seq) => {
                let node_id = graph.add_node(NodeType::While);
                graph.add_edge(parent_node, node_id, 1);

                // left
                let left = *ev;
                Self::dfs_recursive(graph, left, node_id);

                // right
                let right = Behavior::sequence(seq);
                Self::dfs_recursive(graph, right, node_id)
            }
            Behavior::WhileAll(ev, seq) => {
                let node_id = graph.add_node(NodeType::WhileAll);
                graph.add_edge(parent_node, node_id, 1);

                // left
                let left = *ev;
                Self::dfs_recursive(graph, left, node_id);

                // right
                let right = Behavior::sequence(seq);
                Self::dfs_recursive(graph, right, node_id)
            }
            Behavior::WhenAll(all) => {
                let node_id = graph.add_node(NodeType::WhenAll);
                graph.add_edge(parent_node, node_id, 1);
                for b in all {
                    Self::dfs_recursive(graph, b, node_id)
                }
            }
            Behavior::WhenAny(any) => {
                let node_id = graph.add_node(NodeType::WhenAny);
                graph.add_edge(parent_node, node_id, 1);
                for b in any {
                    Self::dfs_recursive(graph, b, node_id)
                }
            }
            Behavior::After(after_all) => {
                let node_id = graph.add_node(NodeType::After);
                graph.add_edge(parent_node, node_id, 1);
                for b in after_all {
                    Self::dfs_recursive(graph, b, node_id)
                }
            }
            Behavior::Race(behaviors) => {
                let node_id = graph.add_node(NodeType::Race);
                graph.add_edge(parent_node, node_id, 1);
                for b in behaviors {
                    Self::dfs_recursive(graph, b, node_id)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualizer::tests::TestActions::{Dec, Inc};
    use crate::Behavior::{self, Action, After, AlwaysSucceed, If, Invert, Wait, WaitForever, WhenAll, WhenAny, While};
    use crate::Status::{self, Success};
    use crate::{ActionArgs, Event, UpdateArgs};
    use petgraph::dot::{Config, Dot};
    use petgraph::Graph;
    use std::collections::HashMap;

    /// Some test actions.
    #[derive(Clone, Debug)]
    enum TestActions {
        /// Increment accumulator.
        Inc,
        /// Decrement accumulator.
        Dec,
    }

    // A test state machine that can increment and decrement.
    fn tick(mut acc: i32, dt: Float, bt: &mut BT<TestActions, HashMap<String, i32>>) -> (i32, Status, Float) {
        let e: Event = UpdateArgs { dt }.into();
        let (s, t) = bt
            .tick(&e, &mut |args, blackboard| match args.action {
                TestActions::Inc => {
                    acc += 1;
                    (Success, args.dt)
                }
                TestActions::Dec => {
                    acc -= 1;
                    (Success, args.dt)
                }
            })
            .unwrap();
        (acc, s, t)
    }

    #[test]
    fn test_viz_sequence_and_action() {
        use petgraph::dot::{Config, Dot};
        use petgraph::Graph;

        let behavior = Behavior::sequence(vec![
            Action(Dec),
            Action(Dec),
            Behavior::sequence(vec![Action(Inc), Behavior::sequence(vec![Action(Inc)])]),
        ]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 7);
        assert_eq!(g.node_count(), 8);
    }

    #[test]
    fn test_viz_select_and_action() {
        let behavior = Behavior::select(vec![
            Action(Dec),
            Behavior::select(vec![
                Action(Inc),
                Behavior::sequence(vec![Action(Inc), Action(Dec)]),
                Action(Inc),
            ]),
            Action(Dec),
        ]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 9);
        assert_eq!(g.node_count(), 10);
    }

    #[test]
    fn test_viz_sequence_action_wait() {
        let behavior = Behavior::sequence(vec![
            Action(Dec),
            Wait(10.0),
            Action(Dec),
            Behavior::select(vec![Wait(5.0), Behavior::sequence(vec![Action(Inc), Action(Dec)])]),
        ]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 9);
        assert_eq!(g.node_count(), 10);
    }

    #[test]
    fn test_viz_while() {
        let behavior = While(Box::new(Wait(50.0)), vec![Wait(0.5), Action(Inc), Wait(0.5)]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 6);
        assert_eq!(g.node_count(), 7);
    }

    #[test]
    fn test_viz_while_wait_forever() {
        let behavior = While(Box::new(WaitForever), vec![Wait(0.5), Action(Inc), WaitForever]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 6);
        assert_eq!(g.node_count(), 7);
    }

    #[test]
    fn test_viz_while_select_sequence_wait() {
        let behavior = While(
            Box::new(Behavior::select(vec![
                Wait(5.0),
                Behavior::sequence(vec![Action(Inc), Action(Dec)]),
                Action(Inc),
            ])),
            vec![Wait(0.5), Action(Inc), Wait(0.5)],
        );

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 11);
        assert_eq!(g.node_count(), 12);
    }

    #[test]
    fn test_invert() {
        let behavior = Behavior::sequence(vec![Invert(Box::new(Action(Inc))), Action(Dec)]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 4);
        assert_eq!(g.node_count(), 5);
    }

    #[test]
    fn test_sequence_invert_select() {
        let behavior = Behavior::sequence(vec![
            Action(Dec),
            Action(Dec),
            Invert(Box::new(Behavior::select(vec![Action(Inc), Action(Dec)]))),
            Action(Dec),
        ]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 8);
        assert_eq!(g.node_count(), 9);
    }

    #[test]
    fn test_always_succeed() {
        let behavior = Behavior::sequence(vec![AlwaysSucceed(Box::new(Action(Inc))), Action(Dec)]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 4);
        assert_eq!(g.node_count(), 5);
    }

    #[test]
    fn test_sequence_always_succeed_select() {
        let behavior = Behavior::sequence(vec![
            Action(Dec),
            Action(Dec),
            AlwaysSucceed(Box::new(Behavior::select(vec![Action(Inc), Action(Dec)]))),
            Action(Dec),
        ]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 8);
        assert_eq!(g.node_count(), 9);
    }

    #[test]
    fn test_complex_while_select_sequence() {
        let _while = While(
            Box::new(Behavior::select(vec![
                Wait(5.0),
                Behavior::sequence(vec![Action(Inc), Action(Dec)]),
            ])),
            vec![Wait(0.5), Action(Inc), Action(Inc)],
        );

        let seq = Behavior::sequence(vec![Action(Inc), Action(Dec)]);

        let behavior = Behavior::select(vec![_while, Action(Inc), seq, Action(Dec)]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));
        assert_eq!(g.edge_count(), 16);
        assert_eq!(g.node_count(), 17);
    }

    #[test]
    fn test_while_invert() {
        let _while = While(
            Box::new(Behavior::select(vec![
                Wait(5.0),
                Behavior::sequence(vec![Action(Inc), Action(Dec)]),
            ])),
            vec![Wait(0.5), Action(Inc), Invert(Box::new(Wait(0.5)))],
        );

        // let _select = Behavior::select(vec![Wait(5.0), Behavior::sequence(vec![Action(Inc), Action(Dec)])]);
        let _select = Behavior::sequence(vec![Action(Inc), Action(Dec)]);

        let behavior = Behavior::select(vec![Invert(Box::new(_while)), _select, Action(Inc), Action(Dec)]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 18);
        assert_eq!(g.node_count(), 19);
    }

    #[test]
    fn test_if() {
        let condition = Behavior::sequence(vec![AlwaysSucceed(Box::new(Action(Inc))), Action(Dec)]);
        let behavior = If(
            Box::new(condition),
            Box::new(Action(Inc)), // if true
            Box::new(Action(Dec)), // else
        );

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));
        assert_eq!(g.edge_count(), 7);
        assert_eq!(g.node_count(), 8);
    }

    #[test]
    fn test_whenall_invert_whenany() {
        let behavior = WhenAll(vec![
            Action(Dec),
            Action(Dec),
            Invert(Box::new(WhenAny(vec![Action(Inc), Action(Dec)]))),
            Action(Dec),
        ]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 8);
        assert_eq!(g.node_count(), 9);
    }

    #[test]
    fn test_viz_after_action_wait() {
        let behavior = After(vec![
            Action(Dec),
            Wait(10.0),
            Action(Dec),
            WhenAny(vec![Wait(5.0), After(vec![Action(Inc), Action(Dec)])]),
        ]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 9);
        assert_eq!(g.node_count(), 10);
    }

    #[test]
    fn test_viz_memoryless_sequence() {
        let behavior = Behavior::memoryless_sequence(vec![
            Action(Dec),
            Action(Inc),
            Behavior::memoryless_sequence(vec![Action(Inc), Action(Dec)]),
        ]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        // Root + outer MemorylessSequence + 2 leaves + inner MemorylessSequence + 2 leaves = 7
        assert_eq!(g.node_count(), 7);
        assert_eq!(g.edge_count(), 6);
    }

    #[test]
    fn test_viz_memoryless_select() {
        let behavior = Behavior::memoryless_selector(vec![
            Action(Dec),
            Behavior::memoryless_selector(vec![Action(Inc), Action(Dec)]),
            Action(Inc),
        ]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        let (_, g) = bt.get_graphviz_with_graph_instance();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.node_count(), 7);
        assert_eq!(g.edge_count(), 6);
    }
}
