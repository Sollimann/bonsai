use std::collections::HashMap;

use bonsai_bt::{Action, Event, Float, Sequence, Success, UpdateArgs, Wait, BT};

use crate::blackboard_tests::TestActions::{Dec, Inc};

/// Some test actions.
#[derive(Clone, Debug, Copy)]
pub enum TestActions {
    /// Increment accumulator.
    Inc,
    /// Decrement accumulator.
    Dec,
}

// A test state machine that can increment and decrement.
fn tick(mut acc: i32, dt: Float, bt: &mut BT<TestActions, HashMap<String, i32>>) -> i32 {
    let e: Event = UpdateArgs { dt }.into();

    let (_s, _t) = bt
        .tick(&e, &mut |args, _| match *args.action {
            Inc => {
                acc += 1;
                (Success, args.dt)
            }
            Dec => {
                acc -= 1;
                (Success, args.dt)
            }
        })
        .unwrap();

    // update counter in blackboard
    let bb = bt.blackboard_mut();

    bb.entry("count".to_string())
        .and_modify(|count| *count = acc)
        .or_insert(0)
        .to_owned()
}

#[test]
fn test_crate_bt() {
    let a: i32 = 0;
    let seq = Sequence(vec![
        Wait(1.0),
        Action(Inc),
        Wait(1.0),
        Action(Inc),
        Wait(0.5),
        Action(Dec),
    ]);

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
    let a = tick(a, 0.5, &mut bt);
    assert_eq!(a, 1);

    let bb = bt.blackboard_mut();
    let count = bb.get("count").unwrap();
    assert_eq!(*count, 1);
}
