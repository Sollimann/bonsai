use std::collections::HashMap;

use crate::blackboard_tests::TestActions::{Dec, Inc, LessThan};
use bonsai::{Action, Event, Failure, Sequence, Success, UpdateArgs, Wait, BT};

/// Some test actions.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum TestActions {
    /// Increment accumulator.
    Inc,
    /// Decrement accumulator.
    Dec,
    ///, Check if less than
    LessThan(i32),
}

// A test state machine that can increment and decrement.
fn tick(mut acc: i32, dt: f64, bt: &mut BT<TestActions, String, i32>) -> i32 {
    let e: Event = UpdateArgs { dt }.into();

    let (_s, _t) = bt.state.event(&e, &mut |args| match *args.action {
        Inc => {
            acc += 1;
            (Success, args.dt)
        }
        Dec => {
            acc -= 1;
            (Success, args.dt)
        }
        LessThan(v) => {
            if acc < v {
                (Success, args.dt)
            } else {
                (Failure, args.dt)
            }
        }
    });

    // update counter in blackboard
    let bb = bt.get_blackboard();

    bb.get_db()
        .entry("count".to_string())
        .and_modify(|count| *count = acc)
        .or_insert(0)
        .to_owned()
}

#[test]
fn test_crate_bt() {
    let a: i32 = 0;
    let seq = Sequence(vec![Wait(1.0), Action(Inc), Wait(1.0), Action(Inc)]);

    let h: HashMap<String, i32> = HashMap::new();
    let mut bt = BT::new(seq, h);
    let a = tick(a, 0.5, &mut bt);
    assert_eq!(a, 0);
    let a = tick(a, 0.5, &mut bt);
    assert_eq!(a, 1);
    let a = tick(a, 0.5, &mut bt);
    assert_eq!(a, 1);
    let a = tick(a, 0.5, &mut bt);
    assert_eq!(a, 2);

    let bb = bt.get_blackboard();
    let count = bb.get_db().get("count").unwrap();
    assert_eq!(*count, 2);
}
