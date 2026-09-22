//! Demonstrates the `Timeout` decorator.
//!
//! `Timeout(limit, child)` runs `child`, but if `child` is still `Running`
//! once the accumulated time reaches `limit`, it is abandoned and `Failure`
//! is returned. If `child` finishes (success or failure) before the limit,
//! its status is passed through unchanged.
use bonsai_bt::Behavior::{Timeout, Wait};
use bonsai_bt::{Event, Float, Status, UpdateArgs, BT};

/// Advance the tree by `dt` seconds and return the resulting `(Status, Float)`.
fn tick(bt: &mut BT<(), ()>, dt: Float) -> Option<(Status, Float)> {
    let e: Event = UpdateArgs { dt }.into();
    // No `Action` nodes in these trees, so the callback is never invoked.
    bt.tick(&e, &mut |_, _| (Status::Running, 0.0))
}

fn main() {
    // (1) A child that keeps Running past the limit is cut off -> Failure.
    let mut bt = BT::new(Timeout(1.0, Box::new(Wait(5.0))), ());
    println!(
        "Timeout(1.0, Wait(5.0)) @ 0.6s -> {:?} (expect Running)",
        tick(&mut bt, 0.6)
    );
    println!(
        "Timeout(1.0, Wait(5.0)) @ 1.2s -> {:?} (expect Failure)",
        tick(&mut bt, 0.6)
    );

    // (2) A child that finishes before the limit passes through unchanged.
    let mut bt2 = BT::new(Timeout(10.0, Box::new(Wait(0.5))), ());
    println!(
        "Timeout(10.0, Wait(0.5)) @ 0.2s -> {:?} (expect Running)",
        tick(&mut bt2, 0.2)
    );
    println!(
        "Timeout(10.0, Wait(0.5)) @ 0.6s -> {:?} (expect Success)",
        tick(&mut bt2, 0.4)
    );
}
