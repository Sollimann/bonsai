#[cfg(feature = "visualize")]
use std::collections::HashMap;
#[cfg(feature = "visualize")]
use crate::Status;

#[cfg(feature = "visualize")]
#[derive(serde::Serialize, Clone, Debug)]
pub struct TickTrace {
    pub tick_id: u64,

    /// Maps Node ID to its return Status for the current tick.
    pub states: HashMap<usize, Status>
}

#[cfg(feature = "visualize")]
pub const VISUALIZER_HTML: &str = include_str!("index.html");