# bonsai-py scripts

Developer scripts for working on the `bonsai_py` extension module.

## Scripts

| Script | Purpose |
|---|---|
| [regen-stubs.sh](regen-stubs.sh) | Regenerates `python/bonsai_py/__init__.pyi` from the `#[gen_stub_*]` annotations on the Rust side. Run after editing any annotated `#[pyclass]` / `#[pyfunction]` / `#[pymethods]`. Also runs automatically via the `regen-stubs` pre-commit hook and is enforced in CI. |

## Prerequisites

A Python venv with the `bonsai_py` extension built in. See [../README.md](../README.md#installation-dev) for the one-time setup (`python -m venv .venv`, activate, `pip install maturin`, `maturin develop --release`).

## Running

From the repository root, with the venv activated:

```bash
# Regenerate stubs after editing Rust annotations
bash bonsai-py/scripts/regen-stubs.sh
```
