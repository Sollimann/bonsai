// use serde_derive::{Deserialize, Serialize};

/// The result of a behavior or action.
///
/// A tree node that receives a tick signal executes it's callback. The callback
/// must return either:
/// * Success
/// * Failure or
/// * RUNNING, if the action is asynchronous and it needs more time to complete
#[derive(Copy, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug)]
pub enum Status {
    /// The behavior or action succeeded.
    Success,
    /// The behavior or action failed.
    Failure,
    /// The behavior or action is still running.
    ///
    /// 'Running' is usually returned by nodes that has long-
    /// running operations (e.g NavigatetoGoal, CountToHundred) and nodes
    /// that has operations that are everlasting (e.g ComputePI, AvoidObstacles)
    /// with no clear definition of an end-state
    Running,
}
