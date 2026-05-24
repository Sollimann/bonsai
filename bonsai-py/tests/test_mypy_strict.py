"""mypy --strict acceptance test for the typed public surface."""
from __future__ import annotations

import importlib.util
import os
import subprocess
import sys
import tempfile

import pytest


def test_mypy_strict_accepts_sample_script() -> None:
    """mypy --strict accepts a sample script using bonsai_bt's typed surface.

    Catches stub regressions that pytest's runtime tests wouldn't notice —
    e.g. a removed annotation, a wrong type in __init__.pyi, or a missing
    overload. Runs `mypy --strict` on a generated sample exercising
    Status, ActionArgs, Behavior factories, BT.tick, with_telemetry.
    """
    if importlib.util.find_spec("mypy") is None:
        pytest.skip("mypy not installed (pip install mypy to enable)")

    sample = (
        "import bonsai_bt as bt\n"
        "\n"
        "def cb(args: bt.ActionArgs, bb: object) -> tuple[bt.Status, float]:\n"
        '    if args.action == "inc":\n'
        "        return (bt.Status.Success, args.dt)\n"
        "    return bt.RUNNING\n"
        "\n"
        'tree = bt.Sequence([bt.Action("inc"), bt.Wait(1)])  # int coerces to float\n'
        'tree_bt = bt.BT(tree, {"count": 0})\n'
        "for _ in range(3):\n"
        "    res: tuple[bt.Status, float] | None = tree_bt.tick(0.5, cb)\n"
        "    if res is None:\n"
        "        tree_bt.reset_bt()\n"
        "\n"
        'chained: bt.BT = bt.BT(bt.Action("x"), None).with_telemetry(0, host="0.0.0.0")\n'
    )

    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".py", delete=False, encoding="utf-8"
    ) as f:
        f.write(sample)
        sample_path = f.name

    try:
        result = subprocess.run(
            [sys.executable, "-m", "mypy", "--strict", sample_path],
            capture_output=True,
            text=True,
            check=False,
        )
        assert result.returncode == 0, (
            f"mypy --strict failed (exit {result.returncode}):\n"
            f"--- stdout ---\n{result.stdout}\n"
            f"--- stderr ---\n{result.stderr}"
        )
    finally:
        os.unlink(sample_path)
