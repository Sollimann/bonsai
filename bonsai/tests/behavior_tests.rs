use crate::behavior_tests::TestActions::{Dec, Inc, LessThan};
use bonsai_bt::{
    Action, Behavior::If, Event, Failure, Sequence, State, Success, UpdateArgs, Wait, WaitForever, WhenAll, While,
};

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
fn tick(mut acc: i32, dt: f64, state: &mut State<TestActions>) -> i32 {
    let e: Event = UpdateArgs { dt }.into();

    let (s, t) = state.tick(&e, &mut |args| match *args.action {
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
    println!("status: {:?} dt: {}", s, t);

    acc
}

// A test state machine that can increment and decrement.
fn tick_with_ref(acc: &mut i32, dt: f64, state: &mut State<TestActions>) {
    let e: Event = UpdateArgs { dt }.into();
    state.tick(&e, &mut |args| match *args.action {
        Inc => {
            *acc += 1;
            (Success, args.dt)
        }
        Dec => {
            *acc -= 1;
            (Success, args.dt)
        }
        LessThan(_) => todo!(),
    });
}

// Each action that terminates immediately
// consumes a time of 0.0 seconds.
// This makes it possible to execute one action
// after another without delay or waiting for next update.
#[test]
fn test_immediate_termination() {
    let mut a: i32 = 0;

    let seq = Sequence(vec![Action(Inc), Action(Inc)]);
    let mut state = State::new(seq);
    tick_with_ref(&mut a, 0.0, &mut state);
    assert_eq!(a, 2);
    tick_with_ref(&mut a, 1.0, &mut state);
    assert_eq!(a, 2)
}

// Tree terminates after 2.001 seconds
// This makes it possible to execute several actions within
// the lifetime of the behavior tree
#[test]
fn while_wait_sequence_twice() {
    let mut a: i32 = 0;
    let w = While(
        Box::new(Wait(2.001)),
        vec![Sequence(vec![Wait(0.5), Action(Inc), Wait(0.5), Action(Inc)])],
    );
    let mut state = State::new(w);
    tick_with_ref(&mut a, 1.0, &mut state);
    assert_eq!(a, 2);
    tick_with_ref(&mut a, 1.0, &mut state);
    assert_eq!(a, 4);
    tick_with_ref(&mut a, 1.0, &mut state);
    assert_eq!(a, 4);
}

// If you wait the exact amount before to execute an action,
// it will execute. This behavior makes it easy to predict
// when an action will run.
#[test]
fn wait_sec() {
    let a: i32 = 0;
    let seq = Sequence(vec![Wait(1.0), Action(Inc)]);
    let mut state = State::new(seq);
    let a = tick(a, 1.0, &mut state);
    assert_eq!(a, 1);
}

// When we execute half the time and then the other half,
// then the action should be executed.
#[test]
fn wait_half_sec() {
    let a: i32 = 0;
    let seq = Sequence(vec![Wait(1.0), Action(Inc)]);
    let mut state = State::new(seq);
    let a = tick(a, 0.5, &mut state);
    assert_eq!(a, 0);
    let a = tick(a, 0.5, &mut state);
    assert_eq!(a, 1);
}

// A sequence of one tick is like a bare tick.
#[test]
fn sequence_of_one_event() {
    let a: i32 = 0;
    let seq = Sequence(vec![Action(Inc)]);
    let mut state = State::new(seq);
    let a = tick(a, 1.0, &mut state);
    assert_eq!(a, 1);
}

// A sequence of wait events is the same as one wait tick.
#[test]
fn wait_two_waits() {
    let a: i32 = 0;
    let seq = Sequence(vec![Wait(0.5), Wait(0.5), Action(Inc)]);
    let mut state = State::new(seq);
    let a = tick(a, 1.0, &mut state);
    assert_eq!(a, 1);
}

// Increase counter ten times.
#[test]
fn loop_ten_times() {
    let a: i32 = 0;
    let rep = While(Box::new(Wait(50.0)), vec![Wait(0.5), Action(Inc), Wait(0.5)]);
    let mut state = State::new(rep);

    // sample after 10 seconds
    let a = tick(a, 10.0, &mut state);
    assert_eq!(a, 10);
}

#[test]
fn when_all_wait() {
    let a: i32 = 0;
    let all = Sequence(vec![
        // Wait in parallel.
        WhenAll(vec![Wait(0.5), Wait(1.0)]),
        Action(Inc),
    ]);
    let mut state = State::new(all);
    let a = tick(a, 0.5, &mut state);
    assert_eq!(a, 0);
    let a = tick(a, 0.5, &mut state);
    assert_eq!(a, 1);
}

#[test]
fn while_wait_sequence() {
    let mut a: i32 = 0;
    let w = While(
        Box::new(Wait(9.999999)),
        vec![Sequence(vec![Wait(0.5), Action(Inc), Wait(0.5), Action(Inc)])],
    );
    let mut state = State::new(w);
    for _ in 0..100 {
        a = tick(a, 0.1, &mut state);
    }
    // The last increment is never executed, because there is not enough time.
    assert_eq!(a, 19);
}

#[test]
fn while_wait_forever_sequence() {
    let mut a: i32 = 0;
    let w = While(Box::new(WaitForever), vec![Sequence(vec![Action(Inc), Wait(1.0)])]);
    let mut state = State::new(w);
    a = tick(a, 1.001, &mut state);
    assert_eq!(a, 2);
}

#[test]
fn if_less_than() {
    let mut a: i32 = 0;
    let inc = Sequence(vec![Action(Inc), Action(Inc)]);
    let dec = Sequence(vec![Action(Dec), Action(Dec)]);
    let _if = If(Box::new(Action(LessThan(0))), Box::new(inc), Box::new(dec));
    let mut state = State::new(_if);

    a = tick(a, 0.0, &mut state);
    assert_eq!(a, -2);
}

#[test]
fn when_all_if() {
    let mut a: i32 = 0;
    let inc = Sequence(vec![Action(Inc), Action(Inc)]);
    let dec = Sequence(vec![Action(Dec), Action(Dec)]);
    let _if = If(Box::new(Action(LessThan(1))), Box::new(inc), Box::new(dec));

    // Run sequence over and over for 2 seconds
    let _while = While(Box::new(Wait(50.0)), vec![Wait(0.5), Action(Inc), Wait(0.5)]);

    let w = WhenAll(vec![_if, _while]);
    let mut state = State::new(w);

    // sample state after 8 seconds
    a = tick(a, 8.0, &mut state);
    assert_eq!(a, 10);

    // // sample state after 10 seconds
    a = tick(a, 2.0, &mut state);
    assert_eq!(a, 12);
}

#[test]
fn test_alter_wait_time() {
    let a: i32 = 0;
    let rep = While(Box::new(Wait(50.0)), vec![Wait(0.5), Action(Inc), Wait(0.5)]);
    let mut state = State::new(rep);

    // sample after 10 seconds
    let a = tick(a, 10.0, &mut state);
    assert_eq!(a, 10);
}
