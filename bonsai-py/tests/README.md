# bonsai-py tests

Pytest suite for the `bonsai_bt` extension module. Primarily used to prevent any drift between the Rust bindings (`bonsai-py/src/*.rs`) and the Python surface: every `#[pyclass]`, `#[pymethods]`, and `#[pyfunction]` has at least one Python test exercising it.

## Test files

| File | What it covers |
|---|---|
| [conftest.py](conftest.py) | Shared fixtures: `free_port`, `noop_callback`, `counting_callback`, `basic_tree`. |
| [test_status.py](test_status.py) | `Status` enum: variants, `eq_int` discriminants (0/1/2), equality, singleton identity, hash, repr, `__module__`, pickle/copy round-trip preserves singleton identity. |
| [test_action_args.py](test_action_args.py) | `ActionArgs`: construct with str/None/dict/object/int-dt; `dt` and `action` are read-only; repr format; `__module__`. |
| [test_behavior.py](test_behavior.py) | All 16 factories exported and callable; each builds with expected repr; validation guards (empty `While`/`WhileAll` body, NaN `Wait`); empty composites + neg/inf/int `Wait` pass through; subtree reuse; kwargs on `If`; tuple accepted / generator rejected for children; identity-based equality per variant; ported Rust `behavior_tests.rs` cases (immediate termination, wait timing, select short-circuit, if branches, invert, always-succeed, when-all, while-loop). |
| [test_bt.py](test_bt.py) | `BT` construction (dict/None/custom blackboard); doctest-equivalent five-tick port; tick return shape; tick on finished returns None; `tick_count` survives `reset_bt`; callback exceptions propagate; bad return shape rejected; `WhenAll` short-circuits siblings after a raise; NaN dt sanitized to 0.0; blackboard identity + mutation persistence; action identity through callback; `reset_bt` preserves blackboard; `RUNNING` constant value and usage. |
| [test_telemetry.py](test_telemetry.py) | `with_telemetry` is chainable; accepts `host` kwarg; second bind on same port raises `OSError`; binding to unreachable IP raises `OSError`; failed `with_telemetry` poisons the BT (every subsequent method raises `RuntimeError`). |
| [test_module.py](test_module.py) | `__version__ == "0.13.0"`; module docstring present; `__all__` lists exactly the 21 expected names; all names accessible; `RUNNING` is `(Status.Running, 0.0)`; `.pyi` stub ships with the wheel. |
| [test_threading_and_pickle.py](test_threading_and_pickle.py) | `BT` is unsendable across threads (PyO3 `PanicException`); `BT`, `Behavior`, `ActionArgs` are unpicklable; `Status` IS picklable (multiprocessing-friendly). |
| [test_performance.py](test_performance.py) | `@pytest.mark.perf`: 100 ticks under 500 ms, 1000 constructions under 5 s. `@pytest.mark.bench` (opt-in): tick throughput microbenchmark. |
| [test_drift.py](test_drift.py) | Checks that every Rust `#[pyo3(name=...)]` symbol appears in `__all__`; every `#[pyclass(name=..., module='bonsai_bt')]` appears in `__all__`; every name in `__all__` is mentioned in at least one other test file. |

## Prerequisites

A Python venv with the `bonsai_bt` extension built in. See [../README.md](../README.md#installation-dev) for the one-time setup (`python -m venv .venv`, activate, `pip install maturin`, `maturin develop --release`).

## Running

From the repository root, with the venv activated:

```bash
# Full suite (default — runs perf budget tests, skips benchmarks)
pytest bonsai-py/tests/

# Verbose
pytest -v bonsai-py/tests/

# Single file
pytest -v bonsai-py/tests/test_status.py

# Single test
pytest -v bonsai-py/tests/test_bt.py::TestTick::test_doctest_equivalent

# Drift gate only
pytest -v bonsai-py/tests/test_drift.py

# Skip perf budget tests
pytest -v -m "not perf" bonsai-py/tests/

# Run microbenchmarks only (prints throughput; no assertions)
pytest -v -m bench bonsai-py/tests/
```

A `pytest-timeout` of 10 seconds per test is configured in [pyproject.toml](../pyproject.toml). If you change a test that legitimately needs longer, bump the per-test timeout with `@pytest.mark.timeout(30)`.

## Dependencies

```bash
pip install pytest pytest-timeout
```

`mypy` is used by [test_mypy_strict.py](test_mypy_strict.py) — install with `pip install mypy`. The test is skipped if `mypy` isn't installed.

## CI

The `pytest` job in [.github/workflows/rust-pr.yml](../../.github/workflows/rust-pr.yml) runs this suite on Python 3.10 and 3.13 (matrix) on every PR, after building the wheel in release mode.
