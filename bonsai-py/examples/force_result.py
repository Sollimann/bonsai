"""
Demonstrates the `AlwaysSucceed` and `AlwaysFail` decorators.

`AlwaysSucceed(A)` coerces a child's Failure into Success — useful for
best-effort nodes (telemetry, logging) that must not abort a Sequence.

`AlwaysFail(A)` coerces a child's Success into Failure — useful for
probes/guards that should always route the tree down a fallback branch.

Run:
    python bonsai-py/examples/force_result.py
"""
from __future__ import annotations

from typing import Any

import bonsai_bt as bt


def flaky_telemetry(_args: Any, _bb: Any) -> tuple[bt.Status, float]:
    """A best-effort node that reports Failure (e.g. telemetry send failed)."""
    return (bt.Status.Failure, 0.0)


def probe(_args: Any, _bb: Any) -> tuple[bt.Status, float]:
    """A probe that reports Success (e.g. the resource was found)."""
    return (bt.Status.Success, 0.0)


def main() -> None:
    # (1) AlwaysSucceed — the flaky telemetry node fails, but the decorator
    #     coerces it to Success so it doesn't abort its parent Sequence.
    telemetry = bt.AlwaysSucceed(bt.Action("flaky-telemetry"))
    tree = bt.BT(telemetry, None)
    result = tree.tick(0.0, flaky_telemetry)
    print(f"AlwaysSucceed(flaky telemetry) -> {result[0]} (expect Success)")

    # (2) AlwaysFail — the probe succeeds, but the decorator coerces it to
    #     Failure so control flow is forced down a fallback branch.
    probe_tree = bt.AlwaysFail(bt.Action("probe"))
    tree2 = bt.BT(probe_tree, None)
    result2 = tree2.tick(0.0, probe)
    print(f"AlwaysFail(probe) -> {result2[0]} (expect Failure)")


if __name__ == "__main__":
    main()
