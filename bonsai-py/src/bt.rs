use bonsai_bt::{Event, Status, UpdateArgs, BT};
use pyo3::exceptions::{PyOSError, PyRuntimeError};
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::action_args::PyActionArgs;
use crate::behavior::{PyAction, PyBehavior};
use crate::status::PyStatus;

const POISONED_MSG: &str = "BT was invalidated by a failed with_telemetry call; construct a new BT";

/// A behavior-tree executor wrapping `bonsai_bt::BT<PyObject, PyObject>`.
///
/// Construct from a tree and a blackboard, then drive with `.tick(dt, callback)`.
/// The callback receives `(args, blackboard)` and must return `(Status, float)`.
#[gen_stub_pyclass]
#[pyclass(unsendable, module = "bonsai_bt", name = "BT")]
pub struct PyBT {
    inner: Option<BT<PyAction, Py<PyAny>>>,
}

impl PyBT {
    fn require_inner(&self) -> PyResult<&BT<PyAction, Py<PyAny>>> {
        self.inner.as_ref().ok_or_else(|| PyRuntimeError::new_err(POISONED_MSG))
    }

    fn require_inner_mut(&mut self) -> PyResult<&mut BT<PyAction, Py<PyAny>>> {
        self.inner.as_mut().ok_or_else(|| PyRuntimeError::new_err(POISONED_MSG))
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyBT {
    #[new]
    fn py_new(behavior: PyRef<'_, PyBehavior>, blackboard: Py<PyAny>) -> Self {
        let tree = behavior.inner.clone();
        Self {
            inner: Some(BT::new(tree, blackboard)),
        }
    }

    fn tick(&mut self, py: Python<'_>, dt: f64, callback: Py<PyAny>) -> PyResult<Option<(PyStatus, f64)>> {
        let inner = self.require_inner_mut()?;
        // `UpdateArgs.dt` is `bonsai_bt::Float`; cast from the f64 Python input.
        let event: Event = UpdateArgs {
            dt: dt as bonsai_bt::Float,
        }
        .into();
        let mut cb_err: Option<PyErr> = None;
        let result = inner.tick(&event, &mut |args, bb: &mut Py<PyAny>| {
            if cb_err.is_some() {
                return (Status::Failure, 0.0);
            }
            let py_args = PyActionArgs::from_rust(&args, py);
            let bb_ref = bb.clone_ref(py);
            match callback.call1(py, (py_args, bb_ref)) {
                // Callback returns Python f64; cast back to `bonsai_bt::Float`.
                Ok(ret) => match ret.extract::<(PyStatus, f64)>(py) {
                    Ok((s, remaining)) => (s.into(), remaining as bonsai_bt::Float),
                    Err(e) => {
                        cb_err = Some(e);
                        (Status::Failure, 0.0)
                    }
                },
                Err(e) => {
                    cb_err = Some(e);
                    (Status::Failure, 0.0)
                }
            }
        });
        if let Some(e) = cb_err {
            return Err(e);
        }
        // Tick result's `Float` -> f64 for the Python return tuple.
        Ok(result.map(|(s, dt)| (s.into(), dt as f64)))
    }

    fn blackboard(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(self.require_inner()?.blackboard().clone_ref(py))
    }

    fn reset_bt(&mut self) -> PyResult<()> {
        self.require_inner_mut()?.reset_bt();
        Ok(())
    }

    fn tick_count(&self) -> PyResult<u64> {
        Ok(self.require_inner()?.tick_count())
    }

    fn is_finished(&self) -> PyResult<bool> {
        Ok(self.require_inner()?.is_finished())
    }

    fn graphviz(&mut self) -> PyResult<String> {
        Ok(self.require_inner_mut()?.get_graphviz())
    }

    #[pyo3(signature = (port, host = "127.0.0.1"))]
    fn with_telemetry<'py>(mut slf: PyRefMut<'py, Self>, port: u16, host: &str) -> PyResult<PyRefMut<'py, Self>> {
        let inner = slf.inner.take().ok_or_else(|| PyRuntimeError::new_err(POISONED_MSG))?;
        match inner.with_telemetry_at(host, port) {
            Ok(new_inner) => {
                slf.inner = Some(new_inner);
                Ok(slf)
            }
            Err(e) => Err(PyOSError::new_err(format!("with_telemetry({host}:{port}) failed: {e}"))),
        }
    }
}
