#![allow(dead_code, unused_variables)]
use std::collections::HashMap;
use std::fmt::Debug;

use crate::{Behavior, State};

#[derive(Clone, Debug)]
pub struct BlackBoard<K, V>(HashMap<K, V>);

impl<K, V> BlackBoard<K, V> {
    pub fn get_db(&mut self) -> &mut HashMap<K, V> {
        &mut self.0
    }
}

pub struct BT<A, K, V> {
    /// constructed behavior tree
    pub state: State<A>,
    /// blackboard
    bb: BlackBoard<K, V>,
}

impl<A: Clone, K: Debug, V: Debug> BT<A, K, V> {
    pub fn new(behavior: Behavior<A>, blackboard: HashMap<K, V>) -> Self {
        let bt = State::new(behavior);
        Self {
            state: bt,
            bb: BlackBoard(blackboard),
        }
    }

    pub fn get_blackboard(&mut self) -> &mut BlackBoard<K, V> {
        &mut self.bb
    }

    pub fn _get_blackboard(bt: &mut BT<A, K, V>) -> &mut BlackBoard<K, V> {
        &mut bt.bb
    }

    fn _get_state(&mut self) -> &mut State<A> {
        &mut self.state
    }

    pub fn get_state(bt: &mut BT<A, K, V>) -> &mut State<A> {
        &mut bt.state
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_bb() {
        println!("placeholder")
    }
}
