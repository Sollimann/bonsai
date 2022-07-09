#![allow(dead_code, unused_variables)]
use std::collections::HashMap;
use std::fmt::Debug;

use crate::{Behavior, State};

pub struct BT<A, B, C> {
    /// constructed behavior tree
    pub bt: State<A>,
    /// blackboard
    pub bb: HashMap<B, C>,
}

impl<A: Clone, B: Debug, C: Debug> BT<A, B, C> {
    pub fn new(behavior: Behavior<A>, blackboard: HashMap<B, C>) -> Self {
        let bt = State::new(behavior);
        Self { bt, bb: blackboard }
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
