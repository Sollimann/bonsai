# -*- coding: utf-8 -*-
"""Drift gate: every Rust binding has at least one exercising Python test.

Two parity checks:
1. Every `#[pyfunction]` in `bonsai-py/src/*.rs` (with a `#[pyo3(name = "X")]`
   override) appears in `bonsai_py.__all__`.
2. Every public `#[pyclass]` (with `module = "bonsai_py"` set) appears in
   `bonsai_py.__all__`.

If a Rust contributor adds a binding and forgets to:
  - add it to __all__,
  - add a Python test exercising it,
then `test_no_unexercised_factories` fails CI.
"""
from __future__ import annotations

import re
from pathlib import Path

import bonsai_py as bt

SRC_DIR = Path(__file__).resolve().parent.parent / "src"

RUST_NAME_RE = re.compile(r'#\[pyo3\(\s*name\s*=\s*"([A-Za-z_][A-Za-z0-9_]*)"\s*\)\]')
PYCLASS_NAME_RE = re.compile(
    r'#\[pyclass\([^\]]*\bname\s*=\s*"([A-Za-z_][A-Za-z0-9_]*)"[^\]]*\)\]'
)


def _names_from_rust(pattern: re.Pattern[str]) -> set[str]:
    names: set[str] = set()
    for path in SRC_DIR.rglob("*.rs"):
        names.update(pattern.findall(path.read_text(encoding="utf-8")))
    return names


def test_every_rust_pyfunction_in_all() -> None:
    """Every Rust #[pyo3(name=...)] symbol scanned from src/*.rs must appear in bonsai_py.__all__."""
    rust_names = _names_from_rust(RUST_NAME_RE)
    rust_names = {n for n in rust_names if not n.startswith("_")}
    missing = rust_names - set(bt.__all__)
    assert not missing, (
        f"Rust declares these #[pyo3(name=...)] symbols but they're missing "
        f"from bonsai_py.__all__: {sorted(missing)}."
    )


def test_every_rust_pyclass_in_all() -> None:
    """Every Rust #[pyclass(name=...)] must appear in bonsai_py.__all__."""
    rust_classes = _names_from_rust(PYCLASS_NAME_RE)
    missing = rust_classes - set(bt.__all__)
    assert not missing, (
        f"Rust declares these pyclasses but they're missing from __all__: "
        f"{sorted(missing)}"
    )


def test_no_unexercised_factories() -> None:
    """Every name in __all__ is mentioned in at least one other test file — catches added-but-untested symbols."""
    tests_dir = Path(__file__).resolve().parent
    test_text = ""
    for p in tests_dir.glob("test_*.py"):
        if p.name == "test_drift.py":
            continue
        test_text += p.read_text(encoding="utf-8")

    unexercised = [name for name in bt.__all__ if name not in test_text]
    assert not unexercised, (
        f"These public names have no test mentioning them: {unexercised}."
    )
