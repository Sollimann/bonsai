#![allow(dead_code, unused_imports, unused_variables)]
use crate::{Behavior, State, BT};
use petgraph::{graph::Graph, stable_graph::NodeIndex};
use std::{collections::VecDeque, fmt::Debug};
impl<A: Clone + Debug, K: Debug, V: Debug> BT<A, K, V> {
    pub(crate) fn gen_graph(&mut self, mut queue: VecDeque<Behavior<A>>, prev_node: NodeIndex) {
        if queue.is_empty() {
            return;
        }

        let behavior = queue.pop_front().unwrap();

        match behavior {
            Behavior::Action(action) => {
                let node_id = self.graph.add_node(format!("Action({:?})", action));
                self.graph.add_edge(prev_node, node_id, 1);
                self.gen_graph(queue, prev_node)
            }
            Behavior::Invert(ev) => todo!(),
            Behavior::AlwaysSucceed(ev) => todo!(),
            Behavior::Wait(dt) => todo!(),
            Behavior::WaitForever => todo!(),
            Behavior::If(condition, success, failure) => todo!(),
            Behavior::Select(sel) => {
                let node_id = self.graph.add_node("Select".to_string());
                self.graph.add_edge(prev_node, node_id, 1);
                queue.append(&mut VecDeque::from(sel));
                self.gen_graph(queue, node_id)
            }
            Behavior::Sequence(seq) => {
                let node_id = self.graph.add_node("Sequence".to_string());
                self.graph.add_edge(prev_node, node_id, 1);
                queue.append(&mut VecDeque::from(seq));
                self.gen_graph(queue, node_id)
            }
            Behavior::While(ev, rep) => todo!(),
            Behavior::WhenAll(all) => todo!(),
            Behavior::WhenAny(all) => todo!(),
            Behavior::After(seq) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Behavior::{Action, Select, Sequence};
    use crate::Status::{self, Success};
    use crate::{Event, UpdateArgs};

    use super::*;
    use crate::visualizer::tests::TestActions::{Dec, Inc};
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
        use petgraph::dot::{Config, Dot};
        use petgraph::Graph;

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
}