use crate::event::UpdateEvent;
use crate::sequence::sequence;
use crate::state::State::*;
use crate::status::Status::*;
use crate::when_all::when_all;
use crate::{Behavior, Status};
use std::fmt::Debug;
// use serde_derive::{Deserialize, Serialize};

/// The action is still running. So, there is
pub const RUNNING: (Status, f64) = (Running, 0.0);

/// The arguments in the action callback.
pub struct ActionArgs<'a, E: 'a, A: 'a> {
    /// The event.
    pub event: &'a E,
    /// The remaining delta time. When one action terminates,
    /// it can consume some of dt and the remaining is passed
    /// onto the next action.
    pub dt: f64,
    /// The action running.
    pub action: &'a A,
}

/// Keeps track of a behavior.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
pub enum State<A> {
    /// Executes an action.
    ActionState(A),
    /// Converts `Success` into `Failure` and vice versa.
    InvertState(Box<State<A>>),
    /// Ignores failures and always return `Success`.
    AlwaysSucceedState(Box<State<A>>),
    /// Keeps track of waiting for a period of time before continuing.
    ///
    /// f64: Total time in seconds to wait
    ///
    /// f64: Time elapsed in seconds
    WaitState(f64, f64),
    /// Waits forever.
    WaitForeverState,
    /// Keeps track of an `If` behavior.
    /// If status is `Running`, then it evaluates the condition.
    /// If status is `Success`, then it evaluates the success behavior.
    /// If status is `Failure`, then it evaluates the failure behavior.
    IfState(Box<Behavior<A>>, Box<Behavior<A>>, Status, Box<State<A>>),
    /// Keeps track of a `Select` behavior.
    SelectState(Vec<Behavior<A>>, usize, Box<State<A>>),
    /// Keeps track of an `Sequence` behavior.
    SequenceState(Vec<Behavior<A>>, usize, Box<State<A>>),
    /// Keeps track of a `While` behavior.
    WhileState(Box<State<A>>, Vec<Behavior<A>>, usize, Box<State<A>>),
    /// Keeps track of a `WhenAll` behavior.
    WhenAllState(Vec<Option<State<A>>>),
    /// Keeps track of a `WhenAny` behavior.
    WhenAnyState(Vec<Option<State<A>>>),
    /// Keeps track of an `After` behavior.
    AfterState(usize, Vec<State<A>>),
}

impl<A: Clone> State<A> {
    /// Creates a state from a behavior.
    pub fn new(behavior: Behavior<A>) -> Self {
        match behavior {
            Behavior::Action(action) => State::ActionState(action),
            Behavior::Invert(ev) => State::InvertState(Box::new(State::new(*ev))),
            Behavior::AlwaysSucceed(ev) => State::AlwaysSucceedState(Box::new(State::new(*ev))),
            Behavior::Wait(dt) => State::WaitState(dt, 0.0),
            Behavior::WaitForever => State::WaitForeverState,
            Behavior::If(condition, success, failure) => {
                let state = State::new(*condition);
                State::IfState(success, failure, Status::Running, Box::new(state))
            }
            Behavior::Select(sel) => {
                let state = State::new(sel[0].clone());
                State::SelectState(sel, 0, Box::new(state))
            }
            Behavior::Sequence(seq) => {
                let state = State::new(seq[0].clone());
                State::SequenceState(seq, 0, Box::new(state))
            }
            Behavior::While(ev, rep) => {
                let state = State::new(rep[0].clone());
                State::WhileState(Box::new(State::new(*ev)), rep, 0, Box::new(state))
            }
            Behavior::WhenAll(all) => State::WhenAllState(all.into_iter().map(|ev| Some(State::new(ev))).collect()),
            Behavior::WhenAny(all) => State::WhenAnyState(all.into_iter().map(|ev| Some(State::new(ev))).collect()),
            Behavior::After(seq) => State::AfterState(0, seq.into_iter().map(State::new).collect()),
        }
    }

    /// Updates the cursor that tracks an event.
    ///
    /// The action need to return status and remaining delta time.
    /// Returns status and the remaining delta time.
    ///
    /// Passes event, delta time in seconds, action and state to closure.
    /// The closure should return a status and remaining delta time.
    ///
    /// return: (Status, f64)
    /// function returns the result of the tree traversal, and how long
    /// it actually took to complete the traversal and propagate the
    /// results back up to the root node
    pub fn tick<E, F>(&mut self, e: &E, f: &mut F) -> (Status, f64)
    where
        E: UpdateEvent,
        F: FnMut(ActionArgs<E, A>) -> (Status, f64),
        A: Debug,
    {
        let upd = e.update(|args| Some(args.dt)).unwrap_or(None);

        // double match statements
        match (upd, self) {
            (_, &mut ActionState(ref action)) => {
                // println!("In ActionState: {:?}", action);
                f(ActionArgs {
                    event: e,
                    dt: upd.unwrap_or(0.0),
                    action,
                })
            }
            (_, &mut InvertState(ref mut cur)) => {
                // println!("In InvertState: {:?}", cur);
                match cur.tick(e, f) {
                    (Running, dt) => (Running, dt),
                    (Failure, dt) => (Success, dt),
                    (Success, dt) => (Failure, dt),
                }
            }
            (_, &mut AlwaysSucceedState(ref mut cur)) => {
                // println!("In AlwaysSucceedState: {:?}", cur);
                match cur.tick(e, f) {
                    (Running, dt) => (Running, dt),
                    (_, dt) => (Success, dt),
                }
            }
            (Some(dt), &mut WaitState(wait_t, ref mut t)) => {
                // println!("In WaitState: {}", wait_t);
                if *t + dt >= wait_t {
                    let time_overdue = *t + dt - wait_t;
                    *t = wait_t;
                    (Success, time_overdue)
                } else {
                    *t += dt;
                    RUNNING
                }
            }
            (_, &mut IfState(ref success, ref failure, ref mut status, ref mut state)) => {
                // println!("In IfState: {:?}", success);
                let mut remaining_dt = upd.unwrap_or(0.0);
                let remaining_e;
                // Run in a loop to evaluate success or failure with
                // remaining delta time after condition.
                loop {
                    *status = match *status {
                        Running => match state.tick(e, f) {
                            (Running, dt) => {
                                return (Running, dt);
                            }
                            (Success, dt) => {
                                **state = State::new((**success).clone());
                                remaining_dt = dt;
                                Success
                            }
                            (Failure, dt) => {
                                **state = State::new((**failure).clone());
                                remaining_dt = dt;
                                Failure
                            }
                        },
                        _ => {
                            return state.tick(
                                match upd {
                                    Some(_) => {
                                        remaining_e = UpdateEvent::from_dt(remaining_dt, e).unwrap();
                                        &remaining_e
                                    }
                                    _ => e,
                                },
                                f,
                            );
                        }
                    }
                }
            }
            (_, &mut SelectState(ref seq, ref mut i, ref mut cursor)) => {
                // println!("In SelectState: {:?}", seq);
                let select = true;
                sequence(select, upd, seq, i, cursor, e, f)
            }
            (_, &mut SequenceState(ref seq, ref mut i, ref mut cursor)) => {
                // println!("In SequenceState: {:?}", seq);
                let select = false;
                sequence(select, upd, seq, i, cursor, e, f)
            }
            (_, &mut WhileState(ref mut ev_cursor, ref rep, ref mut i, ref mut cursor)) => {
                // println!("In WhileState: {:?}", ev_cursor);
                // If the event terminates, do not execute the loop.
                match ev_cursor.tick(e, f) {
                    (Running, _) => {}
                    x => return x,
                };
                let cur = cursor;
                let mut remaining_dt = upd.unwrap_or(0.0);
                let mut remaining_e;
                loop {
                    match cur.tick(
                        match upd {
                            Some(_) => {
                                remaining_e = UpdateEvent::from_dt(remaining_dt, e).unwrap();
                                &remaining_e
                            }
                            _ => e,
                        },
                        f,
                    ) {
                        (Failure, x) => return (Failure, x),
                        (Running, _) => break,
                        (Success, new_dt) => {
                            remaining_dt = match upd {
                                // Change update event with remaining delta time.
                                Some(_) => new_dt,
                                // Other events are 'consumed' and not passed to next.
                                _ => return RUNNING,
                            }
                        }
                    };
                    *i += 1;
                    // If end of repeated events,
                    // start over from the first one.
                    if *i >= rep.len() {
                        *i = 0;
                    }
                    // Create a new cursor for next event.
                    // Use the same pointer to avoid allocation.
                    **cur = State::new(rep[*i].clone());
                }
                RUNNING
            }
            (_, &mut WhenAllState(ref mut cursors)) => {
                // println!("In WhenAllState: {:?}", cursors);
                let any = false;
                when_all(any, upd, cursors, e, f)
            }
            (_, &mut WhenAnyState(ref mut cursors)) => {
                // println!("In WhenAnyState: {:?}", cursors);
                let any = true;
                when_all(any, upd, cursors, e, f)
            }
            (_, &mut AfterState(ref mut i, ref mut cursors)) => {
                // println!("In AfterState: {}", i);
                // Get the least delta time left over.
                let mut min_dt = f64::MAX;
                for (j, item) in cursors.iter_mut().enumerate().skip(*i) {
                    match item.tick(e, f) {
                        (Running, _) => {
                            min_dt = 0.0;
                        }
                        (Success, new_dt) => {
                            // Remaining delta time must be less to succeed.
                            if *i == j && new_dt < min_dt {
                                *i += 1;
                                min_dt = new_dt;
                            } else {
                                // Return least delta time because
                                // that is when failure is detected.
                                return (Failure, min_dt.min(new_dt));
                            }
                        }
                        (Failure, new_dt) => {
                            return (Failure, new_dt);
                        }
                    };
                }
                if *i == cursors.len() {
                    (Success, min_dt)
                } else {
                    RUNNING
                }
            }
            // WaitForeverState, WaitState
            _ => RUNNING,
        }
    }
}
