use bonsai_bt::Behavior;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// An opaque behavior-tree node.
///
/// Construct via the factory functions (`Sequence`, `Action`, `Wait`, ...)
/// at the module level. Subtrees are reusable - the same `Behavior`
/// can appear as a child of multiple parents.
#[pyclass(unsendable, frozen, module = "bonsai_py", name = "Behavior")]
pub struct PyBehavior {
    pub(crate) inner: Behavior<Py<PyAny>>,
}

impl PyBehavior {
    fn wrap(inner: Behavior<Py<PyAny>>) -> Self {
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

/// GIL-aware recursive clone of a `Behavior<Py<PyAny>>` tree.
///
/// `Py<PyAny>` is not `Clone` in PyO3 0.28 (clone() can't prove the GIL
/// is held), so the derived `Behavior::clone()` is unavailable for our
/// instantiation. This walks the tree manually, bumping the refcount on
/// each action via `clone_ref(py)`.
fn clone_tree(b: &Behavior<Py<PyAny>>, py: Python<'_>) -> Behavior<Py<PyAny>> {
    match b {
        Behavior::Wait(t) => Behavior::Wait(*t),
        Behavior::WaitForever => Behavior::WaitForever,
        Behavior::Action(a) => Behavior::Action(a.clone_ref(py)),
        Behavior::Invert(c) => Behavior::Invert(Box::new(clone_tree(c, py))),
        Behavior::AlwaysSucceed(c) => Behavior::AlwaysSucceed(Box::new(clone_tree(c, py))),
        Behavior::Select(v) => Behavior::Select(clone_vec(v, py)),
        Behavior::If(c, s, f) => Behavior::If(
            Box::new(clone_tree(c, py)),
            Box::new(clone_tree(s, py)),
            Box::new(clone_tree(f, py)),
        ),
        Behavior::Sequence(v) => Behavior::Sequence(clone_vec(v, py)),
        Behavior::While(c, body) => Behavior::While(Box::new(clone_tree(c, py)), clone_vec(body, py)),
        Behavior::WhileAll(c, body) => {
            Behavior::WhileAll(Box::new(clone_tree(c, py)), clone_vec(body, py))
        }
        Behavior::WhenAll(v) => Behavior::WhenAll(clone_vec(v, py)),
        Behavior::WhenAny(v) => Behavior::WhenAny(clone_vec(v, py)),
        Behavior::After(v) => Behavior::After(clone_vec(v, py)),
        Behavior::Race(v) => Behavior::Race(clone_vec(v, py)),
    }
}

fn clone_vec(v: &[Behavior<Py<PyAny>>], py: Python<'_>) -> Vec<Behavior<Py<PyAny>>> {
    v.iter().map(|b| clone_tree(b, py)).collect()
}

fn collect_children(
    children: Vec<PyRef<'_, PyBehavior>>,
    py: Python<'_>,
) -> Vec<Behavior<Py<PyAny>>> {
    children.iter().map(|c| clone_tree(&c.inner, py)).collect()
}

// ----- Leaves --------------------------------------------------------------

#[pyfunction]
#[pyo3(name = "Action")]
pub fn action_fn(action: Py<PyAny>) -> PyBehavior {
    PyBehavior::wrap(Behavior::Action(action))
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
pub fn invert_fn(py: Python<'_>, child: PyRef<'_, PyBehavior>) -> PyBehavior {
    PyBehavior::wrap(Behavior::Invert(Box::new(clone_tree(&child.inner, py))))
}

#[pyfunction]
#[pyo3(name = "AlwaysSucceed")]
pub fn always_succeed_fn(py: Python<'_>, child: PyRef<'_, PyBehavior>) -> PyBehavior {
    PyBehavior::wrap(Behavior::AlwaysSucceed(Box::new(clone_tree(&child.inner, py))))
}

// ----- Composites (Vec<children>) -----------------------------------------

#[pyfunction]
#[pyo3(name = "Sequence")]
pub fn sequence_fn(py: Python<'_>, children: Vec<PyRef<'_, PyBehavior>>) -> PyBehavior {
    PyBehavior::wrap(Behavior::Sequence(collect_children(children, py)))
}

#[pyfunction]
#[pyo3(name = "Select")]
pub fn select_fn(py: Python<'_>, children: Vec<PyRef<'_, PyBehavior>>) -> PyBehavior {
    PyBehavior::wrap(Behavior::Select(collect_children(children, py)))
}

#[pyfunction]
#[pyo3(name = "WhenAll")]
pub fn when_all_fn(py: Python<'_>, children: Vec<PyRef<'_, PyBehavior>>) -> PyBehavior {
    PyBehavior::wrap(Behavior::WhenAll(collect_children(children, py)))
}

#[pyfunction]
#[pyo3(name = "WhenAny")]
pub fn when_any_fn(py: Python<'_>, children: Vec<PyRef<'_, PyBehavior>>) -> PyBehavior {
    PyBehavior::wrap(Behavior::WhenAny(collect_children(children, py)))
}

#[pyfunction]
#[pyo3(name = "After")]
pub fn after_fn(py: Python<'_>, children: Vec<PyRef<'_, PyBehavior>>) -> PyBehavior {
    PyBehavior::wrap(Behavior::After(collect_children(children, py)))
}

#[pyfunction]
#[pyo3(name = "Race")]
pub fn race_fn(py: Python<'_>, children: Vec<PyRef<'_, PyBehavior>>) -> PyBehavior {
    PyBehavior::wrap(Behavior::Race(collect_children(children, py)))
}

// ----- Control flow --------------------------------------------------------

#[pyfunction]
#[pyo3(name = "If")]
pub fn if_fn(
    py: Python<'_>,
    cond: PyRef<'_, PyBehavior>,
    on_success: PyRef<'_, PyBehavior>,
    on_failure: PyRef<'_, PyBehavior>,
) -> PyBehavior {
    PyBehavior::wrap(Behavior::If(
        Box::new(clone_tree(&cond.inner, py)),
        Box::new(clone_tree(&on_success.inner, py)),
        Box::new(clone_tree(&on_failure.inner, py)),
    ))
}

#[pyfunction]
#[pyo3(name = "While")]
pub fn while_fn(
    py: Python<'_>,
    cond: PyRef<'_, PyBehavior>,
    body: Vec<PyRef<'_, PyBehavior>>,
) -> PyResult<PyBehavior> {
    if body.is_empty() {
        return Err(PyValueError::new_err("While: body must not be empty"));
    }
    Ok(PyBehavior::wrap(Behavior::While(
        Box::new(clone_tree(&cond.inner, py)),
        collect_children(body, py),
    )))
}

#[pyfunction]
#[pyo3(name = "WhileAll")]
pub fn while_all_fn(
    py: Python<'_>,
    cond: PyRef<'_, PyBehavior>,
    body: Vec<PyRef<'_, PyBehavior>>,
) -> PyResult<PyBehavior> {
    if body.is_empty() {
        return Err(PyValueError::new_err("WhileAll: body must not be empty"));
    }
    Ok(PyBehavior::wrap(Behavior::WhileAll(
        Box::new(clone_tree(&cond.inner, py)),
        collect_children(body, py),
    )))
}
