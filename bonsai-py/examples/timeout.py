"""
Demonstrates the `Timeout` decorator.

`Timeout(limit, child)` runs `child`, but if `child` is still running once the
accumulated time reaches `limit`, it is abandoned and `Failure` is returned. If
`child` finishes (success or failure) before the limit, its status is passed
through unchanged.

Run:
    python bonsai-py/examples/timeout.py
"""
from __future__ import annotations

import bonsai_bt as bt


def main() -> None:
    # (1) A child that keeps running past the limit is cut off -> Failure.
    tree = bt.BT(bt.Timeout(1.0, bt.Wait(5.0)), None)
    print(f"Timeout(1.0, Wait(5.0)) @ 0.6s -> {tree.tick(0.6, None)[0]} (expect Running)")
    print(f"Timeout(1.0, Wait(5.0)) @ 1.2s -> {tree.tick(0.6, None)[0]} (expect Failure)")

    # (2) A child that finishes before the limit passes through unchanged.
    tree2 = bt.BT(bt.Timeout(10.0, bt.Wait(0.5)), None)
    print(f"Timeout(10.0, Wait(0.5)) @ 0.2s -> {tree2.tick(0.2, None)[0]} (expect Running)")
    print(f"Timeout(10.0, Wait(0.5)) @ 0.6s -> {tree2.tick(0.4, None)[0]} (expect Success)")


if __name__ == "__main__":
    main()
