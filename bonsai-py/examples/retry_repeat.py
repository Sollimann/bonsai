"""
Demonstrates the `Retry` and `Repeat` decorators.

`Retry(n, child)` re-runs `child` until it succeeds, up to `n` attempts in
total. `Repeat(n, child)` re-runs `child` up to `n` times as long as it keeps
succeeding, and stops on the first `Failure`.

Both restart their child from its initial state on every attempt, so a `Wait`
inside them waits the full duration each time.

Run:
    python bonsai-py/examples/retry_repeat.py
"""
from __future__ import annotations

from typing import Any

import bonsai_bt as bt


def main() -> None:
    # (1) Retry(3, flaky): the third attempt succeeds.
    attempts = 0

    def flaky(_args: Any, _bb: Any) -> tuple[bt.Status, float]:
        nonlocal attempts
        attempts += 1
        status = bt.Status.Failure if attempts < 3 else bt.Status.Success
        return (status, 0.0)

    tree = bt.BT(bt.Retry(3, bt.Action("flaky")), None)
    status = tree.tick(1.0, flaky)[0]
    print(f"Retry(3, flaky) -> {status} after {attempts} attempt(s) (expect Success after 3)")

    # (2) Retry(2, always failing): the budget is exhausted -> Failure.
    def always_fails(_args: Any, _bb: Any) -> tuple[bt.Status, float]:
        return (bt.Status.Failure, 0.0)

    tree2 = bt.BT(bt.Retry(2, bt.Action("fails")), None)
    print(f"Retry(2, always failing) -> {tree2.tick(1.0, always_fails)[0]} (expect Failure)")

    # (3) Repeat(2, Wait(1.0)): the child restarts after each completion, so the
    # second repetition waits a fresh second.
    tree3 = bt.BT(bt.Repeat(2, bt.Wait(1.0)), None)
    print(f"Repeat(2, Wait(1.0)) @ 1.0s -> {tree3.tick(1.0, None)[0]} (expect Running)")
    print(f"Repeat(2, Wait(1.0)) @ 2.0s -> {tree3.tick(1.0, None)[0]} (expect Success)")


if __name__ == "__main__":
    main()
