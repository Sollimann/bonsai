use pyo3::prelude::*;
use pyo3_stub_gen::define_stub_info_gatherer;

mod action_args;
mod behavior;
mod bt;
mod status;

use action_args::PyActionArgs;
use behavior::{
    action_fn, after_fn, always_succeed_fn, if_fn, invert_fn, race_fn, select_fn, sequence_fn,
    wait_fn, wait_forever_fn, when_all_fn, when_any_fn, while_all_fn, while_fn, PyBehavior,
};
use bt::PyBT;
use status::PyStatus;

/// Python bindings for the bonsai-bt behavior-tree library.
///
/// Construct trees with the factory functions (Sequence, Action, Wait, ...),
/// wrap one in `BT(tree, blackboard)`, and drive it with `bt.tick(dt, callback)`.
#[pymodule]
fn bonsai_py(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyStatus>()?;
    m.add_class::<PyActionArgs>()?;
    m.add_class::<PyBehavior>()?;
    m.add_class::<PyBT>()?;
    m.add_function(wrap_pyfunction!(action_fn, m)?)?;
    m.add_function(wrap_pyfunction!(wait_fn, m)?)?;
    m.add_function(wrap_pyfunction!(wait_forever_fn, m)?)?;
    m.add_function(wrap_pyfunction!(invert_fn, m)?)?;
    m.add_function(wrap_pyfunction!(always_succeed_fn, m)?)?;
    m.add_function(wrap_pyfunction!(sequence_fn, m)?)?;
    m.add_function(wrap_pyfunction!(select_fn, m)?)?;
    m.add_function(wrap_pyfunction!(when_all_fn, m)?)?;
    m.add_function(wrap_pyfunction!(when_any_fn, m)?)?;
    m.add_function(wrap_pyfunction!(after_fn, m)?)?;
    m.add_function(wrap_pyfunction!(race_fn, m)?)?;
    m.add_function(wrap_pyfunction!(if_fn, m)?)?;
    m.add_function(wrap_pyfunction!(while_fn, m)?)?;
    m.add_function(wrap_pyfunction!(while_all_fn, m)?)?;

    // Convenience constant matching Rust's `bonsai_bt::RUNNING`.
    m.add("RUNNING", (PyStatus::Running, 0.0_f64).into_pyobject(py)?)?;

    Ok(())
}

// Add pyo3-stub-gen: emits `pub fn stub_info() -> ...` that the
// `stub_gen` binary calls to collect every #[gen_stub_*] annotated item.
define_stub_info_gatherer!(stub_info);
