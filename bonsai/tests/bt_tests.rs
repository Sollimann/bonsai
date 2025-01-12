use std::collections::HashMap;

use crate::bt_tests::TestActions::{Dec, Inc, LessThan};
use bonsai_bt::{Action, Behavior::Select, Event, Failure, Success, UpdateArgs, BT};

/// Some test actions.
#[derive(Clone, Debug)]
enum TestActions {
    /// Increment accumulator.
    Inc,
    /// Decrement accumulator.
    Dec,
    ///, Check if less than
    LessThan(i32),
}

// A test state machine that can increment and decrement.
fn tick(mut acc: i32, dt: f64, bt: &mut BT<TestActions, HashMap<String, i32>>) -> (i32, bonsai_bt::Status, f64) {
    let e: Event = UpdateArgs { dt }.into();
    println!("acc {}", acc);
    let (s, t) = bt.tick(&e, &mut |action, _, args| match *action {
        Inc => {
            acc += 1;
            (Success, args.dt)
        }
        Dec => {
            acc -= 1;
            (Success, args.dt)
        }
        LessThan(v) => {
            println!("inside less than with acc: {}", acc);
            if acc < v {
                println!("success {}<{}", acc, v);
                (Success, args.dt)
            } else {
                println!("failure {}>={}", acc, v);
                (Failure, args.dt)
            }
        }
    });
    println!("status: {:?} dt: {}", s, t);

    (acc, s, t)
}

#[test]
fn test_select_succeed_on_second_last() {
    let a: i32 = 3;
    let sel = Select(vec![Action(LessThan(1)), Action(Dec), Action(Inc)]);

    let h: HashMap<String, i32> = HashMap::new();
    let mut bt = BT::new(sel, h);

    let (a, s, _) = tick(a, 0.1, &mut bt);
    assert_eq!(a, 2);
    assert_eq!(s, Success);
    let (a, s, _) = tick(a, 0.1, &mut bt);
    assert_eq!(a, 1);
    assert_eq!(s, Success);
    let (a, s, _) = tick(a, 0.1, &mut bt);
    assert_eq!(a, 0);
    assert_eq!(s, Success);

    // reset bt
    bt.reset_bt();
    let (a, s, _) = tick(a, 0.1, &mut bt);
    assert_eq!(a, 0);
    assert_eq!(s, Success);
}
