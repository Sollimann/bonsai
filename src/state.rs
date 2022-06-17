// use crate::state::Behavior::{Action, ActionStat}
// use crate::{behavior::Behavior, status::Status};
// use serde::{Deserialize, Serialize};

use crate::{Behavior, Status};
// use serde_derive::{Deserialize, Serialize};

/// The action is still running.
pub const RUNNING: (Status, f64) = (Status::Running, 0.0);

/// The arguments in the action callback.
pub struct ActionArgs<'a, E: 'a, A: 'a, S: 'a> {
    /// The event.
    pub event: &'a E,
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
}
