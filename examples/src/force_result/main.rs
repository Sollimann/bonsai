//! Demonstrates the `AlwaysSucceed` and `AlwaysFail` decorators.
//!
//! `AlwaysSucceed(A)` coerces a child's result to `Success` even when the child
//! reports `Failure` — useful for best-effort nodes (telemetry, logging) that
//! must not abort the surrounding `Sequence`.
//!
//! `AlwaysFail(A)` coerces a child's result to `Failure` even when the child
//! reports `Success` — useful for probes/guards that should always route the
//! tree down a fallback branch.
use bonsai_bt::Behavior::{Action, AlwaysFail, AlwaysSucceed};
use bonsai_bt::{Event, Status, UpdateArgs, BT};

#[derive(Clone, Debug)]
enum Op {
    /// Reports `Failure` (e.g. a telemetry send that failed).
    FlakyTelemetry,
    /// Reports `Success` (e.g. a probe that found the resource).
    Probe,
}

fn main() {
    let e: Event = UpdateArgs { dt: 0.0 }.into();

    // (1) AlwaysSucceed — the flaky telemetry node fails, but the decorator
    //     coerces it to Success so it doesn't abort its parent Sequence.
    let telemetry = AlwaysSucceed(Box::new(Action(Op::FlakyTelemetry)));
    let mut bt = BT::new(telemetry, ());
    let (status, _) = bt
        .tick(
            &e,
            &mut |args: bonsai_bt::ActionArgs<Event, Op>, _| match *args.action {
                Op::FlakyTelemetry => (Status::Failure, 0.0),
                Op::Probe => (Status::Success, 0.0),
            },
        )
        .expect("tree should yield a result");
    println!("AlwaysSucceed(flaky telemetry) -> {status:?} (expect Success)");

    // (2) AlwaysFail — the probe succeeds, but the decorator coerces it to
    //     Failure so control flow is forced down a fallback branch.
    let probe = AlwaysFail(Box::new(Action(Op::Probe)));
    let mut bt2 = BT::new(probe, ());
    let (status2, _) = bt2
        .tick(
            &e,
            &mut |args: bonsai_bt::ActionArgs<Event, Op>, _| match *args.action {
                Op::FlakyTelemetry => (Status::Failure, 0.0),
                Op::Probe => (Status::Success, 0.0),
            },
        )
        .expect("tree should yield a result");
    println!("AlwaysFail(probe) -> {status2:?} (expect Failure)");
}
