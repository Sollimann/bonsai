//! Demonstrates the `Retry` and `Repeat` decorators.
//!
//! `Retry(n, child)` re-runs `child` until it succeeds, up to `n` attempts in
//! total. `Repeat(n, child)` re-runs `child` up to `n` times as long as it
//! keeps succeeding, and stops on the first `Failure`.
//!
//! Both restart their child from its initial state on every attempt, so a
//! `Wait` inside them waits the full duration each time.
use std::cell::Cell;

use bonsai_bt::Behavior::{Action, Repeat, Retry, Wait};
use bonsai_bt::{Event, Status, UpdateArgs, BT};

fn main() {
    let e: Event = UpdateArgs { dt: 1.0 }.into();

    // (1) Retry(3, flaky): the third attempt succeeds.
    let attempts = Cell::new(0u32);
    let mut bt = BT::new(Retry(3, Box::new(Action(()))), ());
    let result = bt
        .tick(&e, &mut |_, _| {
            attempts.set(attempts.get() + 1);
            if attempts.get() < 3 {
                (Status::Failure, 0.0)
            } else {
                (Status::Success, 0.0)
            }
        })
        .unwrap();
    println!(
        "Retry(3, flaky) -> {result:?} after {} attempt(s) (expect Success after 3)",
        attempts.get()
    );

    // (2) Retry(2, always failing): the budget is exhausted -> Failure.
    let mut bt = BT::new(Retry(2, Box::new(Action(()))), ());
    let result = bt.tick(&e, &mut |_, _| (Status::Failure, 0.0)).unwrap();
    println!("Retry(2, always failing) -> {result:?} (expect Failure)");

    // (3) Repeat(2, Wait(1.0)): the child restarts after each completion, so
    // the second repetition waits a fresh second.
    let mut bt: BT<(), ()> = BT::new(Repeat(2, Box::new(Wait(1.0))), ());
    println!(
        "Repeat(2, Wait(1.0)) @ 1.0s -> {:?} (expect Running)",
        bt.tick(&e, &mut |_, _| (Status::Running, 0.0))
    );
    println!(
        "Repeat(2, Wait(1.0)) @ 2.0s -> {:?} (expect Success)",
        bt.tick(&e, &mut |_, _| (Status::Running, 0.0))
    );
}
