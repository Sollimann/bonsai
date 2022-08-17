#![allow(dead_code, unused_imports, unused_variables)]
use crate::{Behavior, Sequence, State, BT};
use petgraph::{graph::Graph, stable_graph::NodeIndex, Direction::Outgoing};
use std::{collections::VecDeque, fmt::Debug};

#[derive(Debug, Clone)]
pub(crate) enum NodeType<A> {
    Root,
    Wait(f64),
    WaitForever,
    Action(A),
    Invert,
    AlwaysSucceed,
    Select,
    If,
    Sequence,
    While,
    WhenAll,
    WhenAny,
    After,
}

impl<A: Clone + Debug, K: Debug, V: Debug> BT<A, K, V> {
    pub(crate) fn gen_graph(&mut self, queue: &mut VecDeque<Behavior<A>>, prev_node: NodeIndex) {
        if queue.is_empty() {
            return;
        }

        let behavior = queue.pop_front().unwrap();

        match behavior {
            Behavior::Action(action) => {
                println!("Action: {:?}", action);
                let node_id = self.graph.add_node(NodeType::Action(action));
                self.graph.add_edge(prev_node, node_id, 1);
                self.gen_graph(queue, prev_node)
            }
            Behavior::Invert(ev) => {
                println!("Invert: {:?}", ev);
                let node_id = self.graph.add_node(NodeType::Invert);
                self.graph.add_edge(prev_node, node_id, 1);

                // invert node descendants
                let mut invert_descendants_queue: VecDeque<Behavior<A>> = VecDeque::new();
                invert_descendants_queue.push_back(*ev);
                self.gen_graph(&mut invert_descendants_queue, node_id);

                // right queue
                self.gen_graph(queue, prev_node)
            }
            Behavior::AlwaysSucceed(ev) => todo!(),
            Behavior::Wait(dt) => {
                let node_id = self.graph.add_node(NodeType::Wait(dt));
                self.graph.add_edge(prev_node, node_id, 1);
                self.gen_graph(queue, prev_node)
            }
            Behavior::WaitForever => todo!(),
            Behavior::If(condition, success, failure) => todo!(),
            Behavior::Select(sel) => {
                println!("Select: {:?}", sel);
                let node_id = self.graph.add_node(NodeType::Select);
                self.graph.add_edge(prev_node, node_id, 1);
                queue.append(&mut VecDeque::from(sel));
                self.gen_graph(queue, node_id)
            }
            Behavior::Sequence(seq) => {
                println!("seq: {:?}", seq);
                let node_id = self.graph.add_node(NodeType::Sequence);
                self.graph.add_edge(prev_node, node_id, 1);
                queue.append(&mut VecDeque::from(seq));
                self.gen_graph(queue, node_id)
            }
            Behavior::While(ev, seq) => {
                let node_id = self.graph.add_node(NodeType::While);
                self.graph.add_edge(prev_node, node_id, 1);

                queue.push_back(*ev);
                self.gen_graph(queue, node_id);
                queue.append(&mut VecDeque::from(vec![Sequence(seq)]));
                self.gen_graph(queue, node_id);
            }
            Behavior::WhenAll(all) => todo!(),
            Behavior::WhenAny(all) => todo!(),
            Behavior::After(seq) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualizer::tests::TestActions::{Dec, Inc};
    use crate::Behavior::{Action, After, Invert, Select, Sequence, Wait, WaitForever, WhenAll, WhenAny, While};
    use crate::Status::{self, Success};
    use crate::{Event, UpdateArgs};
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
    fn tick(mut acc: i32, dt: f64, bt: &mut BT<TestActions, String, i32>) -> (i32, Status, f64) {
        let e: Event = UpdateArgs { dt }.into();
        let (s, t) = bt.state.tick(&e, &mut |args| match args.action {
            TestActions::Inc => {
                acc += 1;
                (Success, args.dt)
            }
            TestActions::Dec => {
                acc -= 1;
                (Success, args.dt)
            }
        });
        (acc, s, t)
    }

    #[test]
    fn test_viz_sequence_and_action() {
        use petgraph::dot::{Config, Dot};
        use petgraph::Graph;

        let behavior = Sequence(vec![
            Action(Dec),
            Action(Dec),
            Sequence(vec![Action(Inc), Sequence(vec![Action(Inc)])]),
        ]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        bt.generate_graph();
        let g = bt.graph.clone();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 7);
        assert_eq!(g.node_count(), 8);
    }

    #[test]
    fn test_viz_select_and_action() {
        let behavior = Select(vec![
            Action(Dec),
            Action(Dec),
            Select(vec![Action(Inc), Sequence(vec![Action(Inc), Action(Dec)])]),
        ]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        bt.generate_graph();
        let g = bt.graph.clone();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 8);
        assert_eq!(g.node_count(), 9);
    }

    #[test]
    fn test_viz_sequence_action_wait() {
        let behavior = Sequence(vec![
            Action(Dec),
            Wait(10.0),
            Action(Dec),
            Select(vec![Wait(5.0), Sequence(vec![Action(Inc), Action(Dec)])]),
        ]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        bt.generate_graph();
        let g = bt.graph.clone();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 9);
        assert_eq!(g.node_count(), 10);
    }

    #[test]
    fn test_viz_while() {
        let behavior = While(Box::new(Wait(50.0)), vec![Wait(0.5), Action(Inc), Wait(0.5)]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        bt.generate_graph();
        let g = bt.graph.clone();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 6);
        assert_eq!(g.node_count(), 7);
    }

    #[test]
    fn test_viz_while_select_sequence_wait() {
        let behavior = While(
            Box::new(Select(vec![Wait(5.0), Sequence(vec![Action(Inc), Action(Dec)])])),
            vec![Wait(0.5), Action(Inc), Wait(0.5)],
        );

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        bt.generate_graph();
        let g = bt.graph.clone();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 10);
        assert_eq!(g.node_count(), 11);
    }

    #[test]
    fn test_invert() {
        let behavior = Sequence(vec![Invert(Box::new(Action(Inc))), Action(Dec)]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        bt.generate_graph();
        let g = bt.graph.clone();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 4);
        assert_eq!(g.node_count(), 5);
    }

    #[test]
    fn test_sequence_invert_select() {
        let behavior = Sequence(vec![
            Invert(Box::new(Select(vec![Action(Inc), Action(Dec)]))),
            Action(Dec),
            Action(Inc),
            Action(Dec),
        ]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        bt.generate_graph();
        let g = bt.graph.clone();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        assert_eq!(g.edge_count(), 8);
        assert_eq!(g.node_count(), 9);
    }

    #[test]
    fn test_complex_while_select_sequence() {
        let _while = While(
            Box::new(Select(vec![Wait(5.0), Sequence(vec![Action(Inc), Action(Dec)])])),
            vec![Wait(0.5), Action(Inc), Action(Inc)],
        );

        let seq = Sequence(vec![Action(Inc), Action(Dec)]);

        let behavior = Select(vec![Action(Inc), seq, Action(Dec)]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        bt.generate_graph();
        let g = bt.graph.clone();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));
    }

    #[test]
    fn test_while_invert() {
        let _while = While(
            Box::new(Select(vec![Wait(5.0), Sequence(vec![Action(Inc), Action(Dec)])])),
            vec![Wait(0.5), Action(Inc), Invert(Box::new(Wait(0.5)))],
        );

        // let _select = Select(vec![Wait(5.0), Sequence(vec![Action(Inc), Action(Dec)])]);
        let _select = Sequence(vec![Action(Inc), Action(Dec)]);

        let behavior = Select(vec![Invert(Box::new(_while)), _select, Action(Inc), Action(Dec)]);

        let h: HashMap<String, i32> = HashMap::new();
        let mut bt = BT::new(behavior, h);
        bt.generate_graph();
        let g = bt.graph.clone();

        println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));

        // assert_eq!(g.edge_count(), 8);
        // assert_eq!(g.node_count(), 9);
    }
}
