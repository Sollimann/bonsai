"""
ReactiveSequence and ReactiveSelect demos.

ReactiveSequence: chase while visible. A regular `Sequence` would keep
chasing because it resumes the running child; the reactive variant re-walks
from the first child every tick, so when visibility flips off on tick 3 the
chase aborts immediately.

ReactiveSelect: priority preemption. `Attack` is tried first. While the
enemy is out of range, attack fails and `Chase` runs instead. Once the enemy
enters range, the next tick preempts the chase and runs attack.

Run:
    python bonsai-py/examples/reactive_chase.py
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Any

import bonsai_bt as bt


@dataclass
class World:
    visible: bool = False
    in_range: bool = False
    chase_progress: int = 0
    attack_progress: int = 0


def make_chase_callback(world: World):
    def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
        if args.action == "visible":
            status = bt.Status.Success if world.visible else bt.Status.Failure
            return (status, args.dt)
        if args.action == "chase":
            world.chase_progress += 1
            return (bt.Status.Running, 0.0)
        raise ValueError(f"unknown action: {args.action!r}")

    return cb


def make_priority_callback(world: World):
    def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
        if args.action == "attack":
            if world.in_range:
                world.attack_progress += 1
                return (bt.Status.Running, 0.0)
            return (bt.Status.Failure, args.dt)
        if args.action == "chase":
            world.chase_progress += 1
            return (bt.Status.Running, 0.0)
        raise ValueError(f"unknown action: {args.action!r}")

    return cb


def run_chase_demo() -> None:
    print("=== ReactiveSequence: chase the enemy while it stays visible ===")
    print("Setup: enemy starts visible. At tick 3 it vanishes from sight.\n")

    world = World(visible=True)
    tree = bt.ReactiveSequence([bt.Action("visible"), bt.Action("chase")])
    machine = bt.BT(tree, {})
    callback = make_chase_callback(world)

    for t in range(6):
        if t == 3:
            world.visible = False
            print("  >>> enemy vanishes; leading condition will now fail <<<")

        machine.tick(0.1, callback)

        state = "visible" if world.visible else "hidden "
        activity = "chasing" if world.visible else "idle (chase aborted; steps frozen)"
        print(
            f"tick {t}: enemy {state}  ->  {activity:36}  "
            f"chase_steps={world.chase_progress}"
        )

        if machine.is_finished():
            machine.reset_bt()


def run_priority_demo() -> None:
    print("\n=== ReactiveSelect: prefer attack when in range; otherwise chase ===")
    print("Setup: enemy starts out of range. At tick 3 it closes to melee.\n")

    world = World()
    tree = bt.ReactiveSelect([bt.Action("attack"), bt.Action("chase")])
    machine = bt.BT(tree, {})
    callback = make_priority_callback(world)

    for t in range(6):
        if t == 3:
            world.in_range = True
            print("  >>> enemy enters range; Attack now succeeds and preempts Chase <<<")

        machine.tick(0.1, callback)

        state = "in range    " if world.in_range else "out of range"
        activity = "attacking" if world.in_range else "attack fails -> falling back to chase"
        print(
            f"tick {t}: enemy {state}  ->  {activity:38}  "
            f"chase={world.chase_progress}, attack={world.attack_progress}"
        )

        if machine.is_finished():
            machine.reset_bt()


def main() -> None:
    run_chase_demo()
    run_priority_demo()


if __name__ == "__main__":
    main()
