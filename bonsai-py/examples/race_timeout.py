# -*- coding: utf-8 -*-
"""
Race a simulated long-running job against a hard timeout.

A `Race` runs two arms in parallel; whichever finishes first wins:

    Race([
        DO_WORK,                              # random 200-1200 ms
        Sequence([Wait(0.6), ON_TIMEOUT]),    # 600 ms hard deadline
    ])

`DO_WORK` runs on a background `threading.Thread` and reports its status
through a `queue.Queue`. The BT polls non-blockingly with `get_nowait()`;
an empty queue means "still running, try again next tick."

Demonstrates `Race` for timeouts, an asyncio main loop alongside a
threading worker (the `BT` itself stays on the main thread — it is
unsendable), the `bt.RUNNING` shorthand, and `time.perf_counter()` for
monotonic dt.

Run:
    python bonsai-py/examples/race_timeout.py
"""
from __future__ import annotations

import asyncio
import enum
import queue
import random
import threading
import time
from dataclasses import dataclass, field
from typing import Any

import bonsai_py as bt


class MissionAction(enum.Enum):
    DO_WORK = enum.auto()
    ON_TIMEOUT = enum.auto()


@dataclass
class MissionState:
    work: queue.Queue[bt.Status] | None = field(default=None)


def do_work_task(q: queue.Queue[bt.Status]) -> None:
    """Background thread: sleeps in 100ms steps, sends Status.Running each step,
    then Status.Success after a random 200..=1200ms total."""
    work_ms = random.randint(200, 1200)
    print(f"[do_work] started; planned duration {work_ms} ms")
    elapsed = 0
    step_ms = 100
    while elapsed < work_ms:
        q.put(bt.Status.Running)
        time.sleep(step_ms / 1000.0)
        elapsed += step_ms
    print(f"[do_work] finished after {elapsed} ms")
    q.put(bt.Status.Success)


def make_callback(state: MissionState):
    def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
        match args.action:
            case MissionAction.DO_WORK:
                if state.work is None:
                    state.work = queue.Queue()
                    threading.Thread(
                        target=do_work_task, args=(state.work,), daemon=True
                    ).start()
                try:
                    status = state.work.get_nowait()
                except queue.Empty:
                    return bt.RUNNING
                if status == bt.Status.Running:
                    return bt.RUNNING
                state.work = None
                return (status, args.dt)
            case MissionAction.ON_TIMEOUT:
                print("do_work timed out!")
                return (bt.Status.Failure, args.dt)
            case _:
                raise ValueError(f"unknown action: {args.action!r}")

    return cb


async def main() -> None:
    timeout_s = 0.6
    tree = bt.Sequence([
        bt.Race([
            bt.Action(MissionAction.DO_WORK),
            bt.Sequence([bt.Wait(timeout_s), bt.Action(MissionAction.ON_TIMEOUT)]),
        ]),
    ])
    tree_bt = bt.BT(tree, None)
    state = MissionState()
    callback = make_callback(state)
    last = time.perf_counter()

    while True:
        await asyncio.sleep(0.05)
        now = time.perf_counter()
        dt = now - last
        last = now
        result = tree_bt.tick(dt, callback)
        if result is None:
            break


if __name__ == "__main__":
    asyncio.run(main())
