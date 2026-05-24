"""
Boids flocking with one Behavior shared across many agents.

Build the `Behavior` ONCE, then construct `bt.BT(shared_tree, boid)` for
each of N agents — every BT instance has its own blackboard (a `Boid`
dataclass with `x, y, dx, dy`) but they all share the same tree
definition. Each tick runs all 5 flocking rules:

    While(
        cond = WhenAll([FlyTowardsCenter, AvoidOthers]),
        body = [MatchVelocity, LimitSpeed, KeepWithinBounds],
    )

Both cond actions return `Running`, so the BT stays in the body each
tick and all 5 actions fire — updating each boid's velocity and
position. Output is text-only (no graphics window); first and last
boids are logged each tick.

Demonstrates a shared `Behavior` across many `BT` instances,
real-time-loop `dt` integration, `WhenAll` for parallel cond updates,
and `While`-body re-execution per tick.

Run:
    python bonsai-py/examples/boids_console.py
"""
from __future__ import annotations

import enum
import math
import random
import time
from dataclasses import dataclass
from typing import Any

import bonsai_bt as bt

NUM_BOIDS = 10
WIDTH = 1280.0
HEIGHT = 720.0
SPEED_LIMIT = 400.0
VISUAL_RANGE = 32.0
MIN_DISTANCE = 16.0
TICKS = 30
DT_SECONDS = 0.1


class Action(enum.Enum):
    AVOID_OTHERS = enum.auto()
    FLY_TOWARDS_CENTER = enum.auto()
    MATCH_VELOCITY = enum.auto()
    LIMIT_SPEED = enum.auto()
    KEEP_WITHIN_BOUNDS = enum.auto()


@dataclass
class Boid:
    x: float
    y: float
    dx: float
    dy: float

    def distance(self, other: Boid) -> float:
        return math.hypot(self.x - other.x, self.y - other.y)


def build_tree() -> bt.Behavior:
    """One Behavior, shared across all N boid BTs (matches the Rust pattern)."""
    avoid_and_fly = bt.WhenAll([
        bt.Action(Action.FLY_TOWARDS_CENTER),
        bt.Action(Action.AVOID_OTHERS),
    ])
    return bt.While(
        avoid_and_fly,
        [
            bt.Action(Action.MATCH_VELOCITY),
            bt.Action(Action.LIMIT_SPEED),
            bt.Action(Action.KEEP_WITHIN_BOUNDS),
        ],
    )


def make_callback(idx: int, all_boids: list[Boid]):
    """Build a callback closed over this boid's neighbors (via `all_boids`)."""

    def cb(args: Any, boid: Boid) -> tuple[bt.Status, float]:
        others = [b for j, b in enumerate(all_boids) if j != idx]
        match args.action:
            case Action.AVOID_OTHERS:
                move_x = move_y = 0.0
                for other in others:
                    dist = boid.distance(other)
                    if 0.0 < dist < MIN_DISTANCE:
                        move_x += boid.x - other.x
                        move_y += boid.y - other.y
                boid.dx += move_x * 0.5
                boid.dy += move_y * 0.5
                return bt.RUNNING
            case Action.FLY_TOWARDS_CENTER:
                cx = cy = 0.0
                n = 0
                for other in others:
                    if boid.distance(other) < VISUAL_RANGE:
                        cx += other.x
                        cy += other.y
                        n += 1
                if n > 0:
                    boid.dx += (cx / n - boid.x) * 0.05
                    boid.dy += (cy / n - boid.y) * 0.05
                return bt.RUNNING
            case Action.MATCH_VELOCITY:
                avg_dx = avg_dy = 0.0
                n = 0
                for other in others:
                    if boid.distance(other) < VISUAL_RANGE:
                        avg_dx += other.dx
                        avg_dy += other.dy
                        n += 1
                if n > 0:
                    boid.dx += (avg_dx / n - boid.dx) * 0.1
                    boid.dy += (avg_dy / n - boid.dy) * 0.1
                return (bt.Status.Success, args.dt)
            case Action.LIMIT_SPEED:
                speed = math.hypot(boid.dx, boid.dy)
                if speed > SPEED_LIMIT:
                    boid.dx = boid.dx / speed * SPEED_LIMIT
                    boid.dy = boid.dy / speed * SPEED_LIMIT
                return (bt.Status.Success, args.dt)
            case Action.KEEP_WITHIN_BOUNDS:
                edge = 40.0
                turn = 16.0
                if boid.x < edge:
                    boid.dx += turn
                if boid.x > WIDTH - edge:
                    boid.dx -= turn
                if boid.y < edge:
                    boid.dy += turn
                if boid.y > HEIGHT - edge:
                    boid.dy -= turn
                return bt.RUNNING
            case _:
                raise ValueError(f"unknown action: {args.action!r}")

    return cb


def main() -> None:
    rng = random.Random(0)  # deterministic for reproducible console output
    boids = [
        Boid(
            x=rng.uniform(WIDTH / 4, 3 * WIDTH / 4),
            y=rng.uniform(HEIGHT / 4, 3 * HEIGHT / 4),
            dx=(rng.random() - 0.5) * SPEED_LIMIT,
            dy=(rng.random() - 0.5) * SPEED_LIMIT,
        )
        for _ in range(NUM_BOIDS)
    ]

    shared_tree = build_tree()
    bts = [bt.BT(shared_tree, boids[i]) for i in range(NUM_BOIDS)]

    print(f"Boids console demo: {NUM_BOIDS} agents sharing one Behavior tree.")
    for step in range(TICKS):
        for i, tree_bt in enumerate(bts):
            tree_bt.tick(DT_SECONDS, make_callback(i, boids))
            boid = boids[i]
            boid.x += boid.dx * DT_SECONDS
            boid.y += boid.dy * DT_SECONDS
            print(
                f"[boid {i:2d}] step {step:2d}"
                f" pos=({boid.x:7.1f}, {boid.y:7.1f})"
                f" vel=({boid.dx:7.1f}, {boid.dy:7.1f})"
            )
        time.sleep(DT_SECONDS / 10.0)  # tiny pause so output is readable

    print(f"Done after {TICKS} ticks. Each BT instance ticked {bts[0].tick_count()} times.")


if __name__ == "__main__":
    main()
