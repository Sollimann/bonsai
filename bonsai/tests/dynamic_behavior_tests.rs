use crate::dynamic_behavior_tests::TestActions::{DynamicWait, Inc};
use bonsai_bt::{Action, Event, State, Success, UpdateArgs, Wait, While, RUNNING};

type Times = Vec<f64>;
/// Some test actions.
#[derive(Clone, Debug)]
enum TestActions {
    /// Increment accumulator.
    Inc,
    ///, Dynamic timing
    DynamicWait(Times),
}

// A test state machine that can increment and decrement.
fn tick(mut acc: usize, dt: f64, t: &mut f64, counter: &mut usize, state: &mut State<TestActions>) -> usize {
    let e: Event = UpdateArgs { dt }.into();

    let (_s, _t) = state.tick(&e, &mut |args| match &*args.action {
        Inc => {
            acc += 1;
            (Success, args.dt)
        }
        DynamicWait(times) => {
            // reset dynamic timer
            if *counter >= times.len() {
                *counter = 0
            }

            let wait_t = times[counter.to_owned()];

            if *t + dt >= wait_t {
                let time_overdue = *t + dt - wait_t;
                *counter += 1;
                *t = -dt;
                (Success, time_overdue)
            } else {
                *t += dt;
                RUNNING
            }
        }
    });

    acc
}

#[test]
fn test_alter_wait_time() {
    let a: usize = 0;
    let mut counter = 0;
    let mut timer: f64 = 0.0;
    let rep = While(
        Box::new(Wait(50.0)),
        vec![Action(DynamicWait(vec![1.0, 2.0, 3.0])), Action(Inc)],
    );
    let mut state = State::new(rep);

    // time passed=1.0
    let a = tick(a, 1.0, &mut timer, &mut counter, &mut state);
    assert_eq!(a, 1);
    // time passed=2.5
    let a = tick(a, 1.5, &mut timer, &mut counter, &mut state);
    assert_eq!(a, 1);
    // time passed = 5.50001
    let a = tick(a, 3.0001, &mut timer, &mut counter, &mut state);
    assert_eq!(a, 2);
    // time passed = 12.50002
    let a = tick(a, 7.0001, &mut timer, &mut counter, &mut state);
    assert_eq!(a, 3);
}
