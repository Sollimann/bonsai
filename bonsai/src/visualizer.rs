#![allow(dead_code, unused_imports, unused_variables)]
use crate::{Behavior, State, BT};
use petgraph::{graph::Graph, stable_graph::NodeIndex};
use std::fmt::Debug;

impl<A: Clone + Debug, K: Debug, V: Debug> BT<A, K, V> {
    pub(crate) fn gen_graph(&mut self, behavior: Behavior<A>, prev_node: NodeIndex) -> State<A> {
        match behavior {
            Behavior::Action(action) => {
                let node_id = self.graph.add_node(format!("Action({:?})", action));
                self.graph.add_edge(prev_node, node_id, 1);
                State::ActionState(action)
            }
            Behavior::Invert(ev) => todo!(),
            Behavior::AlwaysSucceed(ev) => todo!(),
            Behavior::Wait(dt) => todo!(),
            Behavior::WaitForever => todo!(),
            Behavior::If(condition, success, failure) => todo!(),
            Behavior::Select(sel) => todo!(),
            Behavior::Sequence(seq) => todo!(),
            Behavior::While(ev, rep) => todo!(),
            Behavior::WhenAll(all) => todo!(),
            Behavior::WhenAny(all) => todo!(),
            Behavior::After(seq) => todo!(),
        }
    }
}
