use bonsai_bt::Behavior;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Wrapper around `Py<PyAny>` that satisfies `bonsai_bt::BT<A, B>`'s
/// `A: Clone + Debug` bounds.
///
/// `Py<PyAny>` itself is not `Clone` in PyO3 0.28 (the trait was removed
/// because `.clone()` cannot statically prove the GIL is held). We satisfy
/// the bound by acquiring the GIL inside the `Clone` impl via
/// `Python::attach` and forwarding to `clone_ref(py)`. Inside a
/// `#[pymethods]` context the GIL is already held, so re-entry is cheap.
pub(crate) struct PyAction(pub(crate) Py<PyAny>);

impl Clone for PyAction {
    fn clone(&self) -> Self {
        Python::attach(|py| PyAction(self.0.clone_ref(py)))
    }
}

impl std::fmt::Debug for PyAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Show the Python repr in tree-definition output (used by the
        // telemetry visualizer). Fall back to a placeholder if repr fails.
        Python::attach(|py| match self.0.bind(py).repr() {
            Ok(s) => write!(f, "{s}"),
            Err(_) => write!(f, "<repr failed>"),
        })
    }
}

/// An opaque behavior-tree node.
///
/// Construct via the factory functions (`Sequence`, `Action`, `Wait`, ...)
/// at the module level. Subtrees are reusable - the same `Behavior`
/// can appear as a child of multiple parents.
#[pyclass(unsendable, frozen, module = "bonsai_py", name = "Behavior")]
pub struct PyBehavior {
    pub(crate) inner: Behavior<PyAction>,
}

impl PyBehavior {
    fn wrap(inner: Behavior<PyAction>) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyBehavior {
    fn __repr__(&self) -> String {
        match &self.inner {
            Behavior::Wait(t) => format!("Wait({t})"),
            Behavior::WaitForever => "WaitForever".to_string(),
            Behavior::Action(_) => "Action(...)".to_string(),
            Behavior::Invert(_) => "Invert(...)".to_string(),
            Behavior::AlwaysSucceed(_) => "AlwaysSucceed(...)".to_string(),
            Behavior::Select(v) => format!("Select({})", v.len()),
            Behavior::If(_, _, _) => "If(...)".to_string(),
            Behavior::Sequence(v) => format!("Sequence({})", v.len()),
            Behavior::While(_, body) => format!("While({})", body.len()),
            Behavior::WhileAll(_, body) => format!("WhileAll({})", body.len()),
            Behavior::WhenAll(v) => format!("WhenAll({})", v.len()),
            Behavior::WhenAny(v) => format!("WhenAny({})", v.len()),
            Behavior::After(v) => format!("After({})", v.len()),
            Behavior::Race(v) => format!("Race({})", v.len()),
        }
    }
}

fn collect_children(children: Vec<PyRef<'_, PyBehavior>>) -> Vec<Behavior<PyAction>> {
    children.iter().map(|c| c.inner.clone()).collect()
}

// ----- Leaves --------------------------------------------------------------

#[pyfunction]
#[pyo3(name = "Action")]
pub fn action_fn(action: Py<PyAny>) -> PyBehavior {
    PyBehavior::wrap(Behavior::Action(PyAction(action)))
}

#[pyfunction]
#[pyo3(name = "Wait")]
pub fn wait_fn(seconds: f64) -> PyResult<PyBehavior> {
    if seconds.is_nan() {
        return Err(PyValueError::new_err("Wait: seconds must not be NaN"));
    }
    Ok(PyBehavior::wrap(Behavior::Wait(seconds)))
}

#[pyfunction]
#[pyo3(name = "WaitForever")]
pub fn wait_forever_fn() -> PyBehavior {
    PyBehavior::wrap(Behavior::WaitForever)
}

// ----- Decorators ----------------------------------------------------------

#[pyfunction]
#[pyo3(name = "Invert")]
pub fn invert_fn(child: PyRef<'_, PyBehavior>) -> PyBehavior {
    PyBehavior::wrap(Behavior::Invert(Box::new(child.inner.clone())))
}

#[pyfunction]
#[pyo3(name = "AlwaysSucceed")]
pub fn always_succeed_fn(child: PyRef<'_, PyBehavior>) -> PyBehavior {
    PyBehavior::wrap(Behavior::AlwaysSucceed(Box::new(child.inner.clone())))
}

// ----- Composites (Vec<children>) -----------------------------------------

#[pyfunction]
#[pyo3(name = "Sequence")]
pub fn sequence_fn(children: Vec<PyRef<'_, PyBehavior>>) -> PyBehavior {
    PyBehavior::wrap(Behavior::Sequence(collect_children(children)))
}

#[pyfunction]
#[pyo3(name = "Select")]
pub fn select_fn(children: Vec<PyRef<'_, PyBehavior>>) -> PyBehavior {
    PyBehavior::wrap(Behavior::Select(collect_children(children)))
}

#[pyfunction]
#[pyo3(name = "WhenAll")]
pub fn when_all_fn(children: Vec<PyRef<'_, PyBehavior>>) -> PyBehavior {
    PyBehavior::wrap(Behavior::WhenAll(collect_children(children)))
}

#[pyfunction]
#[pyo3(name = "WhenAny")]
pub fn when_any_fn(children: Vec<PyRef<'_, PyBehavior>>) -> PyBehavior {
    PyBehavior::wrap(Behavior::WhenAny(collect_children(children)))
}

#[pyfunction]
#[pyo3(name = "After")]
pub fn after_fn(children: Vec<PyRef<'_, PyBehavior>>) -> PyBehavior {
    PyBehavior::wrap(Behavior::After(collect_children(children)))
}

#[pyfunction]
#[pyo3(name = "Race")]
pub fn race_fn(children: Vec<PyRef<'_, PyBehavior>>) -> PyBehavior {
    PyBehavior::wrap(Behavior::Race(collect_children(children)))
}

// ----- Control flow --------------------------------------------------------

#[pyfunction]
#[pyo3(name = "If")]
pub fn if_fn(
    cond: PyRef<'_, PyBehavior>,
    on_success: PyRef<'_, PyBehavior>,
    on_failure: PyRef<'_, PyBehavior>,
) -> PyBehavior {
    PyBehavior::wrap(Behavior::If(
        Box::new(cond.inner.clone()),
        Box::new(on_success.inner.clone()),
        Box::new(on_failure.inner.clone()),
    ))
}

#[pyfunction]
#[pyo3(name = "While")]
pub fn while_fn(
    cond: PyRef<'_, PyBehavior>,
    body: Vec<PyRef<'_, PyBehavior>>,
) -> PyResult<PyBehavior> {
    if body.is_empty() {
        return Err(PyValueError::new_err("While: body must not be empty"));
    }
    Ok(PyBehavior::wrap(Behavior::While(
        Box::new(cond.inner.clone()),
        collect_children(body),
    )))
}

#[pyfunction]
#[pyo3(name = "WhileAll")]
pub fn while_all_fn(
    cond: PyRef<'_, PyBehavior>,
    body: Vec<PyRef<'_, PyBehavior>>,
) -> PyResult<PyBehavior> {
    if body.is_empty() {
        return Err(PyValueError::new_err("WhileAll: body must not be empty"));
    }
    Ok(PyBehavior::wrap(Behavior::WhileAll(
        Box::new(cond.inner.clone()),
        collect_children(body),
    )))
}
