use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Behavior, Status};

/// The per-tick payload: maps each visited node's preorder ID to its returned Status.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TickTrace {
    pub tick_id: u64,
    /// Maps Node ID to its return Status for the current tick.
    pub states: HashMap<usize, Status>,
}

/// The immutable structure of the tree, sent once upon WebSocket connection.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TreeDefinition {
    pub root: TreeNode,
}

/// A single node in the static tree layout.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TreeNode {
    pub id: usize,
    pub node_type: String,
    pub label: String,
    pub children: Vec<TreeNode>,
}

impl TreeDefinition {
    /// Walk the behavior tree in DFS preorder, assigning stable integer IDs.
    pub fn build<A: std::fmt::Debug>(behavior: &Behavior<A>) -> Self {
        let mut id_counter = 0;
        let root = Self::traverse(behavior, &mut id_counter);
        Self { root }
    }

    pub(crate) fn traverse<A: std::fmt::Debug>(behavior: &Behavior<A>, id_counter: &mut usize) -> TreeNode {
        let id = *id_counter;
        *id_counter += 1;

        match behavior {
            Behavior::Action(action) => TreeNode {
                id,
                node_type: "Action".to_string(),
                label: format!("{:?}", action),
                children: vec![],
            },
            Behavior::Wait(time) => TreeNode {
                id,
                node_type: "Wait".to_string(),
                label: format!("Wait({:.2}s)", time),
                children: vec![],
            },
            Behavior::WaitForever => TreeNode {
                id,
                node_type: "WaitForever".to_string(),
                label: "WaitForever".to_string(),
                children: vec![],
            },
            Behavior::Invert(child) => TreeNode {
                id,
                node_type: "Inverter".to_string(),
                label: "Inverter".to_string(),
                children: vec![Self::traverse(child, id_counter)],
            },
            Behavior::AlwaysSucceed(child) => TreeNode {
                id,
                node_type: "AlwaysSucceed".to_string(),
                label: "AlwaysSucceed".to_string(),
                children: vec![Self::traverse(child, id_counter)],
            },
            Behavior::Select(children) => TreeNode {
                id,
                node_type: "Selector".to_string(),
                label: "Selector".to_string(),
                children: children.iter().map(|c| Self::traverse(c, id_counter)).collect(),
            },
            Behavior::Sequence(children) => TreeNode {
                id,
                node_type: "Sequence".to_string(),
                label: "Sequence".to_string(),
                children: children.iter().map(|c| Self::traverse(c, id_counter)).collect(),
            },
            // Traverse as [condition, on_success, on_failure] — order must match skip_subtree logic.
            Behavior::If(condition, on_success, on_failure) => TreeNode {
                id,
                node_type: "If".to_string(),
                label: "If".to_string(),
                children: vec![
                    Self::traverse(condition, id_counter),
                    Self::traverse(on_success, id_counter),
                    Self::traverse(on_failure, id_counter),
                ],
            },
            // Condition node first, then body — matches State::While execution order.
            Behavior::While(condition, body) => {
                let mut children = vec![Self::traverse(condition, id_counter)];
                children.extend(body.iter().map(|c| Self::traverse(c, id_counter)));
                TreeNode {
                    id,
                    node_type: "While".to_string(),
                    label: "While".to_string(),
                    children,
                }
            }
            Behavior::WhileAll(condition, body) => {
                let mut children = vec![Self::traverse(condition, id_counter)];
                children.extend(body.iter().map(|c| Self::traverse(c, id_counter)));
                TreeNode {
                    id,
                    node_type: "WhileAll".to_string(),
                    label: "WhileAll".to_string(),
                    children,
                }
            }
            Behavior::WhenAll(children) => TreeNode {
                id,
                node_type: "WhenAll".to_string(),
                label: "WhenAll".to_string(),
                children: children.iter().map(|c| Self::traverse(c, id_counter)).collect(),
            },
            Behavior::WhenAny(children) => TreeNode {
                id,
                node_type: "WhenAny".to_string(),
                label: "WhenAny".to_string(),
                children: children.iter().map(|c| Self::traverse(c, id_counter)).collect(),
            },
            Behavior::After(children) => TreeNode {
                id,
                node_type: "After".to_string(),
                label: "After".to_string(),
                children: children.iter().map(|c| Self::traverse(c, id_counter)).collect(),
            },
            Behavior::Race(children) => TreeNode {
                id,
                node_type: "Race".to_string(),
                label: "Race".to_string(),
                children: children.iter().map(|c| Self::traverse(c, id_counter)).collect(),
            },
        }
    }
}
