//! Acceptance tests for `RecordingTracer` round-trip
//! through `BT::tick_recording`. Verifies that `TickTrace.states` contains the
//! correct (id, Status) entries for every variant, including sparse semantics.

use bonsai_bt::{
    Action, ActionArgs, After, AlwaysSucceed, Event, Failure, Float, If, Invert, Race, Running, Select, Sequence,
    Status, Success, UpdateArgs, Wait, WaitForever, WhenAll, WhenAny, While, WhileAll, BT,
};
use std::collections::HashMap;

#[derive(Clone, Debug)]
enum Act {
    A,
    B,
    C,
    D,
    Cond,
    OnS,
    OnF,
}

fn dt_event(dt: Float) -> Event {
    UpdateArgs { dt }.into()
}

/// Test 2 — `Sequence([A, B])`, A=Success, B=Running on tick 1.
#[test]
fn sequence_mixed_success_running() {
    use Act::*;
    let tree = Sequence(vec![Action(A), Action(B)]);
    let mut bt = BT::new(tree, ());
    let e = dt_event(1.0);

    let (_result, trace) = bt
        .tick_recording(&e, &mut |args: ActionArgs<Event, Act>, _| match *args.action {
            A => (Success, args.dt),
            B => (Running, 0.0),
            _ => unreachable!(),
        })
        .unwrap();

    assert_eq!(trace.states.get(&0), Some(&Running), "Sequence root");
    assert_eq!(trace.states.get(&1), Some(&Success), "Action(A)");
    assert_eq!(trace.states.get(&2), Some(&Running), "Action(B)");
    assert_eq!(trace.states.len(), 3);
}

/// Test 3 — `Sequence([A, B, C])` advances past A,B on tick 1; tick 2 only
/// re-ticks C, giving a sparse trace.
#[test]
fn sequence_sparse_on_subsequent_tick() {
    use Act::*;
    let tree = Sequence(vec![Action(A), Action(B), Action(C)]);
    let mut bt = BT::new(tree, 0u32);
    let e = dt_event(1.0);

    // Tick 1: A=Success, B=Success, C=Running.
    let (_r1, trace1) = bt
        .tick_recording(&e, &mut |args: ActionArgs<Event, Act>, count: &mut u32| {
            *count += 1;
            match *args.action {
                A | B => (Success, args.dt),
                C => (Running, 0.0),
                _ => unreachable!(),
            }
        })
        .unwrap();

    assert_eq!(trace1.states.get(&0), Some(&Running));
    assert_eq!(trace1.states.get(&1), Some(&Success));
    assert_eq!(trace1.states.get(&2), Some(&Success));
    assert_eq!(trace1.states.get(&3), Some(&Running));
    assert_eq!(trace1.states.len(), 4);

    // Tick 2: only C should be re-ticked.
    let (_r2, trace2) = bt
        .tick_recording(&e, &mut |args: ActionArgs<Event, Act>, _| match *args.action {
            C => (Running, 0.0),
            _ => panic!("only C should be re-ticked on tick 2, got {:?}", args.action),
        })
        .unwrap();

    assert_eq!(trace2.states.get(&0), Some(&Running), "Sequence root");
    assert_eq!(trace2.states.get(&3), Some(&Running), "Action(C)");
    assert!(!trace2.states.contains_key(&1), "Action(A) is sparse");
    assert!(!trace2.states.contains_key(&2), "Action(B) is sparse");
    assert_eq!(trace2.states.len(), 2);
    assert_eq!(trace2.tick_id, 2, "second tick_recording call gets tick_id 2");
}

/// Test 4 — `If(Cond->Success, OnS, OnF)`. Cond and OnS appear; OnF is sparse.
#[test]
fn if_success_branch_sparse_on_failure() {
    use Act::*;
    let tree = If(Box::new(Action(Cond)), Box::new(Action(OnS)), Box::new(Action(OnF)));
    let mut bt = BT::new(tree, ());
    let e = dt_event(1.0);

    let (_result, trace) = bt
        .tick_recording(&e, &mut |args: ActionArgs<Event, Act>, _| match *args.action {
            Cond | OnS => (Success, args.dt),
            OnF => panic!("OnF must not be ticked when Cond succeeds"),
            _ => unreachable!(),
        })
        .unwrap();

    assert_eq!(trace.states.get(&0), Some(&Success), "If root");
    assert_eq!(trace.states.get(&1), Some(&Success), "Cond");
    assert_eq!(trace.states.get(&2), Some(&Success), "OnS");
    assert!(!trace.states.contains_key(&3), "OnF must be sparse");
    assert_eq!(trace.states.len(), 3);
}

/// Test 5 — `If(Cond->Failure, OnS, OnF)`. Cond and OnF appear; OnS is sparse.
#[test]
fn if_failure_branch_sparse_on_success() {
    use Act::*;
    let tree = If(Box::new(Action(Cond)), Box::new(Action(OnS)), Box::new(Action(OnF)));
    let mut bt = BT::new(tree, ());
    let e = dt_event(1.0);

    let (_result, trace) = bt
        .tick_recording(&e, &mut |args: ActionArgs<Event, Act>, _| match *args.action {
            Cond => (Failure, args.dt),
            OnF => (Failure, args.dt),
            OnS => panic!("OnS must not be ticked when Cond fails"),
            _ => unreachable!(),
        })
        .unwrap();

    assert_eq!(trace.states.get(&0), Some(&Failure), "If root");
    assert_eq!(trace.states.get(&1), Some(&Failure), "Cond");
    assert_eq!(trace.states.get(&3), Some(&Failure), "OnF");
    assert!(!trace.states.contains_key(&2), "OnS must be sparse");
    assert_eq!(trace.states.len(), 3);
}

/// Test 6 — `While(WaitForever, [A, B])`. Body cycles within a single tick.
/// We use a blackboard counter so the handler stops returning Success after
/// 4 calls (2 full cycles), forcing the loop to break with `Running`. Both
/// body action ids must appear in the trace (last-status-wins).
#[test]
fn while_body_cycles_within_tick() {
    use Act::*;
    let tree = While(Box::new(WaitForever), vec![Action(A), Action(B)]);
    let mut bt = BT::new(tree, 0u32);
    let e = dt_event(1.0);

    let (_result, trace) = bt
        .tick_recording(&e, &mut |args: ActionArgs<Event, Act>, count: &mut u32| {
            *count += 1;
            if *count <= 4 {
                // Positive remaining dt keeps the body loop alive.
                (Success, args.dt)
            } else {
                (Running, 0.0)
            }
        })
        .unwrap();

    // ids: 0=While, 1=WaitForever, 2=Action(A), 3=Action(B)
    assert!(trace.states.contains_key(&0), "While root must be recorded");
    assert!(trace.states.contains_key(&1), "WaitForever cond must be recorded");
    assert_eq!(trace.states.get(&1), Some(&Running), "WaitForever always Running");
    assert!(trace.states.contains_key(&2), "Action(A) must be recorded");
    assert!(trace.states.contains_key(&3), "Action(B) must be recorded");
}

/// Test 7 — `WhenAll([A, B])`. A finishes in tick 1; B keeps running.
/// On tick 2, A's cursor is `None` so it must NOT appear in the trace.
#[test]
fn when_all_partial_completion_sparse_on_next_tick() {
    use Act::*;
    let tree = WhenAll(vec![Action(A), Action(B)]);
    let mut bt = BT::new(tree, ());
    let e = dt_event(1.0);

    // Tick 1: A=Success, B=Running.
    let (_r1, trace1) = bt
        .tick_recording(&e, &mut |args: ActionArgs<Event, Act>, _| match *args.action {
            A => (Success, args.dt),
            B => (Running, 0.0),
            _ => unreachable!(),
        })
        .unwrap();

    assert_eq!(trace1.states.get(&0), Some(&Running), "WhenAll root");
    assert_eq!(trace1.states.get(&1), Some(&Success), "A succeeded");
    assert_eq!(trace1.states.get(&2), Some(&Running), "B running");

    // Tick 2: only B re-ticks.
    let (_r2, trace2) = bt
        .tick_recording(&e, &mut |args: ActionArgs<Event, Act>, _| match args.action {
            B => (Running, 0.0),
            other => panic!("only B should be re-ticked, got {:?}", other),
        })
        .unwrap();

    assert_eq!(trace2.states.get(&0), Some(&Running), "WhenAll root");
    assert_eq!(trace2.states.get(&2), Some(&Running), "B");
    assert!(!trace2.states.contains_key(&1), "A is None — must be sparse");
    assert_eq!(trace2.states.len(), 2);
}

/// Test 8a — All-variants smoke test. Tree completes in one tick; every
/// visited id appears in the trace. Excludes raw `While` (unbounded) and
/// `WaitForever`; covered separately in tests 6 and 8b.
#[test]
fn all_variants_id_coverage_smoke() {
    // Tree (DFS preorder ids in comments):
    //  0 Sequence
    //  1   Select(A_fail, B_succ)               // Select succeeds via B
    //  2     Action(A)  -> Failure              // 1st A call
    //  3     Action(B)  -> Success
    //  4   If(Cond->Success, AlwaysSucceed(Action(C)->Failure), OnF)
    //  5     Action(Cond) -> Success
    //  6     AlwaysSucceed                       // records Success
    //  7       Action(C) -> Failure              // child records Failure
    //  8     Action(OnF) -> ...                  // sparse
    //  9   Invert(Action(D)->Failure)            // child Failure -> Invert Success
    // 10     Action(D) -> Failure
    // 11   WhenAll([A, B])                       // both succeed
    // 12     Action(A) -> Success                // 2nd A
    // 13     Action(B) -> Success
    // 14   WhenAny([A, B])                       // first succeeds, short-circuit
    // 15     Action(A) -> Success                // 3rd A
    // 16     Action(B)                           // sparse
    // 17   Race([A, B])                          // first completes, short-circuit
    // 18     Action(A) -> Success                // 4th A
    // 19     Action(B)                           // sparse
    // 20   After([Wait(0.0), Wait(0.5)])         // both Wait nodes succeed; B has
    //                                            // strictly less dt remaining
    // 21     Wait(0.0)
    // 22     Wait(0.5)
    // 23   WhileAll(Cond->Success, [Action(B)])  // condition succeeds -> WhileAll Success
    // 24     Action(Cond) -> Success             // 2nd Cond
    // 25     Action(B)                           // sparse — body never reached
    // 26   Wait(0.0)                             // succeeds

    use Act::*;
    let tree = Sequence(vec![
        Select(vec![Action(A), Action(B)]),
        If(
            Box::new(Action(Cond)),
            Box::new(AlwaysSucceed(Box::new(Action(C)))),
            Box::new(Action(OnF)),
        ),
        Invert(Box::new(Action(D))),
        WhenAll(vec![Action(A), Action(B)]),
        WhenAny(vec![Action(A), Action(B)]),
        Race(vec![Action(A), Action(B)]),
        After(vec![Wait(0.0), Wait(0.5)]),
        WhileAll(Box::new(Action(Cond)), vec![Action(B)]),
        Wait(0.0),
    ]);

    let mut bt = BT::new(tree, HashMap::<&'static str, u32>::new());
    let e = dt_event(1.0);

    let (_result, trace) = bt
        .tick_recording(
            &e,
            &mut |args: ActionArgs<Event, Act>, bb: &mut HashMap<&'static str, u32>| {
                let key: &'static str = match *args.action {
                    A => "A",
                    B => "B",
                    C => "C",
                    D => "D",
                    Cond => "Cond",
                    OnS => "OnS",
                    OnF => "OnF",
                };
                let n = bb.entry(key).and_modify(|v| *v += 1).or_insert(1);
                match (args.action, *n) {
                    (A, 1) => (Failure, args.dt), // Select's first attempt fails
                    (A, _) => (Success, args.dt), // 2nd-4th A calls succeed
                    (B, _) => (Success, args.dt),
                    (C, _) => (Failure, args.dt), // child of AlwaysSucceed
                    (D, _) => (Failure, args.dt), // child of Invert -> Invert Success
                    (Cond, _) => (Success, args.dt),
                    (OnS, _) | (OnF, _) => (Success, args.dt),
                }
            },
        )
        .unwrap();

    // Composite root + visited child ids — must all appear with their
    // expected status.
    let expected: &[(usize, Status)] = &[
        (0, Success),  // outer Sequence
        (1, Success),  // Select
        (2, Failure),  //   Action(A) -> Failure
        (3, Success),  //   Action(B) -> Success
        (4, Success),  // If
        (5, Success),  //   Cond
        (6, Success),  //   AlwaysSucceed
        (7, Failure),  //     Action(C) -> Failure (child status preserved)
        (9, Success),  // Invert (inverted from child Failure)
        (10, Failure), //   Action(D) -> Failure
        (11, Success), // WhenAll
        (12, Success), //   A
        (13, Success), //   B
        (14, Success), // WhenAny
        (15, Success), //   A
        (17, Success), // Race (first cursor completes)
        (18, Success), //   A
        (20, Success), // After
        (21, Success), //   Wait(0.0)
        (22, Success), //   Wait(0.5)
        (23, Success), // WhileAll
        (24, Success), //   Cond
        (26, Success), // Wait(0.0)
    ];
    for (id, status) in expected {
        assert_eq!(
            trace.states.get(id),
            Some(status),
            "id {id} must be present with status {status:?}"
        );
    }

    // Sparse: never-ticked branches must be absent.
    assert!(!trace.states.contains_key(&8), "If's OnF must be sparse");
    assert!(
        !trace.states.contains_key(&16),
        "WhenAny's second child must be sparse (short-circuit)"
    );
    assert!(
        !trace.states.contains_key(&19),
        "Race's second child must be sparse (short-circuit)"
    );
    assert!(
        !trace.states.contains_key(&25),
        "WhileAll's body must be sparse (condition succeeded immediately)"
    );
}

/// Test 8b — `WaitForever` covered via `Race` that resolves instantly.
/// `WaitForever`'s id appears with status `Running`.
#[test]
fn wait_forever_appears_running_when_race_resolves() {
    use Act::*;
    // ids: 0=Race, 1=WaitForever, 2=Action(A)
    let tree = Race(vec![WaitForever, Action(A)]);
    let mut bt = BT::new(tree, ());
    let e = dt_event(1.0);

    let (_result, trace) = bt
        .tick_recording(&e, &mut |args: ActionArgs<Event, Act>, _| match *args.action {
            A => (Success, args.dt),
            _ => unreachable!(),
        })
        .unwrap();

    // WaitForever ticks first (returns Running, no short-circuit), then A
    // succeeds and Race short-circuits.
    assert_eq!(
        trace.states.get(&1),
        Some(&Running),
        "WaitForever must be recorded as Running"
    );
    assert_eq!(trace.states.get(&2), Some(&Success), "Action(A) succeeded");
    assert_eq!(trace.states.get(&0), Some(&Success), "Race short-circuits");
}

/// Test 9 — `Race` short-circuit recording. The first child's id and the
/// Race root's id must appear; the second child's id may or may not appear
/// depending on iteration order.
#[test]
fn race_short_circuit_records_winner_and_root() {
    use Act::*;
    // ids: 0=Race, 1=Action(A), 2=Action(B)
    let tree = Race(vec![Action(A), Action(B)]);
    let mut bt = BT::new(tree, ());
    let e = dt_event(1.0);

    let (_result, trace) = bt
        .tick_recording(&e, &mut |args: ActionArgs<Event, Act>, _| match *args.action {
            A => (Success, args.dt),
            B => (Running, 0.0),
            _ => unreachable!(),
        })
        .unwrap();

    // Must-haves: the winning child and the Race root.
    assert_eq!(trace.states.get(&1), Some(&Success), "A wins");
    assert_eq!(trace.states.get(&0), Some(&Success), "Race root");
    // B may or may not have ticked depending on iteration order; we don't
    // assert on it. Current impl iterates in order so A short-circuits before
    // B is touched.
}

/// Bonus — `tick_id` increments monotonically across `tick_recording` calls
/// and is preserved across `reset_bt`.
#[test]
fn tick_id_monotonic_and_survives_reset() {
    use Act::*;
    let tree = Sequence(vec![Action(A)]);
    let mut bt = BT::new(tree, ());
    let e = dt_event(1.0);

    let (_, t1) = bt
        .tick_recording(&e, &mut |_: ActionArgs<Event, Act>, _| (Success, 0.0))
        .unwrap();
    assert_eq!(t1.tick_id, 1);
    assert_eq!(bt.tick_count(), 1);

    bt.reset_bt();
    assert_eq!(bt.tick_count(), 1, "reset_bt must NOT reset tick_count");

    let (_, t2) = bt
        .tick_recording(&e, &mut |_: ActionArgs<Event, Act>, _| (Success, 0.0))
        .unwrap();
    assert_eq!(t2.tick_id, 2, "tick_id continues past reset");
}

/// Every tick re-walks all children, so the trace is never sparse — every
/// visited child shows up each time, not just the one that was running.
#[test]
fn memoryless_sequence_records_root_and_visited_children() {
    use Act::*;
    let tree = Sequence(vec![Action(A), Action(B)]).memory(false);
    let mut bt = BT::new(tree, ());
    let e = dt_event(1.0);

    // Tick 1: A=Success, B=Running → composite Running.
    let (_r1, t1) = bt
        .tick_recording(&e, &mut |args: ActionArgs<Event, Act>, _| match *args.action {
            A => (Success, args.dt),
            B => (Running, 0.0),
            _ => unreachable!(),
        })
        .unwrap();

    assert_eq!(t1.states.get(&0), Some(&Running), "MemorylessSequence root");
    assert_eq!(t1.states.get(&1), Some(&Success), "Action(A)");
    assert_eq!(t1.states.get(&2), Some(&Running), "Action(B)");
    assert_eq!(t1.states.len(), 3);

    // Tick 2: A is re-ticked from scratch (a regular Sequence would skip it
    // and only resume B). Make B succeed this time.
    let (_r2, t2) = bt
        .tick_recording(&e, &mut |args: ActionArgs<Event, Act>, _| match *args.action {
            A | B => (Success, args.dt),
            _ => unreachable!(),
        })
        .unwrap();

    assert_eq!(
        t2.states.get(&0),
        Some(&Success),
        "root succeeds when all children succeed"
    );
    assert_eq!(t2.states.get(&1), Some(&Success), "A re-ticked");
    assert_eq!(t2.states.get(&2), Some(&Success), "B re-ticked");
    assert_eq!(t2.states.len(), 3, "trace stays dense — no sparse semantics");
}

/// Short-circuits on first Success, so later siblings never enter the trace.
#[test]
fn memoryless_select_short_circuit_omits_later_siblings() {
    use Act::*;
    let tree = Select(vec![Action(A), Action(B)]).memory(false);
    let mut bt = BT::new(tree, ());
    let e = dt_event(1.0);

    // A succeeds → composite returns Success and skips B.
    let (_r, trace) = bt
        .tick_recording(&e, &mut |args: ActionArgs<Event, Act>, _| match *args.action {
            A => (Success, args.dt),
            _ => panic!("B should not be ticked after A succeeds"),
        })
        .unwrap();

    assert_eq!(trace.states.get(&0), Some(&Success), "MemorylessSelector root");
    assert_eq!(trace.states.get(&1), Some(&Success), "Action(A)");
    assert!(!trace.states.contains_key(&2), "Action(B) not visited");
    assert_eq!(trace.states.len(), 2);
}
