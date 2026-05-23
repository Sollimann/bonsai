"""
Print the graphviz DOT representation of an attack-drone behavior tree.

Builds an attack-drone tree (circle the target, attack when in range, give
up when too far). Calls `BT.graphviz()` to emit
a DOT string. Paste the output into <https://dreampuf.github.io/GraphvizOnline/>
to render the tree visually.

Demonstrates `BT.graphviz()`, and composition with `While` / `Sequence` / `WhenAny` /
`Wait` / `WaitForever` / `Action`.

Run:
    python bonsai-py/examples/graphviz_demo.py
"""
from __future__ import annotations

from dataclasses import dataclass

import bonsai_py as bt

# Payload-less actions are plain strings; payload variants are frozen
# dataclasses (hashable, immutable, work as bt.Action(...) values).
CIRCLING = "Circling"
FLY_TOWARD_PLAYER = "FlyTowardPlayer"


@dataclass(frozen=True)
class PlayerWithinDistance:
    distance: float


@dataclass(frozen=True)
class PlayerFarAwayFromTarget:
    distance: float


@dataclass(frozen=True)
class AttackPlayer:
    damage: float


def build_tree() -> bt.Behavior:
    circling = bt.Action(CIRCLING)
    circle_until_player_within_distance = bt.Sequence([
        bt.While(bt.Wait(5.0), [circling]),
        bt.While(bt.Action(PlayerWithinDistance(50.0)), [circling]),
    ])
    give_up_or_attack = bt.WhenAny([
        bt.Action(PlayerFarAwayFromTarget(100.0)),
        bt.Sequence([
            bt.Action(PlayerWithinDistance(10.0)),
            bt.Action(AttackPlayer(0.1)),
        ]),
    ])
    attack_attempt = bt.While(give_up_or_attack, [bt.Action(FLY_TOWARD_PLAYER)])
    return bt.While(
        bt.WaitForever(),
        [circle_until_player_within_distance, attack_attempt],
    )


def main() -> None:
    tree_bt = bt.BT(build_tree(), {})
    print(tree_bt.graphviz())


if __name__ == "__main__":
    main()
