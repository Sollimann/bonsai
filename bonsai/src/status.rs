// use serde_derive::{Deserialize, Serialize};

/// The result of a behavior or action.
#[derive(Copy, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug)]
pub enum Status {
    /// The behavior or action succeeded.
    Success,
    /// The behavior or action failed.
    Failure,
    /// The behavior or action is still running.
    Running,
}
