#![allow(dead_code, unused_variables)]
use std::collections::HashMap;
use std::fmt::Debug;

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
