use crate::event::UpdateEvent;
use crate::sequence::{sequence, SequenceArgs};
use crate::state::State::*;
use crate::status::Status::*;
use crate::when_all::when_all;
use crate::{Behavior, Status};
use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The action is still running, and thus the action consumes
/// all the remaining delta time for the tick
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
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) enum State<A> {
    /// Executes an action.
    Action(A),
    /// Converts `Success` into `Failure` and vice versa.
    Invert(Box<State<A>>),
    /// Ignores failures and always return `Success`.
    AlwaysSucceed(Box<State<A>>),
    /// Keeps track of waiting for a period of time before continuing.
    Wait { time_to_wait: f64, elapsed_time: f64 },
    /// Waits forever.
    WaitForever,
    /// Keeps track of an `If` behavior.
    If {
        /// The behavior to run if the status is a success.
        on_success: Box<Behavior<A>>,
        /// The behavior to run if the status is a failure.
        on_failure: Box<Behavior<A>>,
        /// The status of the condition. The `If` behavior will resolve to one
        /// of `on_success` or `on_failure` once the status is not `Running`.
        status: Status,
        /// The current state to execute.
        current_state: Box<State<A>>,
    },
    /// Keeps track of a `Select` behavior.
    Select {
        /// The behaviors that will be selected across in order.
        behaviors: Vec<Behavior<A>>,
        /// The index of the behavior currently being executed.
        current_index: usize,
        /// The state of the behavior currently being executed.
        current_state: Box<State<A>>,
    },
    /// Keeps track of an `Sequence` behavior.
    Sequence {
        /// The behaviors that will be executed in order.
        behaviors: Vec<Behavior<A>>,
        /// The index of the behavior currently being executed.
        current_index: usize,
        /// The state of the behavior currently being executed.
        current_state: Box<State<A>>,
    },
    /// Keeps track of a `While` behavior.
    While {
        /// The state of the condition of the loop. The loop continues to run
        /// while this state is running.
        condition_state: Box<State<A>>,
        /// The behaviors that compose the loop body in order.
        loop_body: Vec<Behavior<A>>,
        /// The index of the behavior in the loop body currently being executed.
        loop_body_index: usize,
        /// The state of the behavior in the loop body currently being executed.
        loop_body_state: Box<State<A>>,
    },
    /// Keeps track of a `WhileAll` behavior.
    WhileAll {
        /// The state of the condition of the loop. The loop continues to run
        /// while this state is running, though this is only checked once at the
        /// start of each loop.
        condition_state: Box<State<A>>,
        /// Whether to check the condition on the next tick.
        check_condition: bool,
        /// The behaviors that compose the loop body in order.
        loop_body: Vec<Behavior<A>>,
        /// The index of the behavior in the loop body currently being executed.
        loop_body_index: usize,
        /// The state of the behavior in the loop body currently being executed.
        loop_body_state: Box<State<A>>,
    },
    /// Keeps track of a `WhenAll` behavior. As the states finish, they are set
    /// to [`None`].
    WhenAll(Vec<Option<State<A>>>),
    /// Keeps track of a `WhenAny` behavior. As the states finish, they are set
    /// to [`None`].
    WhenAny(Vec<Option<State<A>>>),
    /// Keeps track of an `After` behavior.
    After {
        /// The index of the next state that must succeed.
        next_success_index: usize,
        /// The states for the behaviors currently executing. All the states
        /// before `next_success_index` must have finished with success.
        states: Vec<State<A>>,
    },
}

impl<A: Clone> State<A> {
    /// Creates a state from a behavior.
    ///
    /// For each behavior there is a `State` that keeps track of current running process.
    /// When you declare a behavior, this state is not included, resulting in a compact
    /// representation that can be copied or shared between objects having same behavior.
    /// Behavior means the declarative representation of the behavior, and State represents
    /// the executing instance of that behavior.
    pub fn new(behavior: Behavior<A>) -> Self {
        match behavior {
            Behavior::Action(action) => State::Action(action),
            Behavior::Invert(ev) => State::Invert(Box::new(State::new(*ev))),
            Behavior::AlwaysSucceed(ev) => State::AlwaysSucceed(Box::new(State::new(*ev))),
            Behavior::Wait(dt) => State::Wait {
                time_to_wait: dt,
                elapsed_time: 0.0,
            },
            Behavior::WaitForever => State::WaitForever,
            Behavior::If(condition, on_success, on_failure) => {
                let state = State::new(*condition);
                State::If {
                    on_success,
                    on_failure,
                    status: Status::Running,
                    current_state: Box::new(state),
                }
            }
            Behavior::Select(behaviors) => {
                let state = State::new(behaviors[0].clone());
                State::Select {
                    behaviors,
                    current_index: 0,
                    current_state: Box::new(state),
                }
            }
            Behavior::Sequence(behaviors) => {
                let state = State::new(behaviors[0].clone());
                State::Sequence {
                    behaviors,
                    current_index: 0,
                    current_state: Box::new(state),
                }
            }
            Behavior::While(condition, loop_body) => {
                let state = State::new(loop_body[0].clone());
                State::While {
                    condition_state: Box::new(State::new(*condition)),
                    loop_body,
                    loop_body_index: 0,
                    loop_body_state: Box::new(state),
                }
            }
            Behavior::WhenAll(all) => State::WhenAll(all.into_iter().map(|ev| Some(State::new(ev))).collect()),
            Behavior::WhenAny(any) => State::WhenAny(any.into_iter().map(|ev| Some(State::new(ev))).collect()),
            Behavior::After(after_all) => State::After {
                next_success_index: 0,
                states: after_all.into_iter().map(State::new).collect(),
            },
            Behavior::WhileAll(condition, loop_body) => {
                let state = State::new(
                    loop_body
                        .first()
                        .expect("WhileAll's sequence of behaviors to run cannot be empty!")
                        .clone(),
                );
                State::WhileAll {
                    condition_state: Box::new(State::new(*condition)),
                    check_condition: true,
                    loop_body,
                    loop_body_index: 0,
                    loop_body_state: Box::new(state),
                }
            }
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
    pub fn tick<E, F, B>(&mut self, e: &E, blackboard: &mut B, f: &mut F) -> (Status, f64)
    where
        E: UpdateEvent,
        F: FnMut(ActionArgs<E, A>, &mut B) -> (Status, f64),
    {
        let upd = e.update(|args| Some(args.dt)).unwrap_or(None);

        // double match statements
        match (upd, self) {
            (_, &mut Action(ref action)) => {
                // println!("In ActionState: {:?}", action);
                f(
                    ActionArgs {
                        event: e,
                        dt: upd.unwrap_or(0.0),
                        action,
                    },
                    blackboard,
                )
            }
            (_, &mut Invert(ref mut cur)) => {
                // println!("In InvertState: {:?}", cur);
                match cur.tick(e, blackboard, f) {
                    (Running, dt) => (Running, dt),
                    (Failure, dt) => (Success, dt),
                    (Success, dt) => (Failure, dt),
                }
            }
            (_, &mut AlwaysSucceed(ref mut cur)) => {
                // println!("In AlwaysSucceedState: {:?}", cur);
                match cur.tick(e, blackboard, f) {
                    (Running, dt) => (Running, dt),
                    (_, dt) => (Success, dt),
                }
            }
            (
                Some(dt),
                &mut Wait {
                    time_to_wait,
                    ref mut elapsed_time,
                },
            ) => {
                // println!("In WaitState: {}", time_to_wait);
                *elapsed_time += dt;
                if *elapsed_time >= time_to_wait {
                    let time_overdue = *elapsed_time - time_to_wait;
                    *elapsed_time = time_to_wait;
                    (Success, time_overdue)
                } else {
                    RUNNING
                }
            }
            (
                _,
                &mut If {
                    ref on_success,
                    ref on_failure,
                    ref mut status,
                    ref mut current_state,
                },
            ) => {
                // println!("In IfState: {:?}", success);
                let mut remaining_dt = upd.unwrap_or(0.0);
                let remaining_e;
                // Run in a loop to evaluate success or failure with
                // remaining delta time after condition.
                loop {
                    *status = match *status {
                        Running => match current_state.tick(e, blackboard, f) {
                            (Running, dt) => {
                                return (Running, dt);
                            }
                            (Success, dt) => {
                                **current_state = State::new((**on_success).clone());
                                remaining_dt = dt;
                                Success
                            }
                            (Failure, dt) => {
                                **current_state = State::new((**on_failure).clone());
                                remaining_dt = dt;
                                Failure
                            }
                        },
                        _ => {
                            return current_state.tick(
                                match upd {
                                    Some(_) => {
                                        remaining_e = UpdateEvent::from_dt(remaining_dt, e).unwrap();
                                        &remaining_e
                                    }
                                    _ => e,
                                },
                                blackboard,
                                f,
                            );
                        }
                    }
                }
            }
            (
                _,
                &mut Select {
                    behaviors: ref seq,
                    current_index: ref mut i,
                    current_state: ref mut cursor,
                },
            ) => {
                // println!("In SelectState: {:?}", seq);
                let select = true;
                sequence(SequenceArgs {
                    select,
                    upd,
                    seq,
                    i,
                    cursor,
                    e,
                    f,
                    blackboard,
                })
            }
            (
                _,
                &mut Sequence {
                    behaviors: ref seq,
                    current_index: ref mut i,
                    current_state: ref mut cursor,
                },
            ) => {
                // println!("In SequenceState: {:?}", seq);
                let select = false;
                sequence(SequenceArgs {
                    select,
                    upd,
                    seq,
                    i,
                    cursor,
                    e,
                    f,
                    blackboard,
                })
            }
            (
                _,
                &mut While {
                    ref mut condition_state,
                    ref loop_body,
                    ref mut loop_body_index,
                    ref mut loop_body_state,
                },
            ) => {
                // println!("In WhileState: {:?}", condition_state);
                // If the condition behavior terminates, do not execute the loop.
                match condition_state.tick(e, blackboard, f) {
                    (Running, _) => {}
                    x => return x,
                };
                let cur = loop_body_state;
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
                        blackboard,
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
                    *loop_body_index += 1;
                    // If end of repeated events,
                    // start over from the first one.
                    if *loop_body_index >= loop_body.len() {
                        *loop_body_index = 0;
                    }
                    // Create a new cursor for next event.
                    // Use the same pointer to avoid allocation.
                    **cur = State::new(loop_body[*loop_body_index].clone());
                }
                RUNNING
            }
            (_, &mut WhenAll(ref mut cursors)) => {
                // println!("In WhenAllState: {:?}", cursors);
                let any = false;
                when_all(any, upd, cursors, e, f, blackboard)
            }
            (_, &mut WhenAny(ref mut cursors)) => {
                // println!("In WhenAnyState: {:?}", cursors);
                let any = true;
                when_all(any, upd, cursors, e, f, blackboard)
            }
            (
                _,
                &mut After {
                    ref mut next_success_index,
                    ref mut states,
                },
            ) => {
                // println!("In AfterState: {}", next_success_index);
                // Get the least delta time left over.
                let mut min_dt = f64::MAX;
                for (j, item) in states.iter_mut().enumerate().skip(*next_success_index) {
                    match item.tick(e, blackboard, f) {
                        (Running, _) => {
                            min_dt = 0.0;
                        }
                        (Success, new_dt) => {
                            // Remaining delta time must be less to succeed.
                            if *next_success_index == j && new_dt < min_dt {
                                *next_success_index += 1;
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
                if *next_success_index == states.len() {
                    (Success, min_dt)
                } else {
                    RUNNING
                }
            }
            (
                _,
                &mut WhileAll {
                    ref mut condition_state,
                    ref mut check_condition,
                    ref loop_body,
                    ref mut loop_body_index,
                    ref mut loop_body_state,
                },
            ) => {
                let mut remaining_dt = upd.unwrap_or(0.0);
                loop {
                    // check run condition only if allowed at this time:
                    if *check_condition {
                        *check_condition = false;
                        debug_assert!(
                            *loop_body_index == 0,
                            "sequence index should always be 0 when condition is checked!"
                        );
                        match condition_state.tick(e, blackboard, f) {
                            // if running, move to sequence:
                            (Running, _) => {}
                            // if success or failure, get out:
                            x => return x,
                        };
                    }

                    let remaining_e;
                    let ev = match upd {
                        Some(_) => {
                            remaining_e = UpdateEvent::from_dt(remaining_dt, e).unwrap();
                            &remaining_e
                        }
                        _ => e,
                    };

                    match loop_body_state.tick(ev, blackboard, f) {
                        (Failure, x) => return (Failure, x),
                        (Running, _) => {
                            break;
                        }
                        (Success, new_dt) => {
                            // only success moves the sequence cursor forward:
                            *loop_body_index += 1;

                            // If end of repeated events,
                            // start over from the first one
                            // and allow run condition check to happen:
                            if *loop_body_index >= loop_body.len() {
                                *check_condition = true;
                                *loop_body_index = 0;
                            }

                            // Create a new cursor for next event.
                            // Use the same pointer to avoid allocation.
                            **loop_body_state = State::new(loop_body[*loop_body_index].clone());
                            remaining_dt = new_dt;
                        }
                    };
                }
                RUNNING
            }

            // WaitForeverState, WaitState
            _ => RUNNING,
        }
    }
}
