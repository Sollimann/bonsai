use bonsai_bt::{ActionArgs, Event};
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::behavior::PyAction;

/// Action callback arguments.
///
/// Constructed by the tick bridge and passed to the user's callback.
/// The Rust `ActionArgs::event` field is intentionally not exposed —
/// Python users only see `dt` and `action`.
#[gen_stub_pyclass]
#[pyclass(frozen, module = "bonsai_py", name = "ActionArgs")]
pub struct PyActionArgs {
    /// Remaining delta time in seconds.
    #[pyo3(get)]
    pub dt: f64,
    /// The user-supplied action value (whatever was passed to `bt.Action(...)`).
    #[pyo3(get)]
    pub action: Py<PyAny>,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyActionArgs {
    #[new]
    fn py_new(dt: f64, action: Py<PyAny>) -> Self {
        Self { dt, action }
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let action_repr = self.action.bind(py).repr()?.to_string();
        Ok(format!("ActionArgs(dt={}, action={})", self.dt, action_repr))
    }
}

impl PyActionArgs {
    /// Build a `PyActionArgs` from the Rust `ActionArgs` that the tick
    /// callback receives. Hot path — one `clone_ref` plus an `f64` copy.
    pub(crate) fn from_rust(args: &ActionArgs<Event, PyAction>, py: Python<'_>) -> Self {
        Self {
            // `args.dt` is `bonsai_bt::Float` (f32 or f64 per feature). Cast to
            // f64 at the Python boundary — Python's `float` is always f64.
            dt: args.dt as f64,
            action: args.action.0.clone_ref(py),
        }
    }
}
