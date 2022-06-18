use crate::behavior_tests::TestActions::{Dec, Inc};
use b3::{Action, Event, Sequence, State, Success, UpdateArgs, Wait, WaitForever, WhenAll, While};

/// Some test actions.
#[derive(Clone)]
#[allow(dead_code)]
pub enum TestActions {
    /// Increment accumulator.
    Inc,
    /// Decrement accumulator.
    Dec,
}

// A test state machine that can increment and decrement.
fn exec(mut acc: u32, dt: f64, state: &mut State<TestActions, ()>) -> u32 {
    let e: Event = UpdateArgs { dt }.into();
    state.event(&e, &mut |args| match *args.action {
        Inc => {
            acc += 1;
            (Success, args.dt)
        }
        Dec => {
            acc -= 1;
            (Success, args.dt)
        }
    });
    acc
}

// Each action that terminates immediately
// consumes a time of 0.0 seconds.
// This makes it possible to execute one action
// after another without delay or waiting for next update.
#[test]
fn test_increment_twice_in_sequence() {
    let a: u32 = 0;
    let seq = Sequence(vec![Action(Inc), Action(Inc)]);
    let mut state = State::new(seq);
    let a = exec(a, 0.0, &mut state);
    assert_eq!(a, 2);
}

// If you wait the exact amount before to execute an action,
// it will execute. This behavior makes it easy to predict
// when an action will run.
#[test]
fn wait_sec() {
    let a: u32 = 0;
    let seq = Sequence(vec![Wait(1.0), Action(Inc)]);
    let mut state = State::new(seq);
    let a = exec(a, 1.0, &mut state);
    assert_eq!(a, 1);
}

// When we execute half the time and then the other half,
// then the action should be executed.
#[test]
fn wait_half_sec() {
    let a: u32 = 0;
    let seq = Sequence(vec![Wait(1.0), Action(Inc)]);
    let mut state = State::new(seq);
    let a = exec(a, 0.5, &mut state);
    assert_eq!(a, 0);
    let a = exec(a, 0.5, &mut state);
    assert_eq!(a, 1);
}

// A sequence of one event is like a bare event.
#[test]
fn sequence_of_one_event() {
    let a: u32 = 0;
    let seq = Sequence(vec![Action(Inc)]);
    let mut state = State::new(seq);
    let a = exec(a, 1.0, &mut state);
    assert_eq!(a, 1);
}

// A sequence of wait events is the same as one wait event.
#[test]
fn wait_two_waits() {
    let a: u32 = 0;
    let seq = Sequence(vec![Wait(0.5), Wait(0.5), Action(Inc)]);
    let mut state = State::new(seq);
    let a = exec(a, 1.0, &mut state);
    assert_eq!(a, 1);
}

// Increase counter ten times.
#[test]
fn loop_ten_times() {
    let a: u32 = 0;
    let rep = While(Box::new(Wait(50.0)), vec![Wait(0.5), Action(Inc), Wait(0.5)]);
    let mut state = State::new(rep);
    let a = exec(a, 10.0, &mut state);
    assert_eq!(a, 10);
}

#[test]
fn when_all_wait() {
    let a: u32 = 0;
    let all = Sequence(vec![
        // Wait in parallel.
        WhenAll(vec![Wait(0.5), Wait(1.0)]),
        Action(Inc),
    ]);
    let mut state = State::new(all);
    let a = exec(a, 0.5, &mut state);
    assert_eq!(a, 0);
    let a = exec(a, 0.5, &mut state);
    assert_eq!(a, 1);
}

#[test]
fn while_wait_sequence() {
    let mut a: u32 = 0;
    let w = While(
        Box::new(Wait(9.999999)),
        vec![Sequence(vec![Wait(0.5), Action(Inc), Wait(0.5), Action(Inc)])],
    );
    let mut state = State::new(w);
    for _ in 0..100 {
        a = exec(a, 0.1, &mut state);
    }
    // The last increment is never executed, because there is not enough time.
    assert_eq!(a, 19);
}

#[test]
fn while_wait_forever_sequence() {
    let mut a: u32 = 0;
    let w = While(Box::new(WaitForever), vec![Sequence(vec![Action(Inc), Wait(1.0)])]);
    let mut state = State::new(w);
    a = exec(a, 1.001, &mut state);
    assert_eq!(a, 2);
}
