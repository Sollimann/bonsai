use bonsai_bt::Status;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass_enum, gen_stub_pymethods};

/// Behavior-tree node result.
///
/// Mirrors `bonsai_bt::Status`. Comparable to `int`
/// (`Status.Success == 0`, `Failure == 1`, `Running == 2`) and usable
/// as a `dict` key or `set` member.
#[gen_stub_pyclass_enum]
#[pyclass(eq, eq_int, hash, frozen, from_py_object, module = "bonsai_py", name = "Status")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PyStatus {
    Success,
    Failure,
    Running,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyStatus {
    /// Pickle support: name the singleton by class + variant name, since
    /// PyO3 simple enums refuse construction by call (`Status(0)` raises).
    // The nested-tuple return shape is dictated by Python's pickle protocol
    // (callable, args-tuple); factoring it into a type alias would obscure
    // the contract more than it would clarify.
    #[allow(clippy::type_complexity)]
    fn __reduce__<'py>(&self, py: Python<'py>) -> PyResult<(Bound<'py, PyAny>, (Bound<'py, PyAny>, &'static str))> {
        let getattr = py.import("builtins")?.getattr("getattr")?;
        let cls = py.get_type::<Self>().into_any();
        let name = match self {
            PyStatus::Success => "Success",
            PyStatus::Failure => "Failure",
            PyStatus::Running => "Running",
        };
        Ok((getattr, (cls, name)))
    }
}

impl From<Status> for PyStatus {
    fn from(s: Status) -> Self {
        match s {
            Status::Success => PyStatus::Success,
            Status::Failure => PyStatus::Failure,
            Status::Running => PyStatus::Running,
        }
    }
}

impl From<PyStatus> for Status {
    fn from(s: PyStatus) -> Self {
        match s {
            PyStatus::Success => Status::Success,
            PyStatus::Failure => Status::Failure,
            PyStatus::Running => Status::Running,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_through_rust() {
        for s in [Status::Success, Status::Failure, Status::Running] {
            let py: PyStatus = s.into();
            let back: Status = py.into();
            assert_eq!(s, back);
        }
    }
}
