"""
Console NPC behavior demo.

An NPC runs and shoots while it has action points. When exhausted, it rests
until fully recovered, then dies. Built from a nested `WhileAll`:

    WhileAll(IsDead, [
        WhileAll(HasActionPointsLeft, [Run, Shoot]),
        Rest,
        Die,
    ])

Demonstrates `WhileAll` looping, blackboard mutation through a `@dataclass`,
and an enum action dispatched via Python's structural-match callback.

Run:
    python bonsai-py/examples/simple_npc_ai.py
"""
from __future__ import annotations

import enum
from dataclasses import dataclass
from typing import Any

import bonsai_py as bt


class EnemyNPC(enum.Enum):
    RUN = enum.auto()
    SHOOT = enum.auto()
    HAS_ACTION_POINTS_LEFT = enum.auto()
    REST = enum.auto()
    DIE = enum.auto()
    IS_DEAD = enum.auto()


@dataclass
class BlackBoard:
    times_shot: int = 0


@dataclass
class NPCState:
    action_points: int
    max_action_points: int
    alive: bool

    def consume_action_point(self) -> None:
        self.action_points = max(0, self.action_points - 1)

    def rest(self) -> None:
        self.action_points = min(self.action_points + 1, self.max_action_points)
        print(f"Rested for a while... Action points: {self.action_points}")

    def die(self) -> None:
        print("NPC died...")
        self.alive = False

    def is_alive(self) -> bool:
        print("NPC is alive..." if self.alive else "NPC is dead...")
        return self.alive

    def fully_rested(self) -> bool:
        return self.action_points == self.max_action_points

    def perform_action(self, action: str) -> None:
        if self.action_points > 0:
            self.consume_action_point()
            print(f"Performing action: {action}. Action points: {self.action_points}")
        else:
            print(f"Cannot perform action: {action}. Not enough action points.")


def make_callback(state: NPCState):
    def cb(args: Any, blackboard: BlackBoard) -> tuple[bt.Status, float]:
        match args.action:
            case EnemyNPC.RUN:
                state.perform_action("run")
                return (bt.Status.Success, 0.0)
            case EnemyNPC.HAS_ACTION_POINTS_LEFT:
                if state.action_points == 0:
                    print("NPC does not have action points left...")
                    return (bt.Status.Success, 0.0)
                print(f"NPC has action points: {state.action_points}")
                return (bt.Status.Running, 0.0)
            case EnemyNPC.SHOOT:
                state.perform_action("shoot")
                blackboard.times_shot += 1
                return (bt.Status.Success, 0.0)
            case EnemyNPC.REST:
                if state.fully_rested():
                    return (bt.Status.Success, 0.0)
                state.rest()
                return (bt.Status.Running, 0.0)
            case EnemyNPC.DIE:
                state.die()
                return (bt.Status.Success, 0.0)
            case EnemyNPC.IS_DEAD:
                if state.is_alive():
                    return (bt.Status.Running, 0.0)
                return (bt.Status.Success, 0.0)
            case _:
                raise ValueError(f"unknown action: {args.action!r}")

    return cb


def build_tree() -> bt.Behavior:
    run_and_shoot = bt.WhileAll(
        bt.Action(EnemyNPC.HAS_ACTION_POINTS_LEFT),
        [bt.Action(EnemyNPC.RUN), bt.Action(EnemyNPC.SHOOT)],
    )
    return bt.WhileAll(
        bt.Action(EnemyNPC.IS_DEAD),
        [run_and_shoot, bt.Action(EnemyNPC.REST), bt.Action(EnemyNPC.DIE)],
    )


def main() -> None:
    max_actions = 3
    blackboard = BlackBoard()
    state = NPCState(action_points=max_actions, max_action_points=max_actions, alive=True)
    tree_bt = bt.BT(build_tree(), blackboard)
    callback = make_callback(state)

    while True:
        print("reached main loop...")
        result = tree_bt.tick(0.0, callback)
        if result is None:
            break
        status, _ = result
        if status != bt.Status.Running:
            break

    print(f"NPC shot {blackboard.times_shot} times during the simulation.")


if __name__ == "__main__":
    main()
