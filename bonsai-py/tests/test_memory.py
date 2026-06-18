"""Memoryless Sequence / Select (``memory=False``) tick semantics through the
PyO3 bindings.

Mirrors the headline Rust integration tests so the Python ``memory=False``
factories produce behaviors with identical short-circuit semantics. Also checks
that the default (``memory=True``) resumes the running child.
"""
from __future__ import annotations

from typing import Any

import bonsai_bt as bt


def _short_circuit_callback(
    *, cond_passes: bool, count_box: list[int]
) -> Any:
    """`cond` returns Success or Failure based on `cond_passes`; `inc` bumps
    `count_box[0]`. Used to assert that later children stay un-ticked when the
    composite short-circuits."""

    def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
        action = args.action
        if action == "cond":
            return (bt.Status.Success if cond_passes else bt.Status.Failure, args.dt)
        if action == "inc":
            count_box[0] += 1
            return (bt.Status.Success, args.dt)
        raise AssertionError(f"unknown action: {action!r}")

    return cb


class TestMemorylessSequenceShortCircuit:
    def test_failure_short_circuits(self) -> None:
        """Failing condition aborts the composite; later children stay un-ticked."""
        count: list[int] = [0]
        tree = bt.Sequence([bt.Action("cond"), bt.Action("inc")], memory=False)
        machine = bt.BT(tree, None)
        status, _dt = machine.tick(
            0.0, _short_circuit_callback(cond_passes=False, count_box=count)
        )
        assert status == bt.Status.Failure
        assert count[0] == 0, "later children must not be ticked on failure"

    def test_all_success(self) -> None:
        """All children succeed -> composite Success and BT is finished."""
        count: list[int] = [0]
        tree = bt.Sequence([bt.Action("cond"), bt.Action("inc")], memory=False)
        machine = bt.BT(tree, None)
        status, _dt = machine.tick(
            0.0, _short_circuit_callback(cond_passes=True, count_box=count)
        )
        assert status == bt.Status.Success
        assert count[0] == 1
        assert machine.is_finished()

    def test_empty_is_success(self) -> None:
        """Empty memoryless Sequence is vacuous Success."""
        tree = bt.Sequence([], memory=False)
        machine = bt.BT(tree, None)
        status, _dt = machine.tick(0.0, lambda _a, _bb: (bt.Status.Success, 0.0))
        assert status == bt.Status.Success


class TestMemorylessSelectShortCircuit:
    def test_success_short_circuits(self) -> None:
        """First succeeding child wins; later children stay un-ticked."""
        count: list[int] = [0]
        tree = bt.Select([bt.Action("cond"), bt.Action("inc")], memory=False)
        machine = bt.BT(tree, None)
        status, _dt = machine.tick(
            0.0, _short_circuit_callback(cond_passes=True, count_box=count)
        )
        assert status == bt.Status.Success
        assert count[0] == 0, "short-circuited siblings must not be ticked"

    def test_all_fail_returns_failure(self) -> None:
        """Every child fails -> composite Failure."""
        def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            if args.action in ("a", "b"):
                return (bt.Status.Failure, args.dt)
            raise AssertionError(args.action)

        tree = bt.Select([bt.Action("a"), bt.Action("b")], memory=False)
        machine = bt.BT(tree, None)
        status, _dt = machine.tick(0.0, cb)
        assert status == bt.Status.Failure

    def test_empty_is_failure(self) -> None:
        """Empty memoryless Select is vacuous Failure (dual of empty Sequence)."""
        tree = bt.Select([], memory=False)
        machine = bt.BT(tree, None)
        status, _dt = machine.tick(0.0, lambda _a, _bb: (bt.Status.Success, 0.0))
        assert status == bt.Status.Failure


class TestMemorylessSequenceReEvaluation:
    def test_running_child_is_aborted_when_earlier_condition_fails_next_tick(self) -> None:
        """A previously-running child must NOT be resumed once an earlier
        condition flips to Failure on the next tick."""
        state = {"cond_passes": True, "long_action_ticks": 0}

        def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            action = args.action
            if action == "cond":
                return (bt.Status.Success if state["cond_passes"] else bt.Status.Failure, args.dt)
            if action == "long":
                state["long_action_ticks"] += 1
                return (bt.Status.Running, 0.0)
            raise AssertionError(action)

        tree = bt.Sequence([bt.Action("cond"), bt.Action("long")], memory=False)
        machine = bt.BT(tree, None)

        # Tick 1: cond passes, long returns Running -> composite Running.
        status1, _ = machine.tick(0.0, cb)
        assert status1 == bt.Status.Running
        assert state["long_action_ticks"] == 1

        # Flip the condition before the next tick.
        state["cond_passes"] = False

        # Tick 2: cond now fails -> composite Failure before long is reached.
        status2, _ = machine.tick(0.0, cb)
        assert status2 == bt.Status.Failure
        assert state["long_action_ticks"] == 1, (
            "the previously-running long action must NOT be re-ticked once the "
            "earlier condition fails"
        )


class TestMemoryDefaultResumes:
    """The default (``memory=True``) resumes the running child rather than
    restarting from the first — the contrast with ``memory=False`` above."""

    def test_default_sequence_resumes_running_child(self) -> None:
        state = {"cond_passes": True, "long_action_ticks": 0}

        def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            action = args.action
            if action == "cond":
                return (bt.Status.Success if state["cond_passes"] else bt.Status.Failure, args.dt)
            if action == "long":
                state["long_action_ticks"] += 1
                return (bt.Status.Running, 0.0)
            raise AssertionError(action)

        # memory=True is the default — omit the flag.
        tree = bt.Sequence([bt.Action("cond"), bt.Action("long")])
        machine = bt.BT(tree, None)

        status1, _ = machine.tick(0.0, cb)
        assert status1 == bt.Status.Running
        assert state["long_action_ticks"] == 1

        # Flip the condition. A *memory* sequence resumes `long` directly and
        # never re-checks `cond`, so the composite stays Running.
        state["cond_passes"] = False
        status2, _ = machine.tick(0.0, cb)
        assert status2 == bt.Status.Running
        assert state["long_action_ticks"] == 2, "the running child resumes, cond is not re-checked"


class TestMemorylessRepr:
    """Repr is ``Sequence(N)`` / ``Select(N)`` for the default and gains a
    ``, memory=False`` suffix for the memoryless variant."""

    def test_repr_includes_memory_flag(self) -> None:
        node = bt.Sequence([bt.Action("a"), bt.Action("b"), bt.Action("c")], memory=False)
        assert repr(node) == "Sequence(3, memory=False)"

    def test_select_repr_includes_memory_flag(self) -> None:
        node = bt.Select([bt.Action("a")], memory=False)
        assert repr(node) == "Select(1, memory=False)"

    def test_default_repr_omits_memory_flag(self) -> None:
        assert repr(bt.Sequence([bt.Action("a")])) == "Sequence(1)"
        assert repr(bt.Select([bt.Action("a")])) == "Select(1)"

    def test_empty_repr(self) -> None:
        assert repr(bt.Sequence([], memory=False)) == "Sequence(0, memory=False)"
        assert repr(bt.Select([], memory=False)) == "Select(0, memory=False)"
