use pyo3::prelude::*;

mod action_args;
mod status;

use action_args::PyActionArgs;
use status::PyStatus;

/// Python bindings for the bonsai-bt behavior-tree library.
///
/// Construct trees with the factory functions (Sequence, Action, Wait, ...),
/// wrap one in `BT(tree, blackboard)`, and drive it with `bt.tick(dt, callback)`.
#[pymodule]
fn bonsai_py(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyStatus>()?;
    m.add_class::<PyActionArgs>()?;
    Ok(())
}
