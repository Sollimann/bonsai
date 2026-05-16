# bonsai-py scripts

Developer scripts for working on the `bonsai_py` extension module.

## Scripts

| Script | Purpose |
|---|---|
| [regen-stubs.sh](regen-stubs.sh) | Regenerates `python/bonsai_py/__init__.pyi` from the `#[gen_stub_*]` annotations on the Rust side. Run after editing any annotated `#[pyclass]` / `#[pyfunction]` / `#[pymethods]`. Also runs automatically via the `regen-stubs` pre-commit hook and is enforced in CI. |
| [verify.py](verify.py) | End-to-end smoke test of the installed wheel: imports `bonsai_py`, exercises every public API (Status, ActionArgs, all 14 factories, BT tick/telemetry, type stub presence, mypy `--strict` on a sample script). Standalone — not a pytest suite (the long-form behavior coverage lives in [../tests/](../tests/)). |

## Running

From the repository root, with a venv that has the wheel installed (`maturin develop --release` from `bonsai-py/`):

```bash
# Regenerate stubs after editing Rust annotations
bash bonsai-py/scripts/regen-stubs.sh

# Run the verification smoke test (every test always runs; exits 0 on success)
python bonsai-py/scripts/verify.py
```

## Dependencies

`verify.py` uses `mypy` for the strict-typing check (skipped with a warning if `mypy` is not installed):

```bash
pip install mypy
```
