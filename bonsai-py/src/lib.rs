use pyo3::prelude::*;

/// Python bindings for the bonsai-bt behavior-tree library.
///
/// Construct trees with the factory functions (Sequence, Action, Wait, ...),
/// wrap one in `BT(tree, blackboard)`, and drive it with `bt.tick(dt, callback)`.
#[pymodule]
fn bonsai_py(_py: Python<'_>, _m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
