# -*- coding: utf-8 -*-
"""Module-level surface: __version__, __all__, __doc__, RUNNING."""
from __future__ import annotations

import bonsai_py as bt


def test_version_present() -> None:
    """bt.__version__ pins the wheel version (0.12.0); bump per release."""
    assert bt.__version__ == "0.12.0"


def test_docstring_present() -> None:
    """Module docstring is non-empty and mentions behavior trees."""
    assert bt.__doc__
    assert "behavior" in bt.__doc__.lower()


def test_all_contents() -> None:
    """__all__ contains exactly the 4 types + 14 factories + RUNNING = 19 names."""
    expected = {
        "Status", "ActionArgs", "Behavior", "BT",
        "Action", "Wait", "WaitForever",
        "Invert", "AlwaysSucceed",
        "Sequence", "Select", "WhenAll", "WhenAny", "After", "Race",
        "If", "While", "WhileAll",
        "RUNNING",
    }
    assert set(bt.__all__) == expected


def test_all_names_are_accessible() -> None:
    """Every name listed in __all__ is actually attached to the module."""
    for name in bt.__all__:
        assert hasattr(bt, name), f"missing {name}"


def test_running_constant() -> None:
    """bt.RUNNING is the immutable tuple (Status.Running, 0.0)."""
    assert bt.RUNNING == (bt.Status.Running, 0.0)
    assert isinstance(bt.RUNNING, tuple)


def test_stub_present() -> None:
    """The auto-generated .pyi stub ships alongside the wheel."""
    from pathlib import Path
    stub = Path(bt.__file__).parent / "__init__.pyi"
    assert stub.exists()
