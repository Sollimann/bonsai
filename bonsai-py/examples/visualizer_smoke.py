"""
End-to-end demo for the WebSocket visualizer.

Drives a deliberately rich 27-node tree (covering 12 of the 14 Behavior
factories), attaches the visualizer via `BT.with_telemetry(8910)`, and
re-runs the tree every ~400 ms wall tick. Each leaf's status follows a
5-step rotation with a per-action phase offset, so a varied mix of
green / yellow / red is visible at any moment. After each complete run,
`reset_bt()` rewinds the cursor; `tick_count` and the telemetry
connection survive, so the browser sees a continuous TickTrace stream
with monotonic `tick_id`.

Demonstrates `with_telemetry`, `reset_bt`, every major factory in one
tree, and a deterministic-cycle callback contract.

Run:
    python bonsai-py/examples/visualizer_smoke.py

Then open <http://127.0.0.1:8910/> in a browser.
  1. Tree renders within ~1 s; status bar reads `connected` and `27 nodes`.
  2. Every ~400 ms leaf colors shift across all subtrees.
  3. `Ctrl-C` and restart -> browser reconnects within <=1 s.

Port 8910 must be free; if it is busy, the script raises OSError.
"""
from __future__ import annotations

import time
from typing import Any

import bonsai_py as bt


def build_tree() -> bt.Behavior:
    return bt.Sequence([
        bt.If(
            bt.Action("low_hp"),
            bt.AlwaysSucceed(bt.Action("flee")),
            bt.Action("regroup"),
        ),
        bt.Select([
            bt.Sequence([
                bt.Action("acquire_target"),
                bt.WhenAll([bt.Action("aim"), bt.Action("track")]),
            ]),
            bt.Race([bt.Action("dodge"), bt.Wait(2.0)]),
            bt.Invert(bt.Action("enemy_visible")),
        ]),
        bt.While(bt.Action("has_ammo"), [bt.Action("fire"), bt.Wait(0.3)]),
        bt.After([bt.Action("cooldown"), bt.Action("ready_signal")]),
        bt.WhenAny([bt.Action("victory_check"), bt.Action("retreat_signal")]),
    ])


# Five-step status cycle visible across all three colors. Each action has a
# unique phase offset so the same wall tick produces a varied mix of statuses
# across the tree (and yellow-Running shows up).
CYCLE = (
    bt.Status.Success,
    bt.Status.Running,
    bt.Status.Failure,
    bt.Status.Success,
    bt.Status.Running,
)

PHASE_OFFSET = {
    "low_hp": 0,
    "flee": 1,
    "regroup": 2,
    "acquire_target": 3,
    "aim": 4,
    "track": 0,
    "dodge": 1,
    "enemy_visible": 2,
    "has_ammo": 3,
    "fire": 4,
    "cooldown": 0,
    "ready_signal": 1,
    "victory_check": 2,
    "retreat_signal": 3,
}

# Four leaves whose Failure would short-circuit the root Sequence before
# downstream subtrees ever render. Substitute Running for Failure on these so
# the chain reaches the bottom branches. They still show Success and Running.
KEEP_ALIVE = {"regroup", "has_ammo", "cooldown", "ready_signal"}


def make_callback(tick_n_ref: list[int]):
    def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
        phase = PHASE_OFFSET.get(args.action, 0)
        idx = (tick_n_ref[0] + phase) % len(CYCLE)
        status = CYCLE[idx]
        if args.action in KEEP_ALIVE and status == bt.Status.Failure:
            status = bt.Status.Running
        return (status, 0.0)

    return cb


def main() -> None:
    tree_bt = bt.BT(build_tree(), None).with_telemetry(8910)
    tick_n_ref = [0]
    callback = make_callback(tick_n_ref)
    print("bonsai-bt visualizer: open http://127.0.0.1:8910/")

    while True:
        tick_n_ref[0] += 1
        result = tree_bt.tick(1.0, callback)
        if result is not None:
            status, _ = result
            if status in (bt.Status.Success, bt.Status.Failure):
                tree_bt.reset_bt()
        time.sleep(0.4)


if __name__ == "__main__":
    main()
