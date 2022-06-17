use crate::sequence::sequence;
use crate::when_all::when_all;
use crate::{Behavior, Status};
// use serde_derive::{Deserialize, Serialize};

/// The action is still running.
pub const RUNNING: (Status, f64) = (Status::Running, 0.0);

/// The arguments in the action callback.
pub struct ActionArgs<'a, A: 'a, S: 'a> {
    /// The remaining delta time.
    pub dt: f64,
    /// The action running.
    pub action: &'a A,
    /// The state of the running action, if any.
    pub state: &'a mut Option<S>,
}

/// Keeps track of a behavior.
#[derive(Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub enum State<A, S> {
    /// Executes an action.
    ActionState(A, Option<S>),
    /// Converts `Success` into `Failure` and vice versa.
    FailState(Box<State<A, S>>),
    /// Ignores failures and always return `Success`.
    AlwaysSucceedState(Box<State<A, S>>),
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
    IfState(Box<Behavior<A>>, Box<Behavior<A>>, Status, Box<State<A, S>>),
    /// Keeps track of a `Select` behavior.
    SelectState(Vec<Behavior<A>>, usize, Box<State<A, S>>),
    /// Keeps track of an `Sequence` behavior.
    SequenceState(Vec<Behavior<A>>, usize, Box<State<A, S>>),
    /// Keeps track of a `While` behavior.
    WhileState(Box<State<A, S>>, Vec<Behavior<A>>, usize, Box<State<A, S>>),
    /// Keeps track of a `WhenAll` behavior.
    WhenAllState(Vec<Option<State<A, S>>>),
    /// Keeps track of a `WhenAny` behavior.
    WhenAnyState(Vec<Option<State<A, S>>>),
    /// Keeps track of an `After` behavior.
    AfterState(usize, Vec<State<A, S>>),
}

impl<A: Clone, S> State<A, S> {
    /// Creates a state from a behavior.
    pub fn new(behavior: Behavior<A>) -> Self {
        match behavior {
            Behavior::Action(action) => State::ActionState(action, None),
            Behavior::Fail(ev) => State::FailState(Box::new(State::new(*ev))),
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
    pub fn event<F>(&mut self, upd: Option<f64>, f: &mut F) -> (Status, f64)
    where
        F: FnMut(ActionArgs<A, S>) -> (Status, f64),
    {
        match (upd, self) {
            (_, &mut State::ActionState(ref action, ref mut state)) => {
                // Execute action.
                f(ActionArgs {
                    dt: upd.unwrap_or(0.0),
                    action,
                    state,
                })
            }
            (_, &mut State::FailState(ref mut cur)) => match cur.event(None, f) {
                (Status::Running, dt) => (Status::Running, dt),
                (Status::Failure, dt) => (Status::Success, dt),
                (Status::Success, dt) => (Status::Failure, dt),
            },
            (_, &mut State::AlwaysSucceedState(ref mut cur)) => match cur.event(None, f) {
                (Status::Running, dt) => (Status::Running, dt),
                (_, dt) => (Status::Success, dt),
            },
            (Some(dt), &mut State::WaitState(wait_t, ref mut t)) => {
                if *t + dt >= wait_t {
                    let remaining_dt = *t + dt - wait_t;
                    *t = wait_t;
                    (Status::Success, remaining_dt)
                } else {
                    *t += dt;
                    RUNNING
                }
            }
            (_, &mut State::IfState(ref success, ref failure, ref mut status, ref mut state)) => {
                let mut remaining_dt = upd.unwrap_or(0.0);
                // Run in a loop to evaluate success or failure with
                // remaining delta time after condition.
                loop {
                    *status = match *status {
                        Status::Running => match state.event(upd, f) {
                            (Status::Running, dt) => {
                                return (Status::Running, dt);
                            }
                            (Status::Success, dt) => {
                                **state = State::new((**success).clone());
                                remaining_dt = dt;
                                Status::Success
                            }
                            (Status::Failure, dt) => {
                                **state = State::new((**failure).clone());
                                remaining_dt = dt;
                                Status::Failure
                            }
                        },
                        _ => {
                            return state.event(
                                match upd {
                                    Some(_) => Some(remaining_dt),
                                    _ => upd,
                                },
                                f,
                            );
                        }
                    }
                }
            }
            (_, &mut State::SelectState(ref seq, ref mut i, ref mut cursor)) => {
                let select = true;
                sequence(select, upd, seq, i, cursor, f)
            }
            (_, &mut State::SequenceState(ref seq, ref mut i, ref mut cursor)) => {
                let select = false;
                sequence(select, upd, seq, i, cursor, f)
            }
            (_, &mut State::WhileState(ref mut ev_cursor, ref rep, ref mut i, ref mut cursor)) => {
                // If the event terminates, do not execute the loop.
                match ev_cursor.event(None, f) {
                    (Status::Running, _) => {}
                    x => return x,
                };
                let cur = cursor;
                let mut remaining_dt = upd.unwrap_or(0.0);
                loop {
                    match cur.event(
                        match upd {
                            Some(_) => Some(remaining_dt),
                            _ => upd,
                        },
                        f,
                    ) {
                        (Status::Failure, x) => return (Status::Failure, x),
                        (Status::Running, _) => break,
                        (Status::Success, new_dt) => {
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
            (_, &mut State::WhenAllState(ref mut cursors)) => {
                let any = false;
                when_all(any, upd, cursors, f)
            }
            (_, &mut State::WhenAnyState(ref mut cursors)) => {
                let any = true;
                when_all(any, upd, cursors, f)
            }
            (_, &mut State::AfterState(ref mut i, ref mut cursors)) => {
                // Get the least delta time left over.
                let mut min_dt = f64::MAX;
                for j in *i..cursors.len() {
                    match cursors[j].event(upd, f) {
                        (Status::Running, _) => {
                            min_dt = 0.0;
                        }
                        (Status::Success, new_dt) => {
                            // Remaining delta time must be less to succeed.
                            if *i == j && new_dt < min_dt {
                                *i += 1;
                                min_dt = new_dt;
                            } else {
                                // Return least delta time because
                                // that is when failure is detected.
                                return (Status::Failure, min_dt.min(new_dt));
                            }
                        }
                        (Status::Failure, new_dt) => {
                            return (Status::Failure, new_dt);
                        }
                    };
                }
                if *i == cursors.len() {
                    (Status::Success, min_dt)
                } else {
                    RUNNING
                }
            }
            _ => RUNNING,
        }
    }
}
