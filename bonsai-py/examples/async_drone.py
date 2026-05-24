"""
Async drone mission with multi-job orchestration (asyncio-native).

Same mission as `threaded_drone.py`:

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

…but each long-running action is an `async def` coroutine scheduled on
the same asyncio event loop as the BT tick loop, communicating with the
BT via `asyncio.Queue`. No OS threads — every job is cooperatively
scheduled on a single thread.

**Pick this variant when actions are awaitable** — `aiohttp`, async DB
drivers (`asyncpg`, `motor`), websockets, async stream readers, async
subprocess.

How the integration works:

* `BT.tick()` is synchronous. The callback we pass to it is also
  synchronous — it polls each per-job `asyncio.Queue` with the
  non-blocking `get_nowait()`. The callback never awaits.
* On the first call for a given action, the callback spawns the job
  coroutine via `asyncio.create_task(job(q))`. The task starts running
  on the next loop iteration.
* The main loop is `async def` and uses `await asyncio.sleep(0.5)` to
  yield control back to the event loop between BT ticks. While main
  is sleeping, the job coroutines run and push status updates into
  their queues.

Run:
    python bonsai-py/examples/async_drone.py
"""
from __future__ import annotations

import asyncio
import enum
import random
import time
from dataclasses import dataclass
from typing import Any, Awaitable, Callable, Optional

import bonsai_bt as bt

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
    avoid_others: Optional[asyncio.Queue[bt.Status]] = None
    takeoff: Optional[asyncio.Queue[bt.Status]] = None
    land: Optional[asyncio.Queue[bt.Status]] = None
    fly_to_point: Optional[asyncio.Queue[bt.Status]] = None


# ---- Background "jobs": one coroutine per long-running action --------------
# Each pushes Status.Running every step, then Status.Success when done.

async def collision_avoidance_task(q: asyncio.Queue[bt.Status]) -> None:
    print("collision avoidance task started")
    while True:
        q.put_nowait(bt.Status.Running)
        await asyncio.sleep(0.1)


async def takeoff_task(q: asyncio.Queue[bt.Status]) -> None:
    print("takeoff task started")
    for i in range(3):
        q.put_nowait(bt.Status.Running)
        await asyncio.sleep(0.3)
        print(f"takeoff task running for {(i + 1) * 300} ms")
    print("takeoff task finished")
    q.put_nowait(bt.Status.Success)


async def landing_task(q: asyncio.Queue[bt.Status]) -> None:
    print("landing task started")
    for i in range(3):
        q.put_nowait(bt.Status.Running)
        await asyncio.sleep(0.3)
        print(f"landing task running for {(i + 1) * 300} ms")
    print("landing task finished")
    q.put_nowait(bt.Status.Success)


async def fly_to_point_task(point: FlyToPoint, q: asyncio.Queue[bt.Status]) -> None:
    print(f"flying task started: target=({point.x}, {point.y}, {point.z})")
    for i in range(3):
        q.put_nowait(bt.Status.Running)
        await asyncio.sleep(0.5)
        print(f"flying task running for {(i + 1) * 500} ms")
    print("flying task finished")
    q.put_nowait(bt.Status.Success)


# ---- Polling helpers -------------------------------------------------------

SpawnCoro = Callable[[asyncio.Queue[bt.Status]], Awaitable[None]]


def poll_job(
    q_attr: str,
    state: DroneState,
    spawn: SpawnCoro,
    dt: float,
) -> tuple[bt.Status, float]:
    """Generic 'schedule on first call, poll thereafter' pattern for any async job.

    Calls `asyncio.create_task(spawn(q))` on first invocation; subsequent
    invocations drain the queue non-blockingly. Must be called from inside
    a running event loop (the BT tick runs from inside `await asyncio.sleep`
    in `main`, so this holds).
    """
    q = getattr(state, q_attr)
    if q is None:
        q = asyncio.Queue()
        setattr(state, q_attr, q)
        asyncio.create_task(spawn(q))
    try:
        status = q.get_nowait()
    except asyncio.QueueEmpty:
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
            async def spawn(q: asyncio.Queue[bt.Status], p: FlyToPoint = action) -> None:
                await fly_to_point_task(p, q)
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
