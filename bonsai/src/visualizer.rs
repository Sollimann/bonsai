#![allow(dead_code, unused_imports, unused_variables)]
use crate::{Behavior, State, BT};
use petgraph::graph::Graph;
use std::fmt::Debug;

impl<A: Clone, K: Debug, V: Debug> BT<A, K, V> {
    pub fn parse(behavior: Behavior<A>) -> Self {
        match behavior {
            Behavior::Action(action) => todo!(),
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
