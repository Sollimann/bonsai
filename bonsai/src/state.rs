use crate::event::UpdateEvent;
use crate::sequence::{reactive_sequence, sequence, ReactiveSequenceArgs, SequenceArgs};
use crate::state::State::*;
use crate::status::Status::*;
use crate::tracer::{first_child_id, next_sibling_id, NodeMeta, Tracer};
use crate::when_all::{when_all, WhenAllArgs};
use crate::{Behavior, Float, Status};
use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The action is still running, and thus the action consumes
/// all the remaining delta time for the tick
pub const RUNNING: (Status, Float) = (Running, 0.0);

/// The arguments in the action callback.
pub struct ActionArgs<'a, E: 'a, A: 'a> {
    /// The event.
    pub event: &'a E,
    /// The remaining delta time. When one action terminates,
    /// it can consume some of dt and the remaining is passed
    /// onto the next action.
    pub dt: Float,
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
    Wait { time_to_wait: Float, elapsed_time: Float },
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
    /// Keeps track of a `ReactiveSequence` behavior.
    ///
    /// No `current_index` because every tick walks the children from 0. The
    /// `cursor` box is allocated once and overwritten in place before each
    /// child's tick, so the composite uses a single `Box` for its lifetime.
    ReactiveSequence {
        /// Children, re-walked in order on every tick.
        behaviors: Vec<Behavior<A>>,
        /// Scratch slot for the child being ticked. Overwritten in place; never re-allocated.
        cursor: Box<State<A>>,
    },
    /// Same shape as [`State::ReactiveSequence`]; success short-circuits and
    /// all-fail returns `Failure`.
    ReactiveSelect {
        /// Children, re-walked in order on every tick.
        behaviors: Vec<Behavior<A>>,
        /// Scratch slot for the child being ticked. Overwritten in place; never re-allocated.
        cursor: Box<State<A>>,
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
    /// Keeps track of a `Race` behavior.
    Race(Vec<Option<State<A>>>),
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
            Behavior::ReactiveSequence(behaviors) => State::ReactiveSequence {
                behaviors,
                // ZST placeholder — overwritten on the first tick, so cloning
                // a real child here would just be thrown away.
                cursor: Box::new(State::WaitForever),
            },
            Behavior::ReactiveSelect(behaviors) => State::ReactiveSelect {
                behaviors,
                cursor: Box::new(State::WaitForever),
            },
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
            Behavior::Race(behaviors) => State::Race(behaviors.into_iter().map(|ev| Some(State::new(ev))).collect()),
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
    /// return: (Status, Float)
    /// function returns the result of the tree traversal, and how long
    /// it actually took to complete the traversal and propagate the
    /// results back up to the root node
    pub(crate) fn tick<E, F, B, T>(
        &mut self,
        self_id: usize,
        metas: &[NodeMeta],
        e: &E,
        blackboard: &mut B,
        f: &mut F,
        tracer: &mut T,
    ) -> (Status, Float)
    where
        E: UpdateEvent,
        F: FnMut(ActionArgs<E, A>, &mut B) -> (Status, Float),
        T: Tracer,
    {
        let upd = e.update(|args| Some(args.dt)).unwrap_or(None);

        // double match statements
        match (upd, self) {
            (_, &mut Action(ref action)) => {
                let result = f(
                    ActionArgs {
                        event: e,
                        dt: upd.unwrap_or(0.0),
                        action,
                    },
                    blackboard,
                );
                tracer.record(self_id, result.0);
                result
            }
            (_, &mut Invert(ref mut cur)) => {
                let child_id = first_child_id::<T>(self_id);
                let result = match cur.tick(child_id, metas, e, blackboard, f, tracer) {
                    (Running, dt) => (Running, dt),
                    (Failure, dt) => (Success, dt),
                    (Success, dt) => (Failure, dt),
                };
                tracer.record(self_id, result.0);
                result
            }
            (_, &mut AlwaysSucceed(ref mut cur)) => {
                let child_id = first_child_id::<T>(self_id);
                let result = match cur.tick(child_id, metas, e, blackboard, f, tracer) {
                    (Running, dt) => (Running, dt),
                    (_, dt) => (Success, dt),
                };
                tracer.record(self_id, result.0);
                result
            }
            (
                Some(dt),
                &mut Wait {
                    time_to_wait,
                    ref mut elapsed_time,
                },
            ) => {
                *elapsed_time += dt;
                let result = if *elapsed_time >= time_to_wait {
                    let time_overdue = *elapsed_time - time_to_wait;
                    *elapsed_time = time_to_wait;
                    (Success, time_overdue)
                } else {
                    RUNNING
                };
                tracer.record(self_id, result.0);
                result
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
                let cond_id = first_child_id::<T>(self_id);
                let on_success_id = next_sibling_id::<T>(metas, cond_id);
                let on_failure_id = next_sibling_id::<T>(metas, on_success_id);
                let mut remaining_dt = upd.unwrap_or(0.0);
                let remaining_e;
                // Run in a loop to evaluate success or failure with
                // remaining delta time after condition.
                let result = loop {
                    *status = match *status {
                        Running => match current_state.tick(cond_id, metas, e, blackboard, f, tracer) {
                            (Running, dt) => break (Running, dt),
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
                        s => {
                            let branch_id = if s == Success { on_success_id } else { on_failure_id };
                            let ev = match upd {
                                Some(_) => {
                                    remaining_e = UpdateEvent::from_dt(remaining_dt, e).unwrap();
                                    &remaining_e
                                }
                                _ => e,
                            };
                            break current_state.tick(branch_id, metas, ev, blackboard, f, tracer);
                        }
                    }
                };
                tracer.record(self_id, result.0);
                result
            }
            (
                _,
                &mut Select {
                    behaviors: ref seq,
                    current_index: ref mut i,
                    current_state: ref mut cursor,
                },
            ) => {
                let select = true;
                let result = sequence(SequenceArgs {
                    select,
                    upd,
                    seq,
                    i,
                    cursor,
                    e,
                    f,
                    blackboard,
                    parent_id: self_id,
                    metas,
                    tracer,
                });
                tracer.record(self_id, result.0);
                result
            }
            (
                _,
                &mut Sequence {
                    behaviors: ref seq,
                    current_index: ref mut i,
                    current_state: ref mut cursor,
                },
            ) => {
                let select = false;
                let result = sequence(SequenceArgs {
                    select,
                    upd,
                    seq,
                    i,
                    cursor,
                    e,
                    f,
                    blackboard,
                    parent_id: self_id,
                    metas,
                    tracer,
                });
                tracer.record(self_id, result.0);
                result
            }
            (
                _,
                &mut ReactiveSequence {
                    behaviors: ref seq,
                    ref mut cursor,
                },
            ) => {
                let result = reactive_sequence(ReactiveSequenceArgs {
                    select: false,
                    upd,
                    seq,
                    cursor,
                    e,
                    f,
                    blackboard,
                    parent_id: self_id,
                    metas,
                    tracer,
                });
                tracer.record(self_id, result.0);
                result
            }
            (
                _,
                &mut ReactiveSelect {
                    behaviors: ref seq,
                    ref mut cursor,
                },
            ) => {
                let result = reactive_sequence(ReactiveSequenceArgs {
                    select: true,
                    upd,
                    seq,
                    cursor,
                    e,
                    f,
                    blackboard,
                    parent_id: self_id,
                    metas,
                    tracer,
                });
                tracer.record(self_id, result.0);
                result
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
                let cond_id = first_child_id::<T>(self_id);
                let body_0_id = next_sibling_id::<T>(metas, cond_id);
                let mut current_body_id = if T::IS_RECORDING {
                    let mut id = body_0_id;
                    for _ in 0..*loop_body_index {
                        id = next_sibling_id::<T>(metas, id);
                    }
                    id
                } else {
                    usize::MAX
                };
                // If the condition behavior terminates, do not execute the loop.
                match condition_state.tick(cond_id, metas, e, blackboard, f, tracer) {
                    (Running, _) => {}
                    x => {
                        tracer.record(self_id, x.0);
                        return x;
                    }
                };
                let cur = loop_body_state;
                let mut remaining_dt = upd.unwrap_or(0.0);
                let mut remaining_e;
                let result = loop {
                    let ev = match upd {
                        Some(_) => {
                            remaining_e = UpdateEvent::from_dt(remaining_dt, e).unwrap();
                            &remaining_e
                        }
                        _ => e,
                    };
                    match cur.tick(current_body_id, metas, ev, blackboard, f, tracer) {
                        (Failure, x) => break (Failure, x),
                        (Running, _) => break RUNNING,
                        (Success, new_dt) => {
                            remaining_dt = match upd {
                                // Change update event with remaining delta time.
                                Some(_) => new_dt,
                                // Other events are 'consumed' and not passed to next.
                                _ => break RUNNING,
                            };
                        }
                    };
                    *loop_body_index += 1;
                    if T::IS_RECORDING {
                        current_body_id = next_sibling_id::<T>(metas, current_body_id);
                    }
                    // If end of repeated events,
                    // start over from the first one.
                    if *loop_body_index >= loop_body.len() {
                        *loop_body_index = 0;
                        if T::IS_RECORDING {
                            current_body_id = body_0_id;
                        }
                    }
                    // Create a new cursor for next event.
                    // Use the same pointer to avoid allocation.
                    **cur = State::new(loop_body[*loop_body_index].clone());
                };
                tracer.record(self_id, result.0);
                result
            }
            (_, &mut WhenAll(ref mut cursors)) => {
                let result = when_all(WhenAllArgs {
                    any: false,
                    upd,
                    cursors,
                    e,
                    blackboard,
                    f,
                    parent_id: self_id,
                    metas,
                    tracer,
                });
                tracer.record(self_id, result.0);
                result
            }
            (_, &mut WhenAny(ref mut cursors)) => {
                let result = when_all(WhenAllArgs {
                    any: true,
                    upd,
                    cursors,
                    e,
                    blackboard,
                    f,
                    parent_id: self_id,
                    metas,
                    tracer,
                });
                tracer.record(self_id, result.0);
                result
            }
            (_, &mut Race(ref mut cursors)) => {
                // return the result of the first child to complete,
                // regardless of whether it succeeds or fails.
                let mut child_id = first_child_id::<T>(self_id);
                for cur in cursors.iter_mut() {
                    let this_id = child_id;
                    child_id = next_sibling_id::<T>(metas, this_id);
                    match *cur {
                        None => {}
                        Some(ref mut state) => match state.tick(this_id, metas, e, blackboard, f, tracer) {
                            (Running, _) => continue,
                            (status, dt) => {
                                tracer.record(self_id, status);
                                return (status, dt);
                            }
                        },
                    }
                }
                tracer.record(self_id, Running);
                RUNNING
            }
            (
                _,
                &mut After {
                    ref mut next_success_index,
                    ref mut states,
                },
            ) => {
                // Get the least delta time left over.
                let mut min_dt = Float::MAX;
                let mut child_id = first_child_id::<T>(self_id);
                if T::IS_RECORDING {
                    for _ in 0..*next_success_index {
                        child_id = next_sibling_id::<T>(metas, child_id);
                    }
                }
                for (j, item) in states.iter_mut().enumerate().skip(*next_success_index) {
                    let this_id = child_id;
                    child_id = next_sibling_id::<T>(metas, this_id);
                    match item.tick(this_id, metas, e, blackboard, f, tracer) {
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
                                tracer.record(self_id, Failure);
                                return (Failure, min_dt.min(new_dt));
                            }
                        }
                        (Failure, new_dt) => {
                            tracer.record(self_id, Failure);
                            return (Failure, new_dt);
                        }
                    };
                }
                let result = if *next_success_index == states.len() {
                    (Success, min_dt)
                } else {
                    RUNNING
                };
                tracer.record(self_id, result.0);
                result
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
                let cond_id = first_child_id::<T>(self_id);
                let body_0_id = next_sibling_id::<T>(metas, cond_id);
                let mut current_body_id = if T::IS_RECORDING {
                    let mut id = body_0_id;
                    for _ in 0..*loop_body_index {
                        id = next_sibling_id::<T>(metas, id);
                    }
                    id
                } else {
                    usize::MAX
                };
                let mut remaining_dt = upd.unwrap_or(0.0);
                let mut remaining_e;
                let result = loop {
                    // check run condition only if allowed at this time:
                    if *check_condition {
                        *check_condition = false;
                        debug_assert!(
                            *loop_body_index == 0,
                            "sequence index should always be 0 when condition is checked!"
                        );
                        match condition_state.tick(cond_id, metas, e, blackboard, f, tracer) {
                            // if running, move to sequence:
                            (Running, _) => {}
                            // if success or failure, get out:
                            x => break x,
                        };
                    }

                    let ev = match upd {
                        Some(_) => {
                            remaining_e = UpdateEvent::from_dt(remaining_dt, e).unwrap();
                            &remaining_e
                        }
                        _ => e,
                    };

                    match loop_body_state.tick(current_body_id, metas, ev, blackboard, f, tracer) {
                        (Failure, x) => break (Failure, x),
                        (Running, _) => break RUNNING,
                        (Success, new_dt) => {
                            // only success moves the sequence cursor forward:
                            *loop_body_index += 1;
                            if T::IS_RECORDING {
                                current_body_id = next_sibling_id::<T>(metas, current_body_id);
                            }

                            // If end of repeated events,
                            // start over from the first one
                            // and allow run condition check to happen:
                            if *loop_body_index >= loop_body.len() {
                                *check_condition = true;
                                *loop_body_index = 0;
                                if T::IS_RECORDING {
                                    current_body_id = body_0_id;
                                }
                            }

                            // Create a new cursor for next event.
                            // Use the same pointer to avoid allocation.
                            **loop_body_state = State::new(loop_body[*loop_body_index].clone());
                            remaining_dt = new_dt;
                        }
                    };
                };
                tracer.record(self_id, result.0);
                result
            }

            // WaitForeverState, WaitState
            _ => {
                tracer.record(self_id, Running);
                RUNNING
            }
        }
    }
}
