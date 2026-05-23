"""
Async drone mission with multi-job orchestration.

A drone takes off, checks battery, flies to a mission point (or falls
back to landing at the dock if the battery is low), then lands —
repeating while a background collision-avoidance task runs:

    While(
        cond = AvoidOthers,
        body = [
            TakeOff,
            Select([
                Sequence([CheckBattery, FlyToPoint(10, 10, 10)]),
                FlyToPoint(0, 0, 0),     # dock fallback
            ]),
            Land,
        ],
    )

Each long-running action runs on its own `threading.Thread` and reports
status through a per-job `queue.Queue`. The BT polls non-blockingly with
`get_nowait()`. The BT itself stays on the main asyncio loop. The script
prints the tree's `graphviz()` at the start, then runs the mission until
a wall-clock cap.

Demonstrates `Select` for prioritized fallback, multi-job orchestration
via per-job channels, an asyncio main loop alongside per-job threads,
and `BT.graphviz()` for static tree visualization at startup.

Run:
    python bonsai-py/examples/async_drone.py
"""
from __future__ import annotations

import asyncio
import enum
import queue
import random
import threading
import time
from dataclasses import dataclass, field
from typing import Any, Optional

import bonsai_py as bt

MAX_WALL_SECONDS = 8.0  # Demo cap; the BT itself would loop forever.


class DroneAction(enum.Enum):
    AVOID_OTHERS = enum.auto()
    TAKE_OFF = enum.auto()
    LAND = enum.auto()
    CHECK_BATTERY = enum.auto()


@dataclass(frozen=True)
class FlyToPoint:
    x: float
    y: float
    z: float


@dataclass
class DroneState:
    avoid_others: Optional[queue.Queue[bt.Status]] = None
    takeoff: Optional[queue.Queue[bt.Status]] = None
    land: Optional[queue.Queue[bt.Status]] = None
    fly_to_point: Optional[queue.Queue[bt.Status]] = None


# ---- Background "jobs": one thread per long-running action -----------------
# Each pushes Status.Running every step, then Status.Success when done.

def collision_avoidance_task(q: queue.Queue[bt.Status]) -> None:
    print("collision avoidance task started")
    while True:
        try:
            q.put(bt.Status.Running, timeout=1.0)
        except queue.Full:
            return
        time.sleep(0.1)


def takeoff_task(q: queue.Queue[bt.Status]) -> None:
    print("takeoff task started")
    for i in range(3):
        q.put(bt.Status.Running)
        time.sleep(0.3)
        print(f"takeoff task running for {(i + 1) * 300} ms")
    print("takeoff task finished")
    q.put(bt.Status.Success)


def landing_task(q: queue.Queue[bt.Status]) -> None:
    print("landing task started")
    for i in range(3):
        q.put(bt.Status.Running)
        time.sleep(0.3)
        print(f"landing task running for {(i + 1) * 300} ms")
    print("landing task finished")
    q.put(bt.Status.Success)


def fly_to_point_task(point: FlyToPoint, q: queue.Queue[bt.Status]) -> None:
    print(f"flying task started: target=({point.x}, {point.y}, {point.z})")
    for i in range(3):
        q.put(bt.Status.Running)
        time.sleep(0.5)
        print(f"flying task running for {(i + 1) * 500} ms")
    print("flying task finished")
    q.put(bt.Status.Success)


# ---- Polling helpers -------------------------------------------------------

def poll_job(
    q_attr: str,
    state: DroneState,
    spawn: callable,  # type: ignore[type-arg]
    dt: float,
) -> tuple[bt.Status, float]:
    """Generic 'spawn on first call, poll thereafter' pattern for any threaded job."""
    q = getattr(state, q_attr)
    if q is None:
        q = queue.Queue()
        setattr(state, q_attr, q)
        threading.Thread(target=spawn, args=(q,), daemon=True).start()
    try:
        status = q.get_nowait()
    except queue.Empty:
        return bt.RUNNING
    if status == bt.Status.Running:
        return bt.RUNNING
    setattr(state, q_attr, None)
    return (status, dt)


def make_callback(state: DroneState, rng: random.Random):
    def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
        action = args.action
        if action == DroneAction.AVOID_OTHERS:
            return poll_job("avoid_others", state, collision_avoidance_task, args.dt)
        if action == DroneAction.TAKE_OFF:
            return poll_job("takeoff", state, takeoff_task, args.dt)
        if action == DroneAction.LAND:
            return poll_job("land", state, landing_task, args.dt)
        if action == DroneAction.CHECK_BATTERY:
            # Fast sync action: 80% chance OK, 20% chance low -> Select fallback fires.
            ok = rng.random() < 0.8
            print(f"check battery: {'OK' if ok else 'LOW'}")
            return (bt.Status.Success if ok else bt.Status.Failure, args.dt)
        if isinstance(action, FlyToPoint):
            spawn = lambda q, p=action: fly_to_point_task(p, q)
            return poll_job("fly_to_point", state, spawn, args.dt)
        raise ValueError(f"unknown action: {action!r}")

    return cb


def build_tree() -> bt.Behavior:
    fly_if_healthy = bt.Sequence([
        bt.Action(DroneAction.CHECK_BATTERY),
        bt.Action(FlyToPoint(10.0, 10.0, 10.0)),
    ])
    fly_to_dock = bt.Action(FlyToPoint(0.0, 0.0, 0.0))
    mission_with_fallback = bt.Select([fly_if_healthy, fly_to_dock])
    return bt.While(
        bt.Action(DroneAction.AVOID_OTHERS),
        [
            bt.Action(DroneAction.TAKE_OFF),
            mission_with_fallback,
            bt.Action(DroneAction.LAND),
        ],
    )


async def main() -> None:
    tree_bt = bt.BT(build_tree(), None)
    state = DroneState()
    rng = random.Random(0)
    callback = make_callback(state, rng)
    print("=== drone tree ===")
    print(tree_bt.graphviz())
    print("=== mission start ===")

    start = time.perf_counter()
    last = start
    while True:
        await asyncio.sleep(0.5)
        now = time.perf_counter()
        dt = now - last
        last = now
        result = tree_bt.tick(dt, callback)
        if result is None:
            break
        if now - start > MAX_WALL_SECONDS:
            print(f"=== demo cap reached ({MAX_WALL_SECONDS}s) — exiting ===")
            break


if __name__ == "__main__":
    asyncio.run(main())
