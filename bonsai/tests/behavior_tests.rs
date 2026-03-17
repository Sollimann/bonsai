use crate::behavior_tests::TestActions::{Dec, Inc, LessThan, LessThanRunningSuccess};
use bonsai_bt::{
    Action, ActionArgs, After, AlwaysSucceed, Event, Failure, Float, If, Invert, Race, Select, Sequence,
    Status::Running, Success, UpdateArgs, Wait, WaitForever, WhenAll, WhenAny, While, WhileAll, BT,
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
    /// Check if less than and return [Running]. If more or equal return [Success].
    LessThanRunningSuccess(i32),
}

// A test state machine that can increment and decrement.
fn tick(mut acc: i32, dt: Float, state: &mut BT<TestActions, ()>) -> (i32, bonsai_bt::Status, Float) {
    let e: Event = UpdateArgs { dt }.into();
    println!("acc {}", acc);
    let (s, t) = state
        .tick(&e, &mut |args: ActionArgs<Event, TestActions>, _| match *args.action {
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
            TestActions::LessThanRunningSuccess(v) => {
                println!("inside LessThanRunningSuccess with acc: {}", acc);
                if acc < v {
                    println!("success {}<{}", acc, v);
                    (Running, args.dt)
                } else {
                    println!("failure {}>={}", acc, v);
                    (Success, args.dt)
                }
            }
        })
        .unwrap();
    println!("status: {:?} dt: {}", s, t);

    (acc, s, t)
}

// A test state machine that can increment and decrement.
fn tick_with_ref(acc: &mut i32, dt: Float, state: &mut BT<TestActions, ()>) {
    let e: Event = UpdateArgs { dt }.into();

    state
        .tick(&e, &mut |args: ActionArgs<Event, TestActions>, _| match *args.action {
            Inc => {
                *acc += 1;
                (Success, args.dt)
            }
            Dec => {
                *acc -= 1;
                (Success, args.dt)
            }
            TestActions::LessThanRunningSuccess(_) | LessThan(_) => todo!(),
        })
        .unwrap();
}

// Each action that terminates immediately
// consumes a time of 0.0 seconds.
// This makes it possible to execute one action
// after another without delay or waiting for next update.
#[test]
fn test_immediate_termination() {
    let mut a: i32 = 0;

    let seq = Sequence(vec![Action(Inc), Action(Inc)]);
    let mut state = BT::new(seq, ());
    tick_with_ref(&mut a, 0.0, &mut state);
    assert_eq!(a, 2);
    assert!(state.is_finished());
    state.reset_bt();
    tick_with_ref(&mut a, 1.0, &mut state);
    assert_eq!(a, 4);
    assert!(state.is_finished());
    state.reset_bt();
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
    let mut state = BT::new(w, ());
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
    let mut state = BT::new(seq, ());
    let (a, _, _) = tick(a, 1.0, &mut state);
    assert_eq!(a, 1);
}

// When we execute half the time and then the other half,
// then the action should be executed.
#[test]
fn wait_half_sec() {
    let a: i32 = 0;
    let seq = Sequence(vec![Wait(1.0), Action(Inc)]);
    let mut state = BT::new(seq, ());
    let (a, _, _) = tick(a, 0.5, &mut state);
    assert_eq!(a, 0);
    let (a, _, _) = tick(a, 0.5, &mut state);
    assert_eq!(a, 1);
}

// A sequence of one tick is like a bare tick.
#[test]
fn sequence_of_one_event() {
    let a: i32 = 0;
    let seq = Sequence(vec![Action(Inc)]);
    let mut state = BT::new(seq, ());
    let (a, _, _) = tick(a, 1.0, &mut state);
    assert_eq!(a, 1);
}

// A sequence of wait events is the same as one wait tick.
#[test]
fn wait_two_waits() {
    let a: i32 = 0;
    let seq = Sequence(vec![Wait(0.5), Wait(0.5), Action(Inc)]);
    let mut state = BT::new(seq, ());
    let (a, _, _) = tick(a, 1.0, &mut state);
    assert_eq!(a, 1);
}

// Increase counter ten times.
#[test]
fn loop_ten_times() {
    let a: i32 = 0;
    let rep = While(Box::new(Wait(50.0)), vec![Wait(0.5), Action(Inc), Wait(0.5)]);
    let mut state = BT::new(rep, ());

    // sample after 10 seconds
    let (a, _, _) = tick(a, 10.0, &mut state);
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
    let mut state = BT::new(all, ());
    let (a, _, _) = tick(a, 0.5, &mut state);
    assert_eq!(a, 0);
    let (a, _, _) = tick(a, 0.5, &mut state);
    assert_eq!(a, 1);
}

#[test]
fn while_wait_sequence() {
    let mut a: i32 = 0;
    let w = While(
        Box::new(Wait(9.999999)),
        vec![Sequence(vec![Wait(0.5), Action(Inc), Wait(0.5), Action(Inc)])],
    );
    let mut state = BT::new(w, ());
    for _ in 0..100 {
        (a, _, _) = tick(a, 0.1, &mut state);
    }
    // The last increment is never executed, because there is not enough time.
    assert_eq!(a, 19);
}

#[test]
fn while_wait_forever_sequence() {
    let mut a: i32 = 0;
    let w = While(Box::new(WaitForever), vec![Sequence(vec![Action(Inc), Wait(1.0)])]);
    let mut state = BT::new(w, ());
    (a, _, _) = tick(a, 1.001, &mut state);
    assert_eq!(a, 2);
}

#[test]
fn test_if_less_than() {
    let a: i32 = 3;
    let _if = If(
        Box::new(Action(LessThan(1))),
        Box::new(Action(Inc)), // if true
        Box::new(Action(Dec)), // else
    );

    let mut state = BT::new(_if, ());

    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 2);
    assert_eq!(s, Success);
    state.reset_bt();
    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 1);
    assert_eq!(s, Success);
    state.reset_bt();
    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 0);
    assert_eq!(s, Success);
    state.reset_bt();
    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 1);
    assert_eq!(s, Success);
}

#[test]
fn when_all_if() {
    let a: i32 = 0;
    let inc = Sequence(vec![Action(Inc), Action(Inc)]);
    let dec = Sequence(vec![Action(Dec), Action(Dec)]);
    let _if = If(Box::new(Action(LessThan(1))), Box::new(inc), Box::new(dec));

    // Run sequence over and over for 2 seconds
    let _while = While(Box::new(Wait(50.0)), vec![Wait(0.5), Action(Inc), Wait(0.5)]);

    let w = WhenAll(vec![_if, _while]);
    let mut state = BT::new(w, ());

    // sample state after 8 seconds
    let (a, _, _) = tick(a, 8.0, &mut state);
    assert_eq!(a, 10);

    // // sample state after 10 seconds
    let (a, _, _) = tick(a, 2.0, &mut state);
    assert_eq!(a, 12);
}

#[test]
fn test_alter_wait_time() {
    let a: i32 = 0;
    let rep = While(Box::new(Wait(50.0)), vec![Wait(0.5), Action(Inc), Wait(0.5)]);
    let mut state = BT::new(rep, ());

    // sample after 10 seconds
    let (a, _, _) = tick(a, 10.0, &mut state);
    assert_eq!(a, 10);
}

#[test]
fn test_select_succeed_on_first() {
    let a: i32 = 0;
    let sel = Select(vec![Action(Inc), Action(Inc), Action(Inc)]);
    let mut state = BT::new(sel, ());

    let (a, _, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 1);
    state.reset_bt();
    let (a, _, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 2);
}

#[test]
fn test_select_needs_reset() {
    let a: i32 = 3;
    let sel = Select(vec![Action(LessThan(1)), Action(Dec), Action(Inc)]);
    let mut state = BT::new(sel, ());

    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 2);
    assert_eq!(s, Success);
    state.reset_bt();
    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 1);
    assert_eq!(s, Success);
    state.reset_bt();
    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 0);
    assert_eq!(s, Success);
    state.reset_bt();
    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 0);
    assert_eq!(s, Success);
}

#[test]
fn test_select_and_when_all() {
    let a: i32 = 3;
    let sel = Select(vec![Action(LessThan(1)), Action(Dec), Action(Inc)]);
    let whenall = WhenAll(vec![Wait(0.35), sel]);
    let mut state = BT::new(whenall, ());

    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 2);
    assert_eq!(s, Running);
    let (a, s, _) = tick(a, 0.3, &mut state);
    assert_eq!(a, 2);
    assert_eq!(s, Success);
}

#[test]
fn test_select_and_invert() {
    let a: i32 = 3;
    let sel = Invert(Box::new(Select(vec![Action(LessThan(1)), Action(Dec), Action(Inc)])));
    let mut state = BT::new(sel, ());

    // Running + Failure = Failure
    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 2);
    assert_eq!(s, Failure);
    state.reset_bt();
    let (a, s, _) = tick(a, 0.3, &mut state);
    assert_eq!(a, 1);
    assert_eq!(s, Failure);
    state.reset_bt();
    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 0);
    assert_eq!(s, Failure);
    state.reset_bt();
    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 0);
    assert_eq!(s, Failure);
}

#[test]
fn test_always_succeed() {
    let a: i32 = 3;
    let sel = Sequence(vec![
        Wait(0.5),
        Action(LessThan(2)),
        Wait(0.5),
        Action(LessThan(1)),
        Wait(0.5),
    ]);
    let behavior = AlwaysSucceed(Box::new(sel));
    let mut state = BT::new(behavior, ());

    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 3);
    assert_eq!(s, Running);
    let (a, s, _) = tick(a, 0.7, &mut state);
    assert_eq!(a, 3);
    assert_eq!(s, Success);
    state.reset_bt();
    let (a, s, _) = tick(a, 0.5, &mut state);
    assert_eq!(a, 3);
    assert_eq!(s, Success);
    state.reset_bt();
    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 3);
    assert_eq!(s, Running);
}

#[test]
fn test_after_all_succeed_in_order() {
    let a: i32 = 0;
    let after = After(vec![Action(Inc), Wait(0.1), Wait(0.2)]);
    let mut state = BT::new(after, ());

    let (a, s, dt) = tick(a, 0.1, &mut state);

    assert_eq!(a, 1);
    assert_eq!(s, Running);
    assert_eq!(dt, 0.0);

    let (a, s, dt) = tick(a, 0.1, &mut state);

    assert_eq!(a, 1);
    assert_eq!(s, Success);
    assert_eq!(dt, 0.0);
}

#[test]
fn test_after_all_succeed_out_of_order() {
    let a: i32 = 0;
    let after = After(vec![Action(Inc), Wait(0.2), Wait(0.1)]);
    let mut state = BT::new(after, ());

    let (a, s, dt) = tick(a, 0.05, &mut state);

    assert_eq!(a, 1);
    assert_eq!(s, Running);
    assert_eq!(dt, 0.0);

    let (a, s, dt) = tick(a, 0.1, &mut state);

    assert_eq!(a, 1);
    assert_eq!(s, Failure);
    assert_eq!(dt, 0.0);
}

#[test]
fn test_repeat_sequence() {
    {
        let a: i32 = 0;
        let after = WhileAll(Box::new(Action(LessThanRunningSuccess(5))), vec![Action(Inc)]);

        let mut state = BT::new(after, ());

        let (a, s, dt) = tick(a, 0.0, &mut state);

        assert_eq!(a, 5);
        assert_eq!(s, Success);
        assert_eq!(dt, 0.0);
    }
}

#[test]
/// Ensure that if the condition behavior and the first sequence behavior both return
/// running initially, then the condition behavior cannot run more than once until the whole
/// sequence has succeeded.
fn test_repeat_sequence_double_running() {
    let after = WhileAll(
        Box::new(Action(LessThanRunningSuccess(5))), // running...
        vec![
            Action(LessThanRunningSuccess(5)), // running... until current value is 5
            Action(Dec),                       // success... 4
            Action(Dec),                       // success... 3
            Action(LessThan(0)),               // failure
        ],
    );
    let mut state = BT::new(after, ());

    let mut current_value = 0;
    loop {
        let (a, s, _) = tick(current_value, 0.0, &mut state);
        current_value = a;
        match s {
            Running => {
                current_value += 1; // increase curent value everytime sequence behavior returns running
            }
            _ => {
                break;
            }
        }
    }

    assert_eq!(current_value, 3);
}

#[test]
fn test_repeat_sequence2() {
    let after = WhileAll(
        Box::new(Action(LessThanRunningSuccess(5))), // running...
        vec![
            Action(LessThanRunningSuccess(10)), // running... until current value is 5
            Action(Dec),                        // success... 4
            Action(Dec),                        // success... 3
            Action(Dec),                        // success... 2
        ],
    );
    let mut state = BT::new(after, ());

    let mut current_value = 0;
    let mut current_status;
    loop {
        let (a, s, _) = tick(current_value, 0.0, &mut state);
        current_value = a;
        current_status = s;
        match s {
            Running => {
                current_value += 1; // increase current value everytime sequence behavior returns running
            }
            _ => {
                break;
            }
        }
    }
    assert_eq!(current_status, bonsai_bt::Status::Success);
    assert_eq!(current_value, 7);
}

#[test]
fn test_repeat_sequence3() {
    let after = WhileAll(
        Box::new(Action(LessThanRunningSuccess(2))),
        vec![
            Action(LessThanRunningSuccess(10)),
            Action(Dec),
            Action(Dec),
            Action(Dec),
            Action(LessThanRunningSuccess(10)),
            Action(Dec),
        ],
    );
    let mut state = BT::new(after, ());

    let mut current_value = 0;
    let mut current_status;
    loop {
        let (a, s, _) = tick(current_value, 0.0, &mut state);
        current_value = a;
        current_status = s;
        match s {
            Running => {
                current_value += 1;
            }
            _ => {
                break;
            }
        }
    }
    assert_eq!(current_status, Success);
    assert_eq!(current_value, 9);
}

#[test]
fn test_repeat_sequence_nested() {
    let dec2 = Sequence(vec![Action(Dec), Action(Dec)]);
    let inc1 = Sequence(vec![Action(Inc)]);

    let nested = WhileAll(Box::new(Action(LessThanRunningSuccess(5))), vec![Action(Inc), inc1]);

    let after = WhileAll(
        Box::new(Action(LessThanRunningSuccess(1))),
        vec![
            nested,      // inc to 6
            Action(Dec), // -1
            dec2,        // -2
        ], // == 3
    );
    let mut state = BT::new(after, ());

    let mut current_value = 0;
    let mut current_status;
    loop {
        let (a, s, _) = tick(current_value, 0.0, &mut state);
        current_value = a;
        current_status = s;
        match s {
            Running => {}
            _ => {
                break;
            }
        }
    }
    assert_eq!(current_status, bonsai_bt::Status::Success);
    assert_eq!(current_value, 3);
}

#[test]
fn test_repeat_sequence_fail() {
    {
        let a: i32 = 4;
        let after = WhileAll(
            Box::new(Action(LessThanRunningSuccess(5))),
            vec![Action(Dec), Action(LessThan(0))],
        );
        let mut state = BT::new(after, ());
        let (a, s, dt) = tick(a, 0.0, &mut state);
        assert_eq!(a, 3);
        assert_eq!(s, Failure);
        assert_eq!(dt, 0.0);
    }
}

#[test]
fn test_repeat_sequence_timed() {
    let a: i32 = 0;
    let time_step = 0.1;
    let steps = 5;
    let after = WhileAll(
        Box::new(Action(LessThanRunningSuccess(steps))),
        vec![Wait(time_step), Action(Inc)],
    );
    let mut state = BT::new(after, ());

    // increment 3 times
    let (a, s, dt) = tick(a, time_step * 3.0, &mut state);
    assert_eq!(dt, 0.0);
    assert_eq!(a, 3);
    assert_eq!(s, Running);

    let (a, s, dt) = tick(a, 100.0, &mut state);
    assert_eq!(dt, 100.0);
    assert_eq!(a, 5);
    assert_eq!(s, Success);
}

#[test]
#[should_panic]
fn test_repeat_sequence_empty() {
    let after = WhileAll(Box::new(Action(LessThanRunningSuccess(0))), vec![]);
    // panics because no behaviors...
    let _state = BT::new(after, ());
}

#[test]
fn race_returns_first_success() {
    let a: i32 = 0;
    // Inc succeeds immediately, Wait is still running
    let behavior = Race(vec![Action(Inc), Wait(10.0)]);
    let mut state = BT::new(behavior, ());
    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 1);
    assert_eq!(s, Success);
}

#[test]
fn race_returns_first_failure() {
    let a: i32 = 5;
    // LessThan(1) fails immediately since 5 >= 1, Wait is still running
    let behavior = Race(vec![Action(LessThan(1)), Wait(10.0)]);
    let mut state = BT::new(behavior, ());
    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 5);
    assert_eq!(s, Failure);
}

#[test]
fn race_running_until_first_completes() {
    let a: i32 = 0;
    // Both children are time-based, neither completes on first tick
    let behavior = Race(vec![Wait(1.0), Wait(2.0)]);
    let mut state = BT::new(behavior, ());

    // After 0.5s, both still running
    let (_a, s, _) = tick(a, 0.5, &mut state);
    assert_eq!(s, Running);

    // After another 0.5s (total 1.0s), first Wait completes with Success
    let (_a, s, _) = tick(_a, 0.5, &mut state);
    assert_eq!(s, Success);
}

#[test]
fn race_second_child_wins_if_first_is_running() {
    let a: i32 = 0;
    let behavior = Race(vec![WaitForever, Action(Inc)]);
    let mut state = BT::new(behavior, ());
    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(a, 1);
    assert_eq!(s, Success);
}

#[test]
fn race_failure_short_circuits_unlike_when_any() {
    // the main difference from WhenAny:
    // WhenAny would swallow the failure and keep running.
    // Race returns the failure immediately.
    let a: i32 = 5;
    // LessThan(1) fails immediately (5 >= 1), Wait(10.0) is still running
    let behavior = Race(vec![Action(LessThan(1)), Wait(10.0)]);
    let mut state = BT::new(behavior, ());
    let (a, s, _) = tick(a, 0.1, &mut state);
    assert_eq!(s, Failure);

    // for WhenAny: same children, but failure is swallowed
    let behavior_any = WhenAny(vec![Action(LessThan(1)), Wait(10.0)]);
    let mut state_any = BT::new(behavior_any, ());
    let (_, s_any, _) = tick(a, 0.1, &mut state_any);
    assert_eq!(s_any, Running);
}

#[test]
fn race_timeout_pattern() {
    let a: i32 = 0;
    // Simulate a "slow action" using WaitForever with a 1-second timeout.
    // The timeout (Wait) fires first.
    let behavior = Race(vec![WaitForever, Wait(1.0)]);
    let mut state = BT::new(behavior, ());

    let (_, s, _) = tick(a, 0.5, &mut state);
    assert_eq!(s, Running);

    let (_, s, _) = tick(a, 0.5, &mut state);
    assert_eq!(s, Success);
}

#[test]
fn race_empty() {
    let a: i32 = 0;
    let behavior = Race(vec![]);
    let mut state = BT::new(behavior, ());
    let (_, s, _) = tick(a, 0.1, &mut state);
    // No children means nothing can complete, stays Running
    assert_eq!(s, Running);
}
